use clap::Parser;
use std::path::Path;

pub mod cli_manager;
mod image_processing;
mod video_processing;

const VIDEO_FORMATS: [&'static str; 4] = ["mp4", "mkv", "mov", "avi"];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli_manager::Args::parse();
    if VIDEO_FORMATS.contains(
        &Path::new(&args.path)
            .extension()
            .expect("Bad path")
            .to_str()
            .unwrap_or(""),
    ) {
        let framerate = args.framerate;
        let processor = video_processing::VideoProcessor::new(args);
        let output = processor.process()?;
        let player = video_processing::VideoPlayer::new(output, framerate);
        println!("Conversion done, press enter to play");
        std::io::stdin().read_line(&mut String::new())?;
        player.play();
    } else {
        let processor = image_processing::ImageProcessor::new(args);
        let output = processor.process()?;
        println!("{}", output);
    }

    Ok(())
}
