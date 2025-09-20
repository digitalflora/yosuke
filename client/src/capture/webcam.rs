// god bless claude's little cotton socks
use nokhwa::{
    Camera, NokhwaError,
    pixel_format::RgbAFormat,
    utils::{CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType},
};
use shared::commands::{BaseResponse, CapturePacket, CaptureQuality, CaptureType, Response};
use smol::channel::Sender;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use crate::{
    capture::jpeg::{FrameSize, encode_fast},
    handler::send,
};

pub fn main(
    id: u64,
    tx: Sender<Vec<u8>>,
    running: Arc<AtomicBool>,
    quality: CaptureQuality,
    device: u32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Query available cameras with error handling
    let cameras = nokhwa::query(nokhwa::utils::ApiBackend::Auto)
        .map_err(|e| format!("Failed to query cameras: {}", e))?;

    if cameras.is_empty() {
        return Err("No cameras found".into());
    }

    let index = CameraIndex::Index(device);
    let requested =
        RequestedFormat::new::<RgbAFormat>(RequestedFormatType::AbsoluteHighestFrameRate);

    // Create camera with error handling
    let mut camera = Camera::new(index, requested).map_err(|e| {
        match &e {
            NokhwaError::OpenDeviceError(msg, _) => {
                eprintln!("[!] Camera might be in use by another application: {}", msg);
            }
            NokhwaError::GetPropertyError { property, error } => {
                eprintln!(
                    "[!] Failed to get camera property '{}': {}",
                    property, error
                );
            }
            _ => {
                eprintln!("[!] Camera initialization failed with unexpected error");
            }
        }
        format!("Failed to create camera: {}", e)
    })?;

    // Open camera stream with retry logic for busy camera
    let mut retry_count = 0;
    const MAX_RETRIES: u32 = 2;
    const RETRY_DELAY: Duration = Duration::from_secs(2);

    loop {
        match camera.open_stream() {
            Ok(()) => {
                println!("[*] Successfully opened camera stream");
                break;
            }
            Err(e) => {
                eprintln!("[!] Failed to open camera stream: {}", e);

                match &e {
                    NokhwaError::OpenStreamError(msg) => {
                        if msg.contains("busy")
                            || msg.contains("in use")
                            || msg.contains("occupied")
                        {
                            eprintln!("[!] Camera is busy/in use by another application");

                            if retry_count < MAX_RETRIES {
                                retry_count += 1;
                                eprintln!(
                                    "[*] Retrying in {} seconds... (attempt {}/{})",
                                    RETRY_DELAY.as_secs(),
                                    retry_count,
                                    MAX_RETRIES
                                );
                                std::thread::sleep(RETRY_DELAY);
                                continue;
                            } else {
                                return Err("Max retries reached. Camera remains busy.".into());
                            }
                        } else {
                            return Err(format!("Stream error: {}", msg).into());
                        }
                    }
                    NokhwaError::SetPropertyError {
                        property,
                        value,
                        error,
                    } => {
                        return Err(format!(
                            "Failed to set property '{}' to '{}': {}",
                            property, value, error
                        )
                        .into());
                    }
                    _ => {
                        return Err("Unexpected error opening stream".into());
                    }
                }
            }
        }
    }

    println!("[*] cam name:   {:?}", camera.info().human_name());
    println!("[*] cam format: {:?}", camera.camera_format());

    let mut consecutive_errors = 0;
    const MAX_CONSECUTIVE_ERRORS: u32 = 10;

    if quality == CaptureQuality::Speed {
        if let Err(err) = camera.set_resolution(nokhwa::utils::Resolution {
            width_x: 640,
            height_y: 480,
        }) {
            eprintln!("[!] Failed to set resolution: {}", err);
        };
        if let Err(err) = camera.set_frame_rate(30) {
            eprintln!("[!] Failed to set frame rate: {}", err);
        };
    }

    let mut buf = Vec::new();
    let mut rgb_buf = Vec::new();

    loop {
        if !running.load(Ordering::SeqCst) {
            println!(
                "[v] signal to stop capturing! shutting capture, clearing buffer, breaking loop"
            );
            // mop up part 6: the heart
            drop(buf);
            drop(rgb_buf);
            if let Err(e) = camera.stop_stream() {
                eprintln!("[!] Error stopping camera stream: {}", e);
            }
            drop(camera); // fakemink kill everything
            break;
        }

        match camera.frame() {
            Ok(frame) => {
                println!("[*] got camera frame");
                consecutive_errors = 0; // Reset error counter on success

                let (width, height) = if frame.source_frame_format() == FrameFormat::MJPEG {
                    println!("[*] frame was MJPEG");
                    match image::load_from_memory(frame.buffer()) {
                        Ok(image) => {
                            buf = image.to_rgba8().into_raw();
                            (image.width(), image.height())
                        }
                        Err(e) => {
                            eprintln!("[!] Failed to decode MJPEG frame: {}", e);
                            consecutive_errors += 1;
                            if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                                return Err("Too many consecutive decode errors".into());
                            }
                            continue;
                        }
                    }
                } else {
                    println!("[*] frame was not MJPEG");
                    buf.clear();
                    buf.extend_from_slice(frame.buffer());
                    (frame.resolution().width(), frame.resolution().height())
                };

                // The jpeg encoder expects BGRA, but nokhwa provides RGBA. Swap R and B.
                for chunk in buf.chunks_mut(4) {
                    if chunk.len() == 4 {
                        chunk.swap(0, 2);
                    }
                }

                let (max_width, max_height) = match quality {
                    CaptureQuality::Quality => (1280.0, 720.0),
                    CaptureQuality::Speed => (640.0, 480.0),
                };

                let aspect_ratio = width as f32 / height as f32;
                let (target_width, target_height) = if aspect_ratio > max_width / max_height {
                    (max_width as u32, (max_width / aspect_ratio) as u32)
                } else {
                    ((max_height * aspect_ratio) as u32, max_height as u32)
                };

                let mut jpeg_quality = 60;
                if quality == CaptureQuality::Quality {
                    jpeg_quality = 80;
                }

                let packet = encode_fast(
                    &buf,
                    FrameSize { width, height },
                    FrameSize {
                        width: target_width as u32,
                        height: target_height as u32,
                    },
                    jpeg_quality,
                    &mut rgb_buf,
                );

                println!("[*] sending response");
                send(
                    BaseResponse {
                        id: id,
                        response: Response::CapturePacket(
                            CaptureType::Camera,
                            CapturePacket::Video(packet),
                        ),
                    },
                    &tx,
                );
            }
            Err(e) => {
                consecutive_errors += 1;
                eprintln!(
                    "[!] Failed to capture frame (error {}/{}): {}",
                    consecutive_errors, MAX_CONSECUTIVE_ERRORS, e
                );

                match &e {
                    NokhwaError::ReadFrameError(msg) => {
                        if msg.contains("busy") || msg.contains("disconnected") {
                            return Err("Camera became unavailable during capture".into());
                        }
                    }
                    NokhwaError::GetPropertyError { property, error } => {
                        eprintln!("[!] Property access error for '{}': {}", property, error);
                    }
                    _ => {
                        eprintln!("[!] Unexpected frame capture error");
                    }
                }

                if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                    return Err("Too many consecutive frame errors".into());
                }

                // Brief pause before retrying
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }

    Ok(())
}
