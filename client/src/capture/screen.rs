// SHOUTOUT TO CLAUDE SONNET 4
// U SAVED MY MOTHAFKN LIFE.

use shared::commands::{
    BaseResponse, CapturePacket, CaptureQuality, CaptureType, Response, VideoPacket,
};
use smol::channel::Sender;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
#[cfg(windows)]
use winapi;
use windows_capture::{
    capture::{Context, GraphicsCaptureApiHandler},
    frame::Frame,
    graphics_capture_api::InternalCaptureControl,
    monitor::Monitor,
    settings::{
        ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
        MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
    },
};

use crate::{
    capture::jpeg::{FrameSize, encode_fast},
    handler::send,
};

struct CaptureHandler {
    id: u64,
    tx: Sender<Vec<u8>>,
    running: Arc<AtomicBool>,
    quality: CaptureQuality,
    target_width: usize,
    target_height: usize,
    frame_vec: Vec<u8>,
    rgb_buf: Vec<u8>,
}

impl GraphicsCaptureApiHandler for CaptureHandler {
    type Flags = (
        u64,
        Sender<Vec<u8>>,
        Arc<AtomicBool>,
        CaptureQuality,
        usize,
        usize,
        usize,
    );
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        let (id, tx, running, quality, target_width, target_height, initial_capacity) = ctx.flags;
        Ok(Self {
            id,
            tx,
            running,
            quality,
            target_width,
            target_height,
            frame_vec: Vec::with_capacity(initial_capacity),
            rgb_buf: Vec::new(),
        })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        if !self.running.load(Ordering::SeqCst) {
            capture_control.stop();
            return Ok(());
        }

        // Get frame dimensions
        let width = frame.width() as usize;
        let height = frame.height() as usize;

        // Get the raw frame data
        let mut frame_buffer = frame.buffer()?;
        let frame_data = frame_buffer.as_raw_buffer();

        // Process the frame data - try without conversion first
        self.stride(frame_data, width, height);

        let mut jpeg_quality = 60;
        if self.quality == CaptureQuality::Quality {
            jpeg_quality = 80;
        }

        let packet: VideoPacket = encode_fast(
            &self.frame_vec,
            FrameSize {
                // from
                width: width as u32,
                height: height as u32,
            },
            FrameSize {
                // to
                width: self.target_width as u32,
                height: self.target_height as u32,
            },
            jpeg_quality,
            &mut self.rgb_buf,
        );

        send(
            BaseResponse {
                id: self.id,
                response: Response::CapturePacket(
                    CaptureType::Screen,
                    CapturePacket::Video(packet),
                ),
            },
            &self.tx,
        );

        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), Self::Error> {
        println!("[*] Capture session closed");
        Ok(())
    }
}

impl CaptureHandler {
    fn stride(&mut self, frame: &[u8], width: usize, height: usize) {
        self.frame_vec.clear();

        let expected_size = width * height * 4;

        if frame.len() == expected_size {
            self.frame_vec.extend_from_slice(frame);
            return;
        }

        let bytes_per_row = frame.len() / height;
        let expected_bytes_per_row = width * 4;

        for y in 0..height {
            let row_start = y * bytes_per_row;
            let row_end = row_start + expected_bytes_per_row;

            if row_end <= frame.len() {
                self.frame_vec.extend_from_slice(&frame[row_start..row_end]);
            }
        }
    }
}

pub fn main(
    id: u64,
    tx: Sender<Vec<u8>>,
    running: Arc<AtomicBool>,
    quality: CaptureQuality,
    device: u32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Set DPI awareness to system aware
    #[cfg(windows)]
    unsafe {
        winapi::um::winuser::SetProcessDpiAwarenessContext(
            winapi::shared::windef::DPI_AWARENESS_CONTEXT_SYSTEM_AWARE,
        );
    }

    // Get the specified monitor
    let monitors = Monitor::enumerate().map_err(|e| e.to_string())?;
    let monitor = monitors
        .into_iter()
        .nth(device as usize)
        .ok_or("Monitor not found")?;

    // Get monitor dimensions for calculating target size
    let (width, height) = (
        monitor.width().map_err(|e| e.to_string())? as usize,
        monitor.height().map_err(|e| e.to_string())? as usize,
    );

    let mut resize_factor = 4.0;
    if quality == CaptureQuality::Quality {
        resize_factor = 2.0;
    }
    let (target_width, target_height) = (
        (width as f32 / resize_factor) as usize,
        (height as f32 / resize_factor) as usize,
    );

    let initial_capacity = width * height * 4;

    // Configure capture settings with flags containing all necessary data
    let settings = Settings::new(
        monitor,
        CursorCaptureSettings::Default,
        DrawBorderSettings::WithoutBorder, // Disable the yellow border
        SecondaryWindowSettings::Default,
        MinimumUpdateIntervalSettings::Default,
        DirtyRegionSettings::Default,
        ColorFormat::Bgra8, // Try RGBA8 first
        (
            id,
            tx,
            running.clone(),
            quality,
            target_width,
            target_height,
            initial_capacity,
        ),
    );

    // Start capture session
    CaptureHandler::start(settings)?;
    println!("[*] capturer should be dropped now!!");

    Ok(())
}
