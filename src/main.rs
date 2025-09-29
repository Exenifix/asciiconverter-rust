use clap::{Parser, ValueEnum};
use image::*;

#[derive(Clone, ValueEnum)]
enum ColorMode {
    RGB,
    Grayscale,
}

#[derive(Clone, ValueEnum)]
enum Charset {
    Simple,
    Detailed,
    Braille,
    Blocks,
}

impl Charset {
    fn get_charset(&self) -> &'static [char] {
        match self {
            Charset::Simple => &[' ', '.', '~', ':', '*', '#', '@'],
            Charset::Detailed => &[' ', '.', '^', '~', ':', '*', '=', '+', '!', '?', '#', '@'],
            Charset::Blocks => &[' ', '\u{2591}', '\u{2592}', '\u{2593}', '\u{2588}'],
            Charset::Braille => &[' ', '\u{2810}', '\u{2812}', '\u{283F}', '\u{28FF}'],
        }
    }
}

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    path: String,

    #[arg(short, long, value_enum, default_value_t = ColorMode::Grayscale)]
    mode: ColorMode,

    #[arg(short, long, value_enum, default_value_t = Charset::Simple)]
    charset: Charset,

    #[arg(short, long, default_value_t = 0.3)]
    resolution: f32,
}

fn brightness_to_char(brightness: u8, charset: &Charset) -> char {
    let chars = charset.get_charset();
    chars[(brightness as f32 / 255. * (chars.len() - 1) as f32) as usize]
}

fn rgb_to_brightness(r: u8, g: u8, b: u8) -> u8 {
    (r as f32 * 0.299 + g as f32 * 0.587 + b as f32 * 0.114) as u8
}

fn rgb_to_ascii_color(r: u8, g: u8, b: u8) -> String {
    format!("\x1b[38;2;{};{};{}m", r, g, b)
}

fn normalize_pixels(img: ImageBuffer<Luma<u8>, Vec<u8>>) -> ImageBuffer<Luma<u8>, Vec<u8>> where
{
    let max_val = img.pixels().map(|p| p[0]).max().unwrap_or(1) as f32;

    if max_val == 0. {
        return img.clone();
    }

    ImageBuffer::from_fn(img.width(), img.height(), |x, y| {
        Luma([(img.get_pixel(x, y).0[0] as f32 / max_val * 255.) as u8])
    })
}

fn process_image_grayscale(
    path: &str,
    resolution: f32,
    charset: Charset,
) -> Result<String, Box<dyn std::error::Error>> {
    let img = ImageReader::open(path)?.decode()?.to_luma8();
    let (mut new_w, mut new_h) = img.dimensions();
    let ratio = 2.;
    new_w = (new_w as f32 * resolution) as u32;
    new_h = (new_h as f32 * resolution / ratio) as u32;
    let resized_image = imageops::resize(&img, new_w, new_h, imageops::FilterType::Triangle);
    let normalized_image = normalize_pixels(resized_image);
    let mut output = String::with_capacity((new_w * new_h + new_h) as usize);
    for y in 0..new_h {
        for x in 0..new_w {
            let brightness = normalized_image.get_pixel(x, y).0[0];
            output.push(brightness_to_char(brightness, &charset));
        }
        output.push('\n');
    }

    Ok(output)
}

fn process_image_rgb(
    path: &str,
    resolution: f32,
    charset: Charset,
) -> Result<String, Box<dyn std::error::Error>> {
    let img = ImageReader::open(path)?.decode()?.to_rgb8();
    let (mut new_w, mut new_h) = img.dimensions();
    let ratio = 2.;
    new_w = (new_w as f32 * resolution) as u32;
    new_h = (new_h as f32 * resolution / ratio) as u32;
    let resized_image = imageops::resize(&img, new_w, new_h, imageops::FilterType::Triangle);

    let max_brightness = resized_image
        .pixels()
        .map(|p| rgb_to_brightness(p[0], p[1], p[2]))
        .max()
        .unwrap_or(1) as f32;

    let mut output = String::with_capacity((new_h * new_w * 20 + new_h) as usize);
    for y in 0..new_h {
        for x in 0..new_w {
            let [r, g, b] = resized_image.get_pixel(x, y).0;
            output.push_str(&rgb_to_ascii_color(r, g, b));
            output.push(brightness_to_char(
                (rgb_to_brightness(r, g, b) as f32 / max_brightness * 255.) as u8,
                &charset,
            ));
        }
        output.push('\n');
    }

    Ok(output)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let f = match args.mode {
        ColorMode::RGB => process_image_rgb,
        ColorMode::Grayscale => process_image_grayscale,
    };
    let output = f(&args.path, args.resolution, args.charset)?;
    println!("{}", output);

    Ok(())
}
