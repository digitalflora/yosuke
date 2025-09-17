use scrap::{Capturer, Display};
use shared::commands::{BaseResponse, CapturePacket, CaptureType, Response, VideoPacket};
use smol::channel::Sender;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use crate::{
    capture::jpeg::{FrameSize, encode, encode_fast},
    handler::send,
};

fn stride(frame: &[u8], width: usize, height: usize) -> Vec<u8> {
    let expected_size = width * height * 4;

    if frame.len() == expected_size {
        return frame.to_vec();
    }

    let bytes_per_row = frame.len() / height;
    let expected_bytes_per_row = width * 4;
    let mut fixed_data = Vec::with_capacity(expected_size);

    for y in 0..height {
        let row_start = y * bytes_per_row;
        let row_end = row_start + expected_bytes_per_row;

        if row_end <= frame.len() {
            fixed_data.extend_from_slice(&frame[row_start..row_end]);
        }
    }

    fixed_data
}

pub fn main(id: u64, tx: Sender<Vec<u8>>, running: Arc<AtomicBool>) {
    let display = Display::primary().unwrap();
    let (width, height) = (display.width(), display.height());
    let mut capturer = Capturer::new(display).unwrap();
    let resize_factor = 2.5;
    let (target_width, target_height) = (
        (width as f32 / resize_factor) as usize,
        (height as f32 / resize_factor) as usize,
    );

    loop {
        if !running.load(Ordering::SeqCst) {
            println!("[v] signal to stop capturing! breaking loop");
            break;
        }

        // let frame_start = Instant::now();
        if let Ok(frame) = capturer.frame() {
            //let capture_start = Instant::now();
            println!("[*] got frame of screen capture");
            //let capture_time = capture_start.elapsed();

            let fixed_frame = stride(&frame, width, height);

            //let compress_start = Instant::now();

            let packet: VideoPacket = encode_fast(
                fixed_frame,
                FrameSize {
                    // from
                    width: width as u32,
                    height: height as u32,
                },
                FrameSize {
                    // to
                    width: target_width as u32,
                    height: target_height as u32,
                },
            );

            //let compress_time = compress_start.elapsed();

            //println!("[*] captured in {:?}", capture_time);
            //println!("[*] compressed in {:?}", compress_time);

            send(
                BaseResponse {
                    id: id,
                    response: Response::CapturePacket(
                        CaptureType::Screen,
                        CapturePacket::Video(packet),
                    ),
                },
                &tx,
            );
        }
        thread::sleep(Duration::from_millis(50)); // send it flying
    }
}
