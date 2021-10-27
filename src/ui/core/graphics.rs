use uefi::proto::console::gop::GraphicsOutput;
use uefi::ResultExt;

use no_std_compat::prelude::v1::vec;

use crate::bmp::{Bitmap, Pixel};
use crate::ui::core::font;

pub enum ColorType {
    Foreground = 0,
    Background,
    Cursor,
    BorderUnfocused,
    BorderFocused,
}

pub type Color = [u8; 3];
pub struct ColorScheme([Color; 5]);

impl ColorScheme {
    pub fn new(
        foreground: Color,
        background: Color,
        cursor: Color,
        border_unfocused: Color,
        border_focused: Color,
    ) -> ColorScheme {
        ColorScheme([
            foreground,
            background,
            cursor,
            border_unfocused,
            border_focused,
        ])
    }

    pub fn get(&self, color: ColorType) -> Color {
        let idx: usize = color as usize;
        assert!(idx < self.0.len());
        return self.0[idx];
    }
}

/// Enum names are as in HTML: <h1>, <h2>, <p>
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum FontSize {
    H1 = 0,
    H2,
    P,
}

pub struct FontSizes([usize; 3]);

impl FontSizes {
    #[allow(dead_code)]
    pub fn default() -> FontSizes {
        FontSizes([2, 2, 2])
    }

    pub fn new(h1: usize, h2: usize, text: usize) -> FontSizes {
        FontSizes([h1, h2, text])
    }

    pub fn get(&self, size: FontSize) -> usize {
        let idx: usize = size as usize;
        assert!(idx < self.0.len());
        return self.0[idx];
    }
}

// maybe later i want to add multiple fonts lol
pub struct Theme {
    pub font_sizes: FontSizes,
    pub color_scheme: ColorScheme,
}

/// `Graphics` is passed around to `Widget`s rather than uefi's `GraphicsOutput`
/// because it can hold onto custom state (currently theme and font size).
///
/// Exposes methods for setting the session's resolution and drawing various things.
pub struct Graphics<'a> {
    gop: &'a mut GraphicsOutput<'a>,
    pub theme: Theme,
}

impl<'a> Graphics<'a> {
    pub fn new(gop: &'a mut GraphicsOutput<'a>, theme: Theme) -> Graphics<'a> {
        Graphics {
            gop: gop,
            theme: theme,
        }
    }

    /// Write a char `c` to the pixel `top_left` scaled by `size` and in `color`.
    /// `color` is a ColorType; when creating an `Application` the user provides
    /// a theme which determines what color will actually be used.
    pub fn write_char(
        &mut self,
        c: char,
        top_left: (usize, usize),
        size: FontSize,
        color: ColorType,
    ) {
        let color = self.theme.color_scheme.get(color);
        let size = self.theme.font_sizes.get(size);
        let bmp = font::get_bitmap(c, color);
        let mut bmp = Bitmap::scale(&bmp, size);
        bmp.draw(self.gop, top_left);
    }

    /// Just draws a little guy
    /// `color` is a ColorType; when creating an `Application` the user provides
    /// a theme which determines what color will actually be used.
    pub fn draw_rect(
        &mut self,
        color: ColorType,
        top_left: (usize, usize),
        dimensions_px: (usize, usize),
        border: Option<ColorType>,
    ) {
        let color = self.theme.color_scheme.get(color);
        let border = border.map(|x| self.theme.color_scheme.get(x));

        let cols = dimensions_px.0;
        let rows = dimensions_px.1;
        let mut bmp = Bitmap::new(
            rows,
            cols,
            Pixel::from_u8_vec(vec![1; cols * rows], color),
            border,
        );
        bmp.draw(self.gop, top_left);
    }

    /// Set the resolution if the specified value is among the list of available
    /// modes. Just yell into the console if it doesn't work.
    pub fn set_resolution(&mut self, resolution: (usize, usize)) {
        let mode = self
            .gop
            .modes()
            .map(|mode| mode.expect("failed to get mode"))
            .find(|mode| mode.info().resolution() == resolution);
        if let Some(value) = mode {
            self.gop
                .set_mode(&value)
                .expect_success("failed to set mode");
        } else {
            log::info!("resolution not found: {}x{}", resolution.0, resolution.1);
        }
    }
}
