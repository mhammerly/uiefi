use no_std_compat::string::String;

pub mod application;
pub mod bmp;
pub mod graphics;
pub mod widget;

pub mod font;
pub use font::{FONT_HEIGHT, FONT_WIDTH};

#[derive(PartialEq)]
/// Returned by `Widget::handle_key()`. Can indicate success or request some
/// action be taken by the `Widget`'s container.
pub enum UIResult {
    /// Everything's groovy
    OK,
    /// We're publishing some data; find `Widgets` who subscribe and show them
    POST(String, String), /* id, data */

    /// Kill me
    CLOSE,
}
