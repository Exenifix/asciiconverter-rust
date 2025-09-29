use image::*;

const CONVERSION_CHARS: [char; 6] = ['.', '~', ':', '*', '#', '@'];


fn get_processed_image(path: &str, resolution: f32) -> Result<String, Box<dyn std::error::Error>> {
    let img = ImageReader::open(path)?.decode()?.to_luma8();
    let (mut new_w, mut new_h) = img.dimensions();
    new_w = (new_w as f32 * resolution) as u32;
    new_h = (new_h as f32 * resolution) as u32;
    let resized_image = imageops::resize(&img, new_w, new_h, imageops::FilterType::Triangle);
    let mut output = String::new();
    for y in 0..new_h {
        for x in 0..new_w {
            let brightness = resized_image.get_pixel(x, y).0[0];
            println!("{}", brightness);
            output.push(CONVERSION_CHARS[(brightness as f32 / 255. * (CONVERSION_CHARS.len() - 1) as f32) as usize]);
        }
        output.push('\n');
    }

    Ok(output)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = get_processed_image("res/test_logo.png", 0.15)?;
    println!("{}", output);

    Ok(())
}
