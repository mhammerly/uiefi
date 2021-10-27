use uefi::proto::console::gop::GraphicsOutput;

use no_std_compat::prelude::v1::vec;
use no_std_compat::prelude::v1::Box;
use no_std_compat::vec::Vec;

/// if true, all bitmaps have a white border
pub static mut DEBUG_BORDER: bool = false;

#[derive(Clone, Copy)]
/// Pixel in a Bitmap. If `draw` is false, it's skipped. Else `color` is drawn.
pub struct Pixel {
    draw: bool,
    color: [u8; 3],
}

impl Pixel {
    /// Simple helper for getting empty `Pixel`s to initialize new `Bitmap`s.
    pub const fn empty() -> Pixel {
        Pixel {
            draw: false,
            color: [0, 0, 0],
        }
    }

    /// Create a new `Pixel`
    pub const fn new(draw: bool, color: [u8; 3]) -> Pixel {
        Pixel {
            draw: draw,
            color: color,
        }
    }

    /// Helper to take a `Vec<u8>` and turn it into something `Bitmap` can take.
    /// It's easier to create things like bitmap fonts as a `Vec<u8>` than as
    /// `Pixel`s.
    pub fn from_u8_vec(v: Vec<u8>, color: [u8; 3]) -> Box<[Pixel]> {
        let new_v: Vec<Pixel> = v
            .iter()
            .map(|p| {
                Pixel::new(
                    match p {
                        1 => true,
                        _ => false,
                    },
                    color,
                )
            })
            .collect();
        return new_v.into_boxed_slice();
    }
}

/// A box of pixels that knows how to draw itself.
/// rows: pixels in the `Bitmap` (y)
/// cols: pixels in the `Bitmap` (x)
/// bmp: the actual data of the `Bitmap`, as a flat array
pub struct Bitmap {
    rows: usize,
    cols: usize,
    bmp: Box<[Pixel]>,
    border: Option<[u8; 3]>,
}

impl Bitmap {
    pub const fn new(
        rows: usize,
        cols: usize,
        bmp: Box<[Pixel]>,
        border: Option<[u8; 3]>,
    ) -> Bitmap {
        Bitmap {
            rows: rows,
            cols: cols,
            bmp: bmp,
            border: border,
        }
    }

    /// If you have a `Bitmap` but need to scale it up, here's your helper.
    /// Useful for setting font size of a bitmap font.
    pub fn scale(other: &Bitmap, factor: usize) -> Bitmap {
        let new_rows = other.rows * factor;
        let new_cols = other.cols * factor;
        let mut scaled: Vec<Pixel> = vec![Pixel::empty(); new_rows * new_cols];
        for y in 0..new_rows {
            for x in 0..new_cols {
                let new_val = other.bmp[(y / factor * other.cols) + (x / factor)];
                scaled[(y * new_cols) + x] = new_val;
            }
        }
        return Bitmap::new(new_rows, new_cols, scaled.into_boxed_slice(), None);
    }

    /// Get the framebuffer out of the UEFI `GraphicsOutput` and write the
    /// `Bitmap` to it.
    /// start: (x, y) coordinates (in px) of the top-left corner of the `Bitmap`
    pub fn draw(&mut self, gop: &mut GraphicsOutput, start: (usize, usize)) {
        unsafe {
            // accessing a mutable static
            if self.border.is_none() && DEBUG_BORDER == true {
                self.border = Some([255, 255, 255]);
            }
        }
        if let Some(value) = self.border {
            self.set_border(value);
        }
        let stride = gop.current_mode_info().stride();
        let mut fb = gop.frame_buffer();
        for y in 0..self.rows {
            for x in 0..self.cols {
                let px = &self.bmp[(self.cols * y) + x];
                if px.draw == true {
                    // stride is pixels per scanline
                    // y * stride == row of pixels, add x for column
                    let idx = ((y + start.1) * stride) + (x + start.0);

                    // resolution.x * resolution.y * 4 == fb.size()
                    // i don't think we get an alpha byte (rgba) so it must be an alignment thing
                    let pixel_addr = 4 * idx;
                    unsafe {
                        fb.write_value(pixel_addr, px.color);
                    }
                }
            }
        }
    }

    /// Overwrite the color on perimeter pixels to make a border.
    pub fn set_border(&mut self, color: [u8; 3]) {
        let last_row_offset = self.cols * (self.rows - 1);
        for i in 0..self.cols {
            self.bmp[i].draw = true;
            self.bmp[i].color = color;

            self.bmp[i + last_row_offset].draw = true;
            self.bmp[i + last_row_offset].color = color;
        }

        for i in 0..self.rows {
            self.bmp[i * self.cols].draw = true;
            self.bmp[i * self.cols].color = color;

            self.bmp[i * self.cols + self.cols - 1].draw = true;
            self.bmp[i * self.cols + self.cols - 1].color = color;
        }
    }
}
