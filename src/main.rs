extern crate image;
extern crate base64;

use std::error::Error;
use image::imageops::FilterType;
use image::{DynamicImage, ImageEncoder, GenericImageView};
use image::jpeg;
use std::fs;
use std::io;
use clap::Clap;

#[derive(Clap)]
#[clap(version = "0.1", author = "Raphael Peters <raphael.r.peters@gmail.com>")]
struct Opts {
    #[clap(about = "Input file in PNG/JPEG/GIF/BMP/ICO/TIFF/.... (see https://docs.rs/image/0.23.4/image/ )")]
    input: String,
    #[clap(about = "Output file as JPEG")]
    output: String,
    #[clap(short, long, default_value = "200",
        about = "Dimensions of the image. The image will be created to a square.")]
    size: u32,
    #[clap(short, long, default_value = "75",
        about = "Quality of the JPEG image. Will be ignored if --max-filesize is set.")]
    quality: u8,
    #[clap(short, long,
        about = "Cut the contents to a circle without adding additional JPEG artifacts.")]
    circle: bool,
    #[clap(short, long,
        about = "Iterate the JPEG quality down until the filesize is smaller than the value.")]
    max_filesize: Option<usize>,
    #[clap(short, long,
        about = "Save the image as grayscale. This file should allways be set if the source data is grayscale.")]
    grayscale: bool,
    #[clap(short, long, default_value = "raw",
        about = "Output encoding format. Available formats are raw (JPEG), base64 and dataurl")]
    encoding: String,
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

fn process_image(img: &DynamicImage, circle: bool, grayscale: bool, size: u32) -> DynamicImage {
    let mut resized = if circle {
        to_circle(img.clone().into(), size)
    } else {
        img.resize(size, size, FilterType::Lanczos3)
    };

    if grayscale {
        resized = DynamicImage::ImageLuma8(resized.to_luma());
    }

    resized
}

fn encode_image(img: &DynamicImage, quality: u8) -> Vec<u8> {
    let mut buffer = Vec::new();
    let encoder = jpeg::JPEGEncoder::new_with_quality(&mut buffer, quality);
    encoder.write_image(
        &img.to_bytes(),
        img.width(),
        img.height(),
        img.color()).unwrap();
    println!("Quality: {:3}, File size: {:6}", quality, buffer.len());
    buffer
}

fn main() -> Result<(), Box<dyn Error>>{
    let opts: Opts = Opts::parse();

    let img = image::open(opts.input)?;

    let mut buffer = Vec::new();

    let processed = process_image(
        &img,
        opts.circle,
        opts.grayscale,
        opts.size
    );

    if let Some(max_filesize) = opts.max_filesize {
        let mut quality: i32 = 100;
        // not small enough or first iteration?
        while (buffer.len() > max_filesize || buffer.is_empty()) && quality >= 0 {
            buffer = encode_image(&processed, quality as u8);

            quality = quality - 5;
        }
    } else {
        buffer = encode_image(&processed, opts.quality);
    }

    let encoded = match opts.encoding.as_str() {
        "raw" | "jpeg" => buffer,
        "base64" => base64::encode(&buffer).into_bytes(),
        "dataurl" => format!("data:image/jpeg;base64,{}", base64::encode(&buffer)).into_bytes(),
        _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Unknown encoding formmat"))?
    };

    fs::write(opts.output, &encoded)?;

    Ok(())
}
