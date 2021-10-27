use log::info;

use uefi::proto::console::text::{Key, ScanCode};

use no_std_compat::prelude::v1::{vec, Box};
use no_std_compat::string::String;
use no_std_compat::vec::Vec;

use crate::graphics::{FontSize, Graphics};
use crate::ui::components::menu::{Menu, MenuOrientation};
use crate::ui::core::UIResult;
use crate::widget::{MultiWidget, TextArea, Widget, XOverflowBehavior};

/// `Widget` combining a text input area (`TextArea`) with a set of buttons.
/// "Cancel" button just exits and "Save" button posts the data. Posting
/// should also quit, but it doesn't yet.
pub struct TextInput {
    id: String,

    // computed
    multiwidget: MultiWidget,
    subscriptions: Vec<String>,
}

impl TextInput {
    /// Create a `TextInput`. It handles creating its own `Widget`s internally.
    /// ^W cycles focus between the text area and button set
    ///
    /// id: the id data will be posted with (and prefix for child `Widget` ids)
    /// start_px: (x, y) coordinate of the top-left corner of the `TextInput`
    /// dimensions_px: (x, y) dimensions of the `TextInput`
    pub fn new(
        id: String,
        start_px: (usize, usize),
        dimensions_px: (usize, usize),
        x_overflow: XOverflowBehavior,
    ) -> TextInput {
        let text_area_id = id.clone() + "_textarea";
        let text_area = TextArea::new(
            text_area_id.clone(),                    /* id */
            Vec::new(),                              /* subscriptions */
            String::new(),                           /* content */
            true,                                    /* edit */
            start_px,                                /* start */
            (dimensions_px.0, dimensions_px.1 - 30), /* dimensions_px */
            FontSize::P,                             /* font_size */
            x_overflow,
        );

        let menu_id = id.clone() + "_action_menu";
        let menu = Menu::new(
            menu_id.clone(),
            vec![String::from("save"), String::from("cancel")],
            (start_px.0, start_px.1 + dimensions_px.1 - 30), // start_px
            (dimensions_px.0, 30),                           // dimensions_px
            MenuOrientation::HORIZONTAL,
        );

        let multiwidget_id = id.clone() + "_multiwidget";
        let multiwidget = MultiWidget::new(
            multiwidget_id.clone(),
            vec![Box::from(text_area), Box::from(menu)],
            0,             /* focused */
            dimensions_px, /* dimensions */
        );

        TextInput {
            id: id.clone(),
            multiwidget: multiwidget,
            subscriptions: vec![menu_id],
        }
    }
}

impl Widget for TextInput {
    fn id(&self) -> &String {
        return &self.id;
    }

    fn get_value(&self) -> String {
        let textarea_id = self.id.clone() + "_textarea";
        return self.multiwidget.get_value_for_id(textarea_id);
    }

    fn get_subscriptions(&self) -> &Vec<String> {
        return &self.subscriptions;
    }

    fn handle_post(&mut self, id: String, data: String) {
        info!("text input got post from {}: {}", id, data);
    }

    fn draw(&mut self, graphics: &mut Graphics, focused: bool) {
        self.multiwidget.draw(graphics, focused);
    }

    fn handle_key(&mut self, k: Key, graphics: &mut Graphics) -> UIResult {
        if let Key::Special(value) = k {
            if ScanCode::ESCAPE == value {
                return UIResult::CLOSE;
            }
        }
        let result = self.multiwidget.handle_key(k, graphics);
        if let UIResult::POST(ref _id, ref data) = result {
            if data == "cancel" {
                return UIResult::CLOSE;
            } else if data == "save" {
                return UIResult::POST(self.id.clone(), data.clone());
            }
        }
        return result;
    }

    fn dimensions(&mut self) -> (usize, usize) {
        return self.multiwidget.dimensions();
    }
}
