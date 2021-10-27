use uefi::proto::console::text::Key;

use no_std_compat::string::String;
use no_std_compat::vec::Vec;

use crate::graphics::Graphics;
use crate::ui::core::UIResult;

// exposed

mod multi_widget;
pub use multi_widget::MultiWidget;

mod text_area;
pub use text_area::{TextArea, XOverflowBehavior};

mod button;
pub use button::Button;

/// America runs on `Widget`s
/// `Widget`s can be drawn, receive keypresses, post/listen for events.
/// A `Widget` can own and coordinate multiple `Widget`s - see `MultiWidget`.
pub trait Widget {
    /// What's in a name
    fn id(&self) -> &String;

    /// Returns the value held by the `Widget`. Empty string by default because
    /// not all `Widget`s will necessarily hold data
    fn get_value(&self) -> String {
        return String::new();
    }

    /// Return the list of topics this `Widget` wants to receive posts for.
    /// Conventionally this is a list of `Widget` IDs.
    fn get_subscriptions(&self) -> &Vec<String>;

    /// Some container or other will let you know if it receives a `UIResult::POST`
    /// from a `Widget` you're subscribed to.
    fn handle_post(&mut self, _id: String, _data: String) {}

    /// Not much else to say!
    fn draw(&mut self, graphics: &mut Graphics, focused: bool);

    /// Handle any keypress. `Widget`s can use this to move a cursor, write text
    /// to the screen, close themselves, whatever.
    fn handle_key(&mut self, k: Key, graphics: &mut Graphics) -> UIResult;

    /// Return this `Widget`'s dimensions in pixels. Not really using this!
    fn dimensions(&mut self) -> (usize, usize);
}
