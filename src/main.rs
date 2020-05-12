extern crate image;

use std::error::Error;
use image::imageops::FilterType;
use image::{DynamicImage, ImageEncoder, GenericImageView};
use image::jpeg;
use std::fs;
use clap::Clap;

#[derive(Clap)]
#[clap(version = "0.1", author = "Raphael Peters <raphael.r.peters@gmail.com>")]
struct Opts {
    input: String,
    output: String,
    #[clap(short, long, default_value = "200")]
    size: u32,
    #[clap(short, long)]
    quality: Option<u8>,
    #[clap(short, long)]
    circle: bool,
    #[clap(short, long)]
    max_filesize: Option<usize>,
    #[clap(short, long)]
    grayscale: bool,
}

fn in_circle(x: i64, y: i64, diameter: i64) -> bool {
    let radius = diameter/2;
    let dx = radius - x;
    let dy = radius - y;
    dx*dx + dy*dy < radius * radius
}

fn block_in_circle(x: u32, y: u32, diameter: u32, blocksize: u32) -> bool {
    let (x,y,diameter,blocksize) = (x as i64, y as i64, diameter as i64, blocksize as i64);
    in_circle(x - x%blocksize - 1, y - y%blocksize - 1, diameter) ||
    in_circle(x - x%blocksize + blocksize, y - y%blocksize - 1, diameter) ||
    in_circle(x - x%blocksize - 1, y - y%blocksize + blocksize, diameter) ||
    in_circle(x - x%blocksize + blocksize, y - y%blocksize + blocksize, diameter)
}

fn to_circle(img: DynamicImage, size: u32) -> DynamicImage {
    let resized = img.resize(size, size, FilterType::Lanczos3).to_rgb();
    let mut out = image::RgbImage::new(size, size);
    for (x, y, pixel) in resized.enumerate_pixels() {

        if block_in_circle(x, y, size, 8) {
            out.put_pixel(x, y, *pixel);
        }
    }
    DynamicImage::ImageRgb8(out)
}

fn process_image(img: &DynamicImage, circle: bool, grayscale: bool, size: u32, quality: u8) -> Vec<u8> {
    let mut resized = if circle {
        to_circle(img.clone().into(), size)
    } else {
        img.resize(size, size, FilterType::Lanczos3)
    };

    if grayscale {
        resized = DynamicImage::ImageLuma8(resized.to_luma());
    }

    let mut buffer = Vec::new();
    let encoder = jpeg::JPEGEncoder::new_with_quality(&mut buffer, quality);
    encoder.write_image(
        &resized.to_bytes(),
        resized.width(),
        resized.height(),
        resized.color()).unwrap();
    println!("Quality: {:3}, File size: {:6}", quality, buffer.len());
    buffer
}

fn main() -> Result<(), Box<dyn Error>>{
    let opts: Opts = Opts::parse();

    let img = image::open(opts.input)?;

    let mut buffer = Vec::new();

    if let Some(max_filesize) = opts.max_filesize {
        let mut quality: i32 = 100;
        // not small enough or first iteration?
        while (buffer.len() > max_filesize || buffer.is_empty()) && quality >= 0 {
            buffer = process_image(
                &img,
                opts.circle,
                opts.grayscale,
                opts.size,
                opts.quality.unwrap_or(quality as u8)
            );
            quality = quality - 10;
        }
    } else {
        buffer = process_image(
            &img,
            opts.circle,
            opts.grayscale,
            opts.size,
            opts.quality.unwrap_or(75)
        );
    }

    fs::write(opts.output, &buffer)?;

    Ok(())
}
