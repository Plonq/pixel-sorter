use std::io::{BufReader, BufWriter, Cursor, Write};

use image::{DynamicImage, GenericImageView, ImageBuffer, ImageFormat, Pixel, Rgba};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Clone, Default)]
pub enum Direction {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Default)]
pub enum Order {
    #[default]
    Ascending,
    Descending,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct SortSettings {
    pub lower_threshold: u8,
    pub upper_threshold: u8,
    pub direction: Direction,
    pub order: Order,
}

impl Default for SortSettings {
    fn default() -> Self {
        SortSettings {
            lower_threshold: 50,
            upper_threshold: 150,
            direction: Direction::Horizontal,
            order: Order::Ascending,
        }
    }
}

pub fn sort_img(img: DynamicImage, settings: SortSettings) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let (w, h) = img.dimensions();
    let mut output = ImageBuffer::new(w, h);

    match settings.direction {
        Direction::Horizontal => {
            for y in 0..h {
                let mut span_start: u32 = 0;
                for x in 0..w {
                    let pixel = img.get_pixel(x, y);
                    let luminance = pixel.to_luma().0[0];
                    output.put_pixel(x, y, pixel);
                    if luminance < settings.lower_threshold || luminance > settings.upper_threshold
                    {
                        if (span_start as isize) < (x as isize) - 1 {
                            // Sort the span
                            let mut span = (span_start..x)
                                .map(|x| img.get_pixel(x, y))
                                .collect::<Vec<_>>();
                            span.sort_unstable_by_key(|p| {
                                p.0[0] as u32 + p.0[1] as u32 + p.0[2] as u32 + p.0[3] as u32
                            });
                            if settings.order == Order::Descending {
                                span.reverse();
                            }
                            for i in span_start..x {
                                if let Some(pixel) = span.get((i - span_start) as usize) {
                                    output.put_pixel(i, y, *pixel);
                                }
                            }
                        }
                        span_start = x;
                    }
                }
            }
        }
        Direction::Vertical => {
            for x in 0..w {
                let mut span_start: u32 = 0;
                for y in 0..h {
                    let pixel = img.get_pixel(x, y);
                    let luminance = pixel.to_luma().0[0];
                    output.put_pixel(x, y, pixel);
                    if luminance < settings.lower_threshold || luminance > settings.upper_threshold
                    {
                        if (span_start as isize) < (y as isize) - 1 {
                            // Sort the span
                            let mut span = (span_start..y)
                                .map(|y| img.get_pixel(x, y))
                                .collect::<Vec<_>>();
                            span.sort_unstable_by_key(|p| {
                                p.0[0] as u32 + p.0[1] as u32 + p.0[2] as u32 + p.0[3] as u32
                            });
                            if settings.order == Order::Descending {
                                span.reverse();
                            }
                            for i in span_start..y {
                                if let Some(pixel) = span.get((i - span_start) as usize) {
                                    output.put_pixel(x, i, *pixel);
                                }
                            }
                        }
                        span_start = y;
                    }
                }
            }
        }
    }

    output
}

pub trait ImageToBytes {
    fn to_bytes(&self, format: ImageFormat) -> Vec<u8>;
}

impl ImageToBytes for ImageBuffer<Rgba<u8>, Vec<u8>> {
    fn to_bytes(&self, format: ImageFormat) -> Vec<u8> {
        let mut buf: BufWriter<Cursor<Vec<u8>>> = BufWriter::new(Cursor::new(vec![]));
        self.write_to(&mut buf, format).unwrap();
        buf.flush().unwrap();
        buf.get_ref().to_owned().into_inner()
    }
}

pub fn get_orientation(img_data: &Vec<u8>) -> Option<u32> {
    let cursor = Cursor::new(img_data);
    let mut file_reader = BufReader::new(cursor);
    let exifreader = exif::Reader::new();
    if let Ok(exif_data) = exifreader.read_from_container(&mut file_reader) {
        match exif_data.get_field(exif::Tag::Orientation, exif::In::PRIMARY) {
            Some(orientation) => match orientation.value.get_uint(0) {
                Some(v @ 1..=8) => Some(v),
                _ => None,
            },
            None => None,
        }
    } else {
        None
    }
}
