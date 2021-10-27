use uefi::proto::console::text::{Key, ScanCode};

use no_std_compat::cmp;
use no_std_compat::prelude::v1::vec;
use no_std_compat::string::String;
use no_std_compat::string::ToString;
use no_std_compat::vec::Vec;

use crate::ui::core::{font, graphics, UIResult};
use crate::widget::Widget;
use graphics::{ColorType, FontSize, Graphics};

/// How many characters of context surronding the cursor to show. Always show
/// `SCROLLOFF` lines above/below the cursor and `SCROLLOFF` columns to the
/// left/right of the cursor.
const SCROLLOFF: u8 = 3;

/// How thick in px the cursor should be.
const CURSOR_WEIGHT: u8 = 3;

/// Whether or not this TextArea should wrap lines that are wider than the
/// viewport.
#[allow(dead_code)]
pub enum XOverflowBehavior {
    Wrap,
    Scroll,
}

/// Primitive `Widget` to display and optionally edit text. Supports wrapping
/// text when lines exceed the available space, or scrolling. A building block
/// for more complex `Widget`s
pub struct TextArea {
    id: String,
    subscriptions: Vec<String>,
    content: Vec<String>,
    edit: bool,
    start: (usize, usize),
    dimensions_px: (usize, usize),
    font_size: FontSize,
    x_overflow: XOverflowBehavior,

    // computed
    cursor: (usize, usize),
    viewport_start: (usize, usize),

    // need `Graphics` to init. does Rust anything like C++'s std::call_once()?
    char_dims_set: bool,
    char_width: usize,
    char_height: usize,
    dimensions_chars: (usize, usize),
}

impl TextArea {
    /// Create a new `TextArea`.
    /// id: the name by which a more complex `Widget` can fetch this one
    /// subscriptions: superfluous
    /// content: initial content to display in the `TextArea`.
    /// start: (x, y) coordinates of the top-left corner (in px)
    /// dimensions_px: (x, y) dimensions in px
    /// font_size: controls the size of the text displayed (thus how many chars fit on a line)
    /// x_overflow: whether to scroll or wrap text that is too wide to be displayed
    pub fn new(
        id: String,
        subscriptions: Vec<String>,
        content: String,
        edit: bool,
        start: (usize, usize),
        dimensions_px: (usize, usize),
        font_size: FontSize,
        x_overflow: XOverflowBehavior,
    ) -> TextArea {
        let content: Vec<String> = content
            .split(&['\n', '\r'][..])
            .map(|x| x.to_string())
            .collect();
        let cursor = (0, 0);
        let viewport_start = (0, 0);

        TextArea {
            id: id,
            subscriptions: subscriptions,
            content: content,
            edit: edit,
            start: start,
            dimensions_px: dimensions_px,
            font_size: font_size,
            x_overflow: x_overflow,
            cursor: cursor,
            viewport_start: viewport_start,
            char_dims_set: false,
            char_width: 0,
            char_height: 0,
            dimensions_chars: (0, 0),
        }
    }

    /// Compute where on the screen the cursor should be drawn based on:
    /// - cursor position
    /// - overflow behavior
    /// - viewport position
    /// Only draw the cursor in edit mode.
    fn maybe_draw_cursor(&self, graphics: &mut Graphics) {
        if self.edit {
            let pos_in_viewport = match self.x_overflow {
                XOverflowBehavior::Scroll => {
                    let col = self.cursor.0 - self.viewport_start.0;
                    let row = self.cursor.1 - self.viewport_start.1;
                    (col, row)
                }
                XOverflowBehavior::Wrap => {
                    let rows = self.dimensions_chars.1;
                    let start_row = self.viewport_start.1;
                    let max_end_row = cmp::min(self.content.len(), self.viewport_start.1 + rows);
                    let mut row_idx = start_row;
                    let mut pos = (0, 0);
                    for row in &self.content[start_row..max_end_row] {
                        let wrapped = self.wrap_line(row);
                        if row_idx != self.cursor.1 {
                            pos.1 += wrapped.len();
                        } else {
                            // figure out which line our cursor should be in. clamp to row length
                            // even though we're fine drawing the cursor one space further.
                            let cursor_region = wrapped
                                .iter()
                                .position(|(l, r)| l <= &self.cursor.0 && &self.cursor.0 <= r)
                                .expect("should have found it");
                            pos.1 += cursor_region;
                            pos.0 = self.cursor.0 - wrapped[cursor_region].0;
                            break;
                        }
                        row_idx += 1;
                    }
                    pos
                }
            };

            let cell_start = (
                self.start.0 + (pos_in_viewport.0 * self.char_width),
                self.start.1 + (pos_in_viewport.1 * self.char_height),
            );

            let top_left = (
                cell_start.0,
                cell_start.1 + self.char_height - usize::from(CURSOR_WEIGHT),
            );
            graphics.draw_rect(
                ColorType::Cursor,
                top_left,
                (self.char_width, usize::from(CURSOR_WEIGHT)),
                None,
            );
        }
    }

    /// To handle text that is too many lines to fit on screen (or too many
    /// columns with `XOverflowBehavior::WRAP`) `TextArea` has a viewport that
    /// gets dragged along as the cursor moves to show the right part of the
    /// text.
    fn drag_viewport(&mut self) {
        match self.x_overflow {
            XOverflowBehavior::Scroll => {
                let move_if_left_of = self.viewport_start.0 + usize::from(SCROLLOFF);
                let move_if_right_of =
                    self.viewport_start.0 + self.dimensions_chars.0 - usize::from(SCROLLOFF);
                if self.cursor.0 < move_if_left_of {
                    self.viewport_start.0 -=
                        cmp::min(self.viewport_start.0, move_if_left_of - self.cursor.0);
                } else if self.cursor.0 > move_if_right_of {
                    self.viewport_start.0 += self.cursor.0 - move_if_right_of;
                }

                let move_if_above = self.viewport_start.1 + usize::from(SCROLLOFF);
                let move_if_below =
                    self.viewport_start.1 + self.dimensions_chars.1 - usize::from(SCROLLOFF);
                if self.cursor.1 < move_if_above {
                    self.viewport_start.1 -=
                        cmp::min(self.viewport_start.1, move_if_above - self.cursor.1);
                } else if self.cursor.1 > move_if_below {
                    self.viewport_start.1 += self.cursor.1 - move_if_below;
                }
            }
            XOverflowBehavior::Wrap => {
                // viewport only moves up and down here but it's still kind of messy.
                // as-is, it won't support single lines that when wrapped are too tall.
                // also currently not bothering with scrolloff here, we'll see if it makes sense
                // in real applications and add it later.
                let wrapped_lines_per_line: Vec<usize> = self
                    .content
                    .iter()
                    .map(|line| self.wrap_line(line).len())
                    .collect();
                let lines_for_active_row = wrapped_lines_per_line[self.cursor.1];
                let move_if_above = self.viewport_start.1;

                if self.cursor.1 < move_if_above {
                    // need to move up to see what we can of the current row
                    self.viewport_start.1 = self.cursor.1;
                } else if self.dimensions_chars.1
                    - cmp::min(
                        wrapped_lines_per_line[self.viewport_start.1..self.cursor.1]
                            .iter()
                            .sum(),
                        self.dimensions_chars.1,
                    )
                    < lines_for_active_row
                {
                    // if the viewport doesn't have enough room for all of the current row's lines,
                    // move down. i'd like to extract the left side of this insane condition to a
                    // `lines_available` variable but computing it when moving the viewport up
                    // crashes.
                    let lines_backwards_from_cursor: Vec<&usize> = wrapped_lines_per_line
                        [0..self.cursor.1]
                        .iter()
                        .rev()
                        .collect();
                    let mut line_total = lines_for_active_row;
                    for i in 0..self.cursor.1 {
                        if line_total + lines_backwards_from_cursor[i] > self.dimensions_chars.1 {
                            self.viewport_start.1 = self.cursor.1 - i + 1;
                            break;
                        }
                        line_total += lines_backwards_from_cursor[i];
                    }
                }
            }
        }
    }

    /// When moving up or down a shorter line, we have to cap our cursor's
    /// horizontal permission at the length of the new line.
    fn snap_cursor(&mut self) {
        self.cursor.0 = cmp::min(self.cursor.0, self.content[self.cursor.1].len());
    }

    fn print_char(&self, graphics: &mut Graphics, c: char, pos: (usize, usize)) {
        let px = (
            self.start.0 + (pos.0 * self.char_width),
            self.start.1 + (pos.1 * self.char_height),
        );

        graphics.write_char(c, px, FontSize::P, ColorType::Foreground);
    }

    // split a string too long to fit into one line into several.
    // returns accumulated indices e.g. [(0, 7), (7, 15), (15, 22)]
    /// Helper method to split a single line of text into multiple lines
    /// according to the viewport dimensions. Returns Vec<(start, end)>, e.g.
    ///   [(0, 7), (7, 15), (15, 23), (23, 25), (25, 33)]
    /// If a line contains a word that is too long to fit on one line, the word
    /// gets broken up onto multiple lines.
    fn wrap_line(&self, s: &String) -> Vec<(usize, usize)> {
        let words: Vec<&str> = s.split_inclusive(&['\t', ' '][..]).collect();
        let cols = self.dimensions_chars.0;

        let mut wrapped: Vec<(usize, usize)> = vec![(0, 0)];
        wrapped = words.iter().fold(wrapped, |mut acc, word| {
            let last = acc.last_mut().expect("Shouldn't be empty");

            if word.len() > cols {
                let mut word_used = cols - (last.1 - last.0);
                last.1 += word_used;
                loop {
                    if word_used == word.len() {
                        break;
                    }
                    let last = acc.last().expect("Shouldn't be empty").clone();
                    let word_to_add = cmp::min(cols, word.len() - word_used);
                    acc.push((last.1, last.1 + word_to_add));
                    word_used += word_to_add;
                }
            } else if last.1 + word.len() - last.0 > cols {
                let last_clone = last.clone();
                acc.push((last_clone.1, last_clone.1 + word.len()));
            } else {
                last.1 += word.len();
            }
            acc
        });
        return wrapped;
    }

    /// Helper to write text with line wrapping enabled
    fn draw_with_x_wrapping(&mut self, graphics: &mut Graphics) {
        let rows = self.dimensions_chars.1;

        // only iterate through content within the viewport. we may not get
        // to max_end_row; each wrapped line will cut our view shorter.
        let start_row = self.viewport_start.1;
        let max_end_row = cmp::min(self.content.len(), self.viewport_start.1 + rows);

        let mut pos = (0, 0);
        for row in &self.content[start_row..max_end_row] {
            let wrapped = self.wrap_line(&row);
            for line in wrapped {
                if pos.1 >= self.dimensions_chars.1 {
                    return;
                }
                pos.0 = 0;
                for c in row[line.0..line.1].chars() {
                    self.print_char(graphics, c, pos);
                    pos.0 += 1;
                }
                pos.1 += 1;
            }
        }
    }

    /// Helper to write text with scrolling enabled
    fn draw_with_x_scrolling(&mut self, graphics: &mut Graphics) {
        let cols = self.dimensions_chars.0;
        let rows = self.dimensions_chars.1;

        let start_row = self.viewport_start.1;
        let end_row = cmp::min(self.content.len(), self.viewport_start.1 + rows);

        let mut pos = (0, 0);
        for row in &self.content[start_row..end_row] {
            let start_col = self.viewport_start.0;
            let end_col = cmp::min(row.len(), self.viewport_start.0 + cols);
            pos.0 = 0;

            if start_col < row.len() {
                for c in row[start_col..end_col].chars() {
                    self.print_char(graphics, c, pos);
                    pos.0 += 1;
                }
            }
            pos.1 += 1;
        }
    }
}

impl Widget for TextArea {
    fn id(&self) -> &String {
        return &self.id;
    }

    fn get_value(&self) -> String {
        return self.content.join("\n");
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
            self.start,
            self.dimensions_px,
            Some(border),
        );

        if !self.char_dims_set {
            self.char_width = graphics.theme.font_sizes.get(self.font_size) * font::FONT_WIDTH;
            self.char_height = graphics.theme.font_sizes.get(self.font_size) * font::FONT_HEIGHT;

            let cols = self.dimensions_px.0 / self.char_width;
            let rows = self.dimensions_px.1 / self.char_height;
            self.dimensions_chars = (cols, rows);
        }

        self.drag_viewport();
        self.maybe_draw_cursor(graphics);

        match &self.x_overflow {
            XOverflowBehavior::Wrap => {
                self.draw_with_x_wrapping(graphics);
            }
            XOverflowBehavior::Scroll => {
                self.draw_with_x_scrolling(graphics);
            }
        }
    }

    fn handle_key(&mut self, k: Key, graphics: &mut Graphics) -> UIResult {
        match k {
            Key::Printable(value) => match char::from(value) {
                '\x08' => {
                    // BACKSPACE
                    // if we aren't the first char, just delete in place
                    if self.cursor.0 > 0 {
                        self.content[self.cursor.1].remove(self.cursor.0 - 1);
                        self.cursor.0 -= 1;
                        self.draw(graphics, true /* focused */);
                    } else {
                        // we are the first char. are we the first row? if not, merge rows
                        // the viewport's x is expected to be 0 here
                        if self.cursor.1 > 0 {
                            let row_to_merge = self.content[self.cursor.1].clone();
                            let prev_row_len = self.content[self.cursor.1 - 1].len();

                            // merge rows
                            self.content[self.cursor.1 - 1] += &row_to_merge;
                            self.content.remove(self.cursor.1);

                            // move cursor
                            self.cursor.0 = prev_row_len;
                            self.cursor.1 -= 1;

                            self.draw(graphics, true /* focused */);
                        }
                    }
                }
                '\n' | '\r' => {
                    // split the current line into two, move the cursor
                    let (l, r) = self.content[self.cursor.1].split_at(self.cursor.0);
                    let (l, r) = (l.to_string(), r.to_string());
                    self.content.insert(self.cursor.1 + 1, r);
                    self.content[self.cursor.1] = l;
                    self.cursor.1 += 1;
                    self.cursor.0 = 0;
                    self.draw(graphics, true /* focused */);
                }
                _ => {
                    self.content[self.cursor.1].insert(self.cursor.0, char::from(value));
                    self.cursor.0 += 1;
                    self.draw(graphics, true /* focused */);
                }
            },
            Key::Special(value) => match value {
                ScanCode::LEFT => {
                    if self.cursor.0 > 0 {
                        self.cursor.0 -= 1;
                        self.draw(graphics, true /* focused */);
                    }
                }
                ScanCode::RIGHT => {
                    // allow cursor to go one past end of row for backspacing
                    if self.cursor.0 < self.content[self.cursor.1].len() {
                        self.cursor.0 += 1;
                        self.snap_cursor();
                        self.draw(graphics, true /* focused */);
                    }
                }
                ScanCode::UP => {
                    // if not already at the top
                    if self.cursor.1 > 0 {
                        self.cursor.1 -= 1;
                        self.snap_cursor();
                        self.draw(graphics, true /* focused */);
                    }
                }
                ScanCode::DOWN => {
                    // if not already at the bottom
                    if self.cursor.1 < self.content.len() - 1 {
                        self.cursor.1 += 1;
                        self.snap_cursor();
                        self.draw(graphics, true /* focused */);
                    }
                }
                ScanCode::DELETE => {
                    // if we aren't the last char, just delete in place
                    if self.cursor.0 < cmp::max(self.content[self.cursor.1].len(), 1) - 1 {
                        self.content[self.cursor.1].remove(self.cursor.0);
                        self.draw(graphics, true /* focused */);
                    } else {
                        // we are the last char. are we the last row? if not, merge rows
                        if self.cursor.1 < cmp::min(self.content.len(), self.dimensions_chars.1) - 1
                        {
                            let row_to_merge = self.content[self.cursor.1 + 1].clone();

                            // merge rows
                            self.content[self.cursor.1] += &row_to_merge;
                            self.content.remove(self.cursor.1 + 1);

                            self.draw(graphics, true /* focused */);
                        }
                    }
                }
                _ => {}
            },
        }
        return UIResult::OK;
    }

    fn dimensions(&mut self) -> (usize, usize) {
        return self.dimensions_px;
    }
}
