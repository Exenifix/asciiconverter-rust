use clap::{Parser, ValueEnum};

#[derive(Clone, ValueEnum)]
pub enum ColorMode {
    RGB,
    Grayscale,
}

#[derive(Clone, ValueEnum)]
pub enum Charset {
    Simple,
    Detailed,
    Braille,
    Blocks,
}

impl Charset {
    pub fn get_charset(&self) -> &'static [char] {
        match self {
            Charset::Simple => &[' ', '.', '~', ':', '*', '#', '@'],
            Charset::Detailed => &[' ', '.', '^', '~', ':', '*', '=', '+', '!', '?', '#', '@'],
            Charset::Blocks => &[' ', '\u{2591}', '\u{2592}', '\u{2593}', '\u{2588}'],
            Charset::Braille => &[' ', '\u{2810}', '\u{2812}', '\u{283F}', '\u{28FF}'],
        }
    }
}

#[derive(Parser)]
pub struct Args {
    #[arg(short, long)]
    pub path: String,

    #[arg(short, long, value_enum, default_value_t = ColorMode::Grayscale)]
    pub mode: ColorMode,

    #[arg(short, long, value_enum, default_value_t = Charset::Simple)]
    pub charset: Charset,

    #[arg(short, long, default_value_t = 0.3)]
    pub resolution: f32,

    #[arg(short, long, default_value_t = 5)]
    pub framerate: u16,
}
