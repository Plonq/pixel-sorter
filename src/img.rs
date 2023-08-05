use image::{DynamicImage, GenericImageView, ImageBuffer, Pixel, Rgba};
use itertools::Itertools;

pub fn sort_img(img: DynamicImage) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    // let img = img.to_rgba8();
    let (w, h) = img.dimensions();
    let mut output = ImageBuffer::new(w, h);

    let mut rows: Vec<Vec<Rgba<u8>>> = vec![];

    for row in &img.pixels().chunks(w as usize) {
        rows.push(row.map(|(_, _, val)| val).collect())
    }

    // For each row, find all spans based on luminance mask, then sort each span individually
    let (min_lum, max_lum) = (50, 200);

    // let mut new_rows = vec![];
    for row in rows.iter_mut() {
        let mut span_start: usize = 0;
        // let mut new_row: Vec<Rgba<u8>> = vec![];
        for (i, pixel) in row.clone().iter().enumerate() {
            let luminance = pixel.to_luma().0[0];
            // test mask
            // if luminance < min_lum || luminance > max_lum {
            //     new_row.push(Rgba::from([0, 0, 0, 255]));
            // } else {
            //     new_row.push(Rgba::from([255, 255, 255, 255]));
            // }
            if luminance < min_lum || luminance > max_lum {
                if (span_start as isize) < (i as isize) - 1 {
                    // Sort the span
                    let mut span = row[span_start..i].to_vec();
                    span.sort_unstable_by_key(|p| {
                        p.0[0] as u32 + p.0[1] as u32 + p.0[2] as u32 + p.0[3] as u32
                    });
                    // span.reverse();
                    row.splice(span_start..i, span);
                }
                span_start = i;
            }
        }
    }

    // Write sorted pixels to output
    for (y, row) in rows.iter().enumerate() {
        for (x, pixel) in row.iter().enumerate() {
            output.put_pixel(x as u32, y as u32, *pixel);
        }
    }

    output
}
