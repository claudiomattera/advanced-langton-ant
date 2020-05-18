// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use std::error;
use std::fmt;

use palette::{Gradient, LinSrgba, Pixel, Srgba};

#[derive(Debug)]
pub struct Palette {
    gradient: Gradient<LinSrgba<f64>>,
    length: f64,
}

impl Palette {
    pub fn new(name: &str, count: usize) -> Result<Self, PaletteParseError> {
        let base_palette = get_base_palette(name)?;
        let linear_palette = base_palette.iter().map(
            |[r, g, b, a]| Srgba::new(
                *r as f64 / 255.0,
                *g as f64 / 255.0,
                *b as f64 / 255.0,
                *a as f64 / 255.0
            ).into_linear()
        );
        let gradient = Gradient::new(linear_palette);
        let length = count as f64;
        Ok(Palette { gradient, length })
    }

    pub fn get_color(self: &Self, index: usize) -> [u8; 4] {
        let value = index as f64 / (self.length - 1.0);
        let color = self.gradient.get(value);
        let pixel: [u8; 4] = Srgba::from_linear(color)
            .into_format()
            .into_raw();
        pixel
    }
}

fn get_base_palette(name: &str) -> Result<&[[u8; 4]], PaletteParseError> {
    match name {
        "blue" => Ok(PALETTE_BLUES),
        "red" => Ok(PALETTE_REDS),
        _ => Err(PaletteParseError),
    }
}

// Palette from https://colorbrewer2.org/
const PALETTE_REDS: &[[u8; 4]] = &[
    [255, 245, 240, 255],
    [254, 224, 210, 255],
    [252, 187, 161, 255],
    [252, 146, 114, 255],
    [251, 106, 74, 255],
    [239, 59, 44, 255],
    [203, 24, 29, 255],
    [165, 15, 21, 255],
    [103, 0, 13, 255],
];

// Palette from https://colorbrewer2.org/
const PALETTE_BLUES: &[[u8; 4]] = &[
    [247, 251, 255, 255],
    [222, 235, 247, 255],
    [198, 219, 239, 255],
    [158, 202, 225, 255],
    [107, 174, 214, 255],
    [66, 146, 198, 255],
    [33, 113, 181, 255],
    [8, 81, 156, 255],
    [8, 48, 107, 255],
];



#[derive(Debug, Clone)]
pub struct PaletteParseError;

impl fmt::Display for PaletteParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid palette name")
    }
}

impl error::Error for PaletteParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_of_zero_is_first_color() {
        let palette = Palette::new("blue", 5).expect("Palette does not exist");
        let color = palette.get_color(0);
        assert_eq!(color, [247, 251, 255, 255]);
    }

    #[test]
    fn color_of_n_is_last_color() {
        let palette = Palette::new("blue", 5).expect("Palette does not exist");
        let color = palette.get_color(4);
        assert_eq!(color, [8, 48, 107, 255]);
    }

    #[test]
    fn color_of_i_is_ith_color() {
        let palette = Palette::new("blue", 9).expect("Palette does not exist");
        let color = palette.get_color(4);
        assert_eq!(color, [107, 174, 214, 255]);
    }
}
