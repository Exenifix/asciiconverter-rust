use ffmpeg_next::format::{input, Pixel};
use ffmpeg_next::media::Type;
use ffmpeg_next::software::scaling::{context::Context, flag::Flags};
use ffmpeg_next::util::frame::Video;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

pub struct VideoProcessor {
    args: super::cli_manager::Args,
}

impl VideoProcessor {
    pub fn new(args: super::cli_manager::Args) -> Self {
        Self { args }
    }

    pub fn process(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let result = match self.args.mode {
            super::cli_manager::ColorMode::RGB => {
                let frames = extract_frames_rgb(&self.args.path, self.args.framerate)?;
                self.process_frames_rgb_parallel(frames.as_slice())
            }
            super::cli_manager::ColorMode::Grayscale => {
                let frames = extract_frames_grayscale(&self.args.path, self.args.framerate)?;
                self.process_frames_grayscale_parallel(frames.as_slice())
            }
        };

        Ok(result)
    }

    fn process_frames_rgb_parallel(&self, frames: &[image::RgbImage]) -> Vec<String> {
        let total_frames = frames.len();
        let pb = ProgressBar::new(total_frames as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                .expect("Template definition error")
                .progress_chars("#>-"),
        );

        let ascii_frames = frames
            .par_iter()
            .enumerate()
            .map(|(_, frame)| {
                let ascii = super::image_processing::process_image_rgb(
                    frame,
                    self.args.resolution,
                    &self.args.charset,
                );
                pb.inc(1);
                ascii
            })
            .collect::<Vec<String>>();

        ascii_frames
    }

    fn process_frames_grayscale_parallel(&self, frames: &[image::GrayImage]) -> Vec<String> {
        let total_frames = frames.len();
        let pb = ProgressBar::new(total_frames as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                .expect("Template definition error")
                .progress_chars("#>-"),
        );

        let ascii_frames = frames
            .par_iter()
            .enumerate()
            .map(|(_, frame)| {
                let ascii = super::image_processing::process_image_grayscale(
                    frame,
                    self.args.resolution,
                    &self.args.charset,
                );
                pb.inc(1);
                ascii
            })
            .collect::<Vec<String>>();

        ascii_frames
    }
}

pub struct VideoPlayer {
    data: Vec<String>, // not a ref specially
    framerate: u16,
}

impl VideoPlayer {
    pub fn new(data: Vec<String>, framerate: u16) -> Self {
        Self { data, framerate }
    }

    pub fn play(&self) {
        let mut last_rendered_frame = 0u32;
        let mut current_frame: u32;
        let mut sequence_start = std::time::Instant::now();
        let frame_duration_millis = 1000. / self.framerate as f32;
        loop {
            current_frame =
                (sequence_start.elapsed().as_millis() as f32 / frame_duration_millis) as u32;
            if current_frame >= self.data.len() as u32 {
                sequence_start = std::time::Instant::now();
                continue;
            }

            if current_frame == last_rendered_frame {
                continue;
            }
            last_rendered_frame = current_frame;
            print!("{}{}", "\x1B[2J\x1B[1;1H", "\n".repeat(3)); // clear screen
            println!("{}", self.data[current_frame as usize]);
        }
    }
}

fn extract_frames(path: &str, target_fps: u16) -> Result<Vec<Video>, Box<dyn std::error::Error>> {
    println!("Initializing ffmpeg...");
    ffmpeg_next::init()?;

    println!("Extracting frames...");
    let mut frames = Vec::new();
    if let Ok(mut ictx) = input(path) {
        let data = ictx
            .streams()
            .best(Type::Video)
            .ok_or(ffmpeg_next::Error::StreamNotFound)?;
        let vsi = data.index();
        let context = ffmpeg_next::codec::context::Context::from_parameters(data.parameters())?;
        let mut decoder = context.decoder().video()?;

        let mut scaler = Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            Pixel::RGB24,
            decoder.width(),
            decoder.height(),
            Flags::BILINEAR,
        )?;

        let original_fps = data.avg_frame_rate();
        let fps_ratio = target_fps as f32 / (original_fps.0 as f32 / original_fps.1 as f32);
        let mut frame_count = 0f32;

        for (stream, packet) in ictx.packets() {
            if stream.index() == vsi {
                let mut decoded = Video::empty();
                if decoder.send_packet(&packet).is_ok() {
                    while decoder.receive_frame(&mut decoded).is_ok() {
                        frame_count += 1.;
                        if frame_count < fps_ratio {
                            continue;
                        }
                        frame_count = 0.;
                        let mut rgb_frame = Video::empty();
                        scaler
                            .run(&decoded, &mut rgb_frame)
                            .expect("Failed to scale");
                        frames.push(rgb_frame);
                    }
                }
            }
        }
    }

    Ok(frames)
}

fn extract_frames_rgb(
    path: &str,
    target_fps: u16,
) -> Result<Vec<image::RgbImage>, Box<dyn std::error::Error>> {
    let mut output = Vec::new();
    for frame in extract_frames(path, target_fps)? {
        output.push(frame_to_rgb(frame));
    }

    Ok(output)
}

fn extract_frames_grayscale(
    path: &str,
    target_fps: u16,
) -> Result<Vec<image::GrayImage>, Box<dyn std::error::Error>> {
    let mut output = Vec::new();
    for frame in extract_frames(path, target_fps)? {
        output.push(frame_to_grayscale(frame));
    }

    Ok(output)
}

fn frame_to_rgb(frame: Video) -> image::RgbImage {
    let (w, h) = (frame.width(), frame.height());
    let data = frame.data(0);
    let img = image::RgbImage::from_fn(w, h, |x, y| {
        let i = ((y * w + x) * 3) as usize;
        image::Rgb([data[i], data[i + 1], data[i + 2]])
    });

    img
}

fn frame_to_grayscale(frame: Video) -> image::GrayImage {
    let (w, h) = (frame.width(), frame.height());
    let data = frame.data(0);
    let img = image::GrayImage::from_fn(w, h, |x, y| {
        let i = (y * w + x) as usize;
        image::Luma([super::image_processing::rgb_to_brightness(
            data[i],
            data[i + 1],
            data[i + 2],
        )])
    });

    img
}
