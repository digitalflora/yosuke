use std::io::Cursor;

use image::{ColorType, ImageBuffer, Rgb, imageops::FilterType};
use shared::commands::VideoPacket;

pub struct FrameSize {
    pub width: u32,
    pub height: u32,
}

pub fn encode(frame: Vec<u8>, from: FrameSize, to: FrameSize) -> VideoPacket {
    let rgb: Vec<u8> = frame
        .chunks(4)
        .flat_map(|bgra| [bgra[2], bgra[1], bgra[0]]) // convert BGR to RGB
        .collect();

    let img =
        ImageBuffer::<Rgb<u8>, _>::from_raw(from.width as u32, from.height as u32, rgb).unwrap();
    let final_img = if to.width != from.width || to.height != from.height {
        image::imageops::resize(&img, to.width as u32, to.height as u32, FilterType::Nearest)
    } else {
        img
    };

    let mut jpeg_data = Vec::new();
    {
        let mut cursor = Cursor::new(&mut jpeg_data);
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, 60);
        encoder
            .encode(
                final_img.as_raw(),
                to.width as u32,
                to.height as u32,
                ColorType::Rgb8.into(),
            )
            .unwrap();
    };

    VideoPacket {
        data: jpeg_data,
        width: to.width as u32,
        height: to.height as u32,
    }
}
