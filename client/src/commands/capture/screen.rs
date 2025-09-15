// Result<Response, Box<dyn std::error::Error>> {
use image::{self, DynamicImage, GenericImageView};
use shared::commands::{Response, ScreenshotResponse};
use xcap::Monitor;

fn compress(image: &DynamicImage) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error>> {
    // Resize to 480p
    let (o_width, o_height) = image.dimensions();
    let aspect_ratio = o_width as f32 / o_height as f32;
    let target_width = (480.0 * aspect_ratio).round() as u32;
    let target_height = 480;

    let resized = image::imageops::resize(
        image,
        target_width,
        target_height,
        image::imageops::FilterType::Triangle,
    );

    println!("hard part incoming");

    let (width, height) = resized.dimensions();
    let rgb_bytes: Vec<u8> = resized
        .pixels()
        .flat_map(|pixel| [pixel[0], pixel[1], pixel[2]]) // Take R, G, B, drop A
        .collect();

    println!("hard part finished");

    // Compress to WebP (much faster than JPEG)
    let encoder = webp::Encoder::from_rgb(&rgb_bytes, width, height);
    let data = encoder.encode(90.0);

    // Return (vec[u8], width, height)
    Ok((data.to_vec(), width, height))
}

pub fn main() -> Result<Response, Box<dyn std::error::Error>> {
    println!("[*] Finding screens");
    match Monitor::all() {
        Ok(monitors) => {
            if let Some(monitor) = monitors.first() {
                println!("[*] Capturing screen");
                match monitor.capture_image() {
                    Ok(image) => {
                        // xcap returns an RgbaImage, convert to RGB for JPEG compatibility
                        let rgb_image =
                            image::ImageBuffer::from_fn(image.width(), image.height(), |x, y| {
                                let rgba = image.get_pixel(x, y);
                                image::Rgb([rgba[0], rgba[1], rgba[2]])
                            });
                        let dynamic_image = DynamicImage::ImageRgb8(rgb_image);
                        println!("[*] Compressing image");
                        match compress(&dynamic_image) {
                            Ok((dat, width, height)) => {
                                Ok(Response::Screenshot(ScreenshotResponse {
                                    height: height,
                                    width: width,
                                    data: dat,
                                }))
                            }
                            Err(e) => Ok(Response::Error(format!("Compression failed: {}", e))),
                        }
                    }
                    Err(e) => Ok(Response::Error(format!("Capture failed: {}", e))),
                }
            } else {
                Ok(Response::Error(format!("No screen to capture")))
            }
        }
        Err(e) => Ok(Response::Error(format!("Failed to get monitors: {}", e))),
    }
}
