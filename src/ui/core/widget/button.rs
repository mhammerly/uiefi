use uefi::proto::console::text::Key;

use no_std_compat::string::String;
use no_std_compat::vec::Vec;

use crate::ui::core::{font, graphics, UIResult};
use crate::widget::Widget;
use graphics::{ColorType, FontSize, Graphics};

/// Just a rectangle with some text in it. If you press enter while it's
/// focused it returns `UIResult::POST(id, data)` else `UIResult::OK`.
pub struct Button {
    id: String,
    subscriptions: Vec<String>,
    label: String,
    start_px: (usize, usize),
    dimensions_px: (usize, usize),
    font_size: FontSize,
}

impl Button {
    pub fn new(
        id: String,
        label: String,
        start_px: (usize, usize),
        dimensions_px: (usize, usize),
        font_size: FontSize,
    ) -> Button {
        Button {
            id: id,
            label: label,
            subscriptions: Vec::new(),
            start_px: start_px,
            dimensions_px: dimensions_px,
            font_size: font_size,
        }
    }
}

impl Widget for Button {
    fn id(&self) -> &String {
        return &self.id;
    }

    fn get_value(&self) -> String {
        return self.label.clone();
    }

    fn get_subscriptions(&self) -> &Vec<String> {
        return &self.subscriptions;
    }

    fn draw(&mut self, graphics: &mut Graphics, focused: bool) {
        let border = if focused {
            ColorType::BorderFocused
        } else {
            ColorType::BorderUnfocused
        };
        graphics.draw_rect(
            ColorType::Background,
            self.start_px,
            self.dimensions_px,
            Some(border),
        );

        let char_width = graphics.theme.font_sizes.get(self.font_size) * font::FONT_WIDTH;
        let char_height = graphics.theme.font_sizes.get(self.font_size) * font::FONT_HEIGHT;

        let x_offset =
            self.start_px.0 + ((self.dimensions_px.0 - (self.label.len() * char_width)) / 2);
        let y_offset = self.start_px.1 + ((self.dimensions_px.1 - char_height) / 2);
        let mut next_char_px = (x_offset, y_offset);
        for c in self.label.chars() {
            graphics.write_char(c, next_char_px, FontSize::P, ColorType::Foreground);
            next_char_px.0 += char_width;
        }
    }

    fn handle_key(&mut self, k: Key, _graphics: &mut Graphics) -> UIResult {
        if let Key::Printable(value) = k {
            let c = char::from(value);
            if '\n' == c || '\r' == c {
                return UIResult::POST(self.id.clone(), self.label.clone());
            }
        }
        return UIResult::OK;
    }

    fn dimensions(&mut self) -> (usize, usize) {
        return self.dimensions_px;
    }
}
