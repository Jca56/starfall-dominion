//! The in-game pointer: Alva's prism, scaled big for the display.
use lntrn_image::Image;
use winit::window::{CustomCursor, CustomCursorSource};

const PRISM: &[u8] = include_bytes!("../assets/cursor.png");
/// Logical pixels tall. Big on purpose.
pub(crate) const HEIGHT: f64 = 64.0;

pub(crate) fn decode() -> Option<Image> {
    lntrn_image::decode(PRISM).ok()
}

/// The prism at `HEIGHT` logical pixels for a display at `scale`.
pub(crate) fn source(prism: &Image, scale: f64) -> Option<CustomCursorSource> {
    let height = (HEIGHT * scale).round().max(1.0) as u32;
    let width = (f64::from(prism.width) * f64::from(height) / f64::from(prism.height))
        .round()
        .max(1.0) as u32;
    let small = resize(prism, width, height);
    // The blade's tip sits in the top-left corner.
    CustomCursor::from_rgba(small.rgba, small.width as u16, small.height as u16, 1, 1).ok()
}

/// Area-average downscale. Colour is weighted by alpha so transparent pixels do
/// not bleed their hidden colour into the edges.
pub(crate) fn resize(image: &Image, width: u32, height: u32) -> Image {
    let (source_w, source_h) = (image.width as usize, image.height as usize);
    let span = |i: u32, out: u32, source: usize| {
        let start = i as usize * source / out as usize;
        let end = ((i as usize + 1) * source / out as usize).max(start + 1);
        start..end.min(source)
    };
    let mut out = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let (mut color, mut alpha, mut count) = ([0u64; 3], 0u64, 0u64);
            for sy in span(y, height, source_h) {
                for sx in span(x, width, source_w) {
                    let pixel = image.pixel(sx as u32, sy as u32);
                    let a = u64::from(pixel[3]);
                    for (sum, channel) in color.iter_mut().zip(pixel) {
                        *sum += u64::from(channel) * a;
                    }
                    alpha += a;
                    count += 1;
                }
            }
            for sum in color {
                out.push((sum + alpha / 2).checked_div(alpha).unwrap_or(0) as u8);
            }
            out.push(((alpha + count / 2) / count.max(1)) as u8);
        }
    }
    Image::new(width, height, out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_prism_decodes_and_scales_to_a_big_cursor() {
        let prism = decode().expect("the embedded PNG decodes");
        assert_eq!((prism.width, prism.height), (380, 512));
        assert!(source(&prism, 1.4).is_some());
        let small = resize(&prism, 71, 96);
        assert_eq!((small.width, small.height), (71, 96));
        assert!(
            small.rgba.iter().skip(3).step_by(4).any(|a| *a > 200),
            "Some of it is solid"
        );
        assert_eq!(small.pixel(70, 0)[3], 0, "The top-right corner stays clear");
    }

    #[test]
    fn downscale_weights_colour_by_alpha() {
        let mut rgba = vec![0; 16];
        rgba[..4].copy_from_slice(&[255, 0, 0, 255]);
        // Three fully transparent pixels hiding bright green.
        for pixel in rgba[4..].chunks_exact_mut(4) {
            pixel.copy_from_slice(&[0, 255, 0, 0]);
        }
        let small = resize(&Image::new(2, 2, rgba), 1, 1);
        assert_eq!(small.pixel(0, 0), [255, 0, 0, 64]);
    }
}
