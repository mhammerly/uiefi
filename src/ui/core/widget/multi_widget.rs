use uefi::proto::console::text::Key;

use no_std_compat::prelude::v1::Box;
use no_std_compat::string::String;
use no_std_compat::vec::Vec;

use crate::graphics::Graphics;
use crate::ui::core::UIResult;
use crate::widget::Widget;

/// A `Widget` that owns and coordinates multiple `Widget`s.
pub struct MultiWidget {
    id: String,
    components: Vec<Box<dyn Widget>>,
    focused: usize,
    dimensions: (usize, usize),

    // computed
    subscriptions: Vec<String>,
}

impl MultiWidget {
    /// Create a new `MultiWidget`. Keypresses are delivered to the "focused"
    /// `Widget`, except maybe ^W will be intercepted here to rotate focus.
    /// In the future resizing/tiling may be supported which will make
    /// `dimensions` matter more.
    ///
    /// id: not important for MultiWidget as no data gets posted
    /// components: the list of `Widget`s controlled by this `MultiWidget`
    /// focused: the index in `components` of the "focused" `Widget`
    /// dimensions: the dimensions that `Widget`s should be contained within
    pub fn new(
        id: String,
        components: Vec<Box<dyn Widget>>,
        focused: usize,
        dimensions: (usize, usize),
    ) -> MultiWidget {
        // doesn't filter out unique subscriptions but that's fine
        let subscriptions: Vec<String> = components
            .iter()
            .flat_map(|c| c.get_subscriptions().clone())
            .collect();
        MultiWidget {
            id: id,
            subscriptions: subscriptions,
            components: components,
            focused: focused,
            dimensions: dimensions,
        }
    }

    /// get_value() is not useful if the wrong `Widget` is focused. This method
    /// provides a way for a component to get the value of a specific `Widget`,
    /// say for a `TextInput` wanting the value of the text field rather than
    /// the "Save" button.
    pub fn get_value_for_id(&self, id: String) -> String {
        return self
            .components
            .iter()
            .find(|c| *c.id() == id)
            .expect("could not find id")
            .as_ref()
            .get_value();
    }

    /// Rotate focus to the previous component in the list.
    /// The focused component is drawn on top and receives all the keystrokes
    /// given to the `MultiWidget`.
    pub fn focus_prev(&mut self, graphics: &mut Graphics) {
        if self.components.len() >= 2 {
            if self.focused == 0 {
                self.focused = self.components.len() - 1;
            } else {
                self.focused -= 1;
            }
            self.draw(graphics, true);
        }
    }

    /// Rotate focus to the next component in the list.
    /// The focused component is drawn on top and receives all the keystrokes
    /// given to the `MultiWidget`.
    pub fn focus_next(&mut self, graphics: &mut Graphics) {
        if self.components.len() >= 2 {
            self.focused = (self.focused + 1) % self.components.len();
            self.draw(graphics, true);
        }
    }
}

impl Widget for MultiWidget {
    fn id(&self) -> &String {
        return &self.id;
    }

    fn get_value(&self) -> String {
        return self.components[self.focused].as_ref().get_value();
    }

    fn get_subscriptions(&self) -> &Vec<String> {
        return &self.subscriptions;
    }

    fn handle_post(&mut self, id: String, data: String) {
        for widget in &mut self.components {
            if widget.get_subscriptions().contains(&id) {
                let (cloned_id, cloned_data) = (id.clone(), data.clone());
                widget.handle_post(cloned_id, cloned_data);
            }
        }
    }

    fn draw(&mut self, graphics: &mut Graphics, focused: bool) {
        // figure out a way to tint out-of-focus components
        // also a way to tint all components for when there is a higher layer
        for i in 0..self.components.len() {
            if i != self.focused {
                self.components[i].draw(graphics, false);
            }
        }
        self.components[self.focused].draw(graphics, focused);
    }

    fn handle_key(&mut self, k: Key, graphics: &mut Graphics) -> UIResult {
        if let Key::Printable(value) = k {
            if '\x17' == char::from(value) {
                // ^W, the W stands for "window"
                // move focus ahead by 1 and wrap around if we hit the end.
                // this won't play nicely when multiple `MultiWidget`s are in
                // play. they'll fight over the ^W
                self.focused = (self.focused + 1) % self.components.len();
                self.draw(graphics, true);
                return UIResult::OK;
            }
        }
        let result = self.components[self.focused].handle_key(k, graphics);
        if result == UIResult::CLOSE {
            self.components.remove(self.focused);
            if self.components.len() == 0 {
                return UIResult::CLOSE;
            }
            self.focused = self.focused & self.components.len();
            self.draw(graphics, true);
            return UIResult::OK;
        }
        return result;
    }

    fn dimensions(&mut self) -> (usize, usize) {
        return self.dimensions;
    }
}
