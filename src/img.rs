use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, Pixel, Rgba};
use itertools::Itertools;
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum Direction {
    Horizontal,
    Vertical,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum Order {
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

pub fn sort_img(img: DynamicImage, settings: SortSettings) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let (w, h) = img.dimensions();
    let mut output = ImageBuffer::new(w, h);

    match settings.direction {
        Direction::Horizontal => {
            for y in 0..h {
                let mut span_start: u32 = 0;
                for x in 0..w {
                    let pixel = img.get_pixel(x, y);
                    output.put_pixel(x, y, pixel); // todo: don't put pixel if we're sorting it
                    let luminance = pixel.to_luma().0[0];
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
                    output.put_pixel(x, y, pixel); // todo: don't put pixel if we're sorting it
                    let luminance = pixel.to_luma().0[0];
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
