use log::info;

use uefi::proto::console::text::{Key, ScanCode};

use no_std_compat::prelude::v1::{vec, Box};
use no_std_compat::string::String;
use no_std_compat::vec::Vec;

use crate::graphics::{FontSize, Graphics};
use crate::ui::core::{font, UIResult};
use crate::widget::{Button, MultiWidget, Widget};

const BUTTON_PADDING: u8 = 3;

#[derive(PartialEq)]
pub enum MenuOrientation {
    HORIZONTAL,
    VERTICAL,
}

/// Lines a bunch of `Button`s up and navigates around with arrow keys.
/// Pressing enter while the `Menu` is focused will return a
/// `UIResult::POST(id, data)` with the label of the selected `Button`.
/// Currently can't handle any more buttons than fit on the screen at once
/// but it can stack buttons vertically for a sidebar/dropdown or horizontally
/// for a toolbar or tabs.
pub struct Menu {
    id: String,
    orientation: MenuOrientation,

    // computed
    multiwidget: MultiWidget,
    subscriptions: Vec<String>,
}

impl Menu {
    pub fn new(
        id: String,
        choices: Vec<String>,
        start_px: (usize, usize),
        dimensions_px: (usize, usize),
        orientation: MenuOrientation,
    ) -> Menu {
        let mut longest_choice: usize = 0;
        for choice in &choices {
            if choice.len() > longest_choice {
                longest_choice = choice.len();
            }
        }

        let char_width = 2 * font::FONT_WIDTH;
        let char_height = 2 * font::FONT_HEIGHT;

        let button_width = longest_choice * char_width + usize::from(BUTTON_PADDING);
        let button_height = char_height + usize::from(BUTTON_PADDING);
        let button_dimensions = (button_width, button_height);

        let mut button_start: (usize, usize);
        let button_x_step: usize;
        let button_y_step: usize;
        if orientation == MenuOrientation::VERTICAL {
            button_x_step = 0;
            let button_x_start = start_px.0 + ((dimensions_px.0 - button_width) / 2);

            let button_area_height = button_height * choices.len();
            if button_area_height >= dimensions_px.1 {
                // rely on scrolling and just stack buttons up
                // need to refactor to support scrolling but may as well leave this here
                button_start = (button_x_start, start_px.1);
                button_y_step = button_height;
            } else {
                let separation = (dimensions_px.1 - button_area_height) / (choices.len() + 1);
                button_start = (button_x_start, start_px.1 + separation);
                button_y_step = button_height + separation;
            }
        } else {
            button_y_step = 0;
            let button_y_start = start_px.1 + ((dimensions_px.1 - button_height) / 2);

            let button_area_width = button_width * choices.len();
            if button_area_width >= dimensions_px.0 {
                // rely on scrolling and just sit buttons next to each other
                // need to refactor to support scrolling but may as well leave this here
                button_start = (start_px.0, button_y_start);
                button_x_step = button_width;
            } else {
                let separation = (dimensions_px.0 - button_area_width) / (choices.len() + 1);
                button_start = (start_px.0 + separation, button_y_start);
                button_x_step = button_width + separation;
            }
        }

        let mut subscriptions: Vec<String> = vec![id.clone()];
        let mut buttons: Vec<Box<dyn Widget>> = Vec::new();
        for choice in &choices {
            let choice_id = id.clone() + choice + "_button";
            subscriptions.push(choice_id.clone());
            let button = Button::new(
                choice_id,
                choice.clone(),
                button_start,
                button_dimensions,
                FontSize::P,
            );
            buttons.push(Box::from(button));
            button_start.0 += button_x_step;
            button_start.1 += button_y_step;
        }

        let multiwidget_id = id.clone() + "_multiwidget";
        let multiwidget = MultiWidget::new(
            multiwidget_id.clone(),
            buttons,
            0,             /* focused */
            dimensions_px, /* dimensions */
        );

        Menu {
            id: id.clone(),
            orientation: orientation,
            multiwidget: multiwidget,
            subscriptions: subscriptions,
        }
    }
}

impl Widget for Menu {
    fn id(&self) -> &String {
        return &self.id;
    }

    fn get_value(&self) -> String {
        return self.multiwidget.get_value();
    }

    fn get_subscriptions(&self) -> &Vec<String> {
        return &self.subscriptions;
    }

    fn handle_post(&mut self, id: String, data: String) {
        info!("menu got post from {}: {}", id, data);
    }

    fn draw(&mut self, graphics: &mut Graphics, focused: bool) {
        self.multiwidget.draw(graphics, focused);
    }

    fn handle_key(&mut self, k: Key, graphics: &mut Graphics) -> UIResult {
        let mut result = UIResult::OK;
        match k {
            Key::Printable(value) => match char::from(value) {
                '\n' | '\r' => {
                    result = self.multiwidget.handle_key(k, graphics);
                }
                _ => {}
            },
            Key::Special(value) => match value {
                ScanCode::ESCAPE => {
                    return UIResult::CLOSE;
                }
                ScanCode::LEFT => {
                    if self.orientation == MenuOrientation::HORIZONTAL {
                        self.multiwidget.focus_prev(graphics);
                    }
                }
                ScanCode::RIGHT => {
                    if self.orientation == MenuOrientation::HORIZONTAL {
                        self.multiwidget.focus_next(graphics);
                    }
                }
                ScanCode::UP => {
                    if self.orientation == MenuOrientation::VERTICAL {
                        self.multiwidget.focus_prev(graphics);
                    }
                }
                ScanCode::DOWN => {
                    if self.orientation == MenuOrientation::VERTICAL {
                        self.multiwidget.focus_next(graphics);
                    }
                }
                _ => {}
            },
        }
        if let UIResult::POST(_id, data) = result {
            return UIResult::POST(self.id.clone(), data);
        }
        return UIResult::OK;
    }

    fn dimensions(&mut self) -> (usize, usize) {
        return self.multiwidget.dimensions();
    }
}
