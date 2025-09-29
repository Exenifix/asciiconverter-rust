use std::cmp::max;
use image::*;

const CONVERSION_CHARS: [char; 7] = [' ', '.', '~', ':', '*', '#', '@'];

fn normalize_pixels(img: ImageBuffer<Luma<u8>, Vec<u8>>) -> ImageBuffer<Luma<u8>, Vec<u8>> where
{
    let mut max_val = 0;
    for pixel in img.enumerate_pixels() {
        max_val = max(max_val, pixel.2[0]);
    }

    ImageBuffer::from_fn(img.width(), img.height(), |x, y| {
        Luma::from([(img.get_pixel(x, y).0[0] as f32 / max_val as f32 * 255.) as u8])
    })
}

fn process_image(path: &str, resolution: f32) -> Result<String, Box<dyn std::error::Error>> {
    let img = ImageReader::open(path)?.decode()?.to_luma8();
    let (mut new_w, mut new_h) = img.dimensions();
    new_w = (new_w as f32 * resolution) as u32;
    new_h = (new_h as f32 * resolution) as u32;
    let resized_image = imageops::resize(&img, new_w, new_h, imageops::FilterType::Triangle);
    let normalized_image = normalize_pixels(resized_image);
    let mut output = String::new();
    for y in 0..new_h {
        for x in 0..new_w {
            let brightness = normalized_image.get_pixel(x, y).0[0];
            output.push(CONVERSION_CHARS[(brightness as f32 / 255. * (CONVERSION_CHARS.len() - 1) as f32) as usize]);
        }
        output.push('\n');
    }

    Ok(output)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = process_image("res/test_logo.png", 0.15)?;
    println!("{}", output);

    Ok(())
}
