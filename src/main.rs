#![no_main]
#![no_std]
#![feature(asm)]
#![feature(abi_efiapi)]

use uefi::prelude::*;

use core::convert::From;
use log::info;

use no_std_compat::prelude::v1::Box;
use no_std_compat::string::*;

mod devices;
mod ui;

use crate::ui::components;
use crate::ui::core::{application, bmp, graphics, widget};

#[entry]
fn efi_main(_image: Handle, mut table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut table).expect_success("failed to init");

    info!("setting up app");

    // font size 1 is tiny so i recommend 2 by default
    let font_sizes = graphics::FontSizes::new(3 /* h1 */, 2 /* h2 */, 2 /* p */);
    let color_scheme = graphics::ColorScheme::new(
        [0x8c, 0x79, 0x40], /* Foreground */
        [0x0f, 0x0f, 0x0f], /* Background */
        [0x80, 0x77, 0x38], /* Cursor */
        [0xf7, 0xff, 0xdd], /* BorderUnfocused */
        [0xd8, 0xe1, 0x93], /* BorderFocused */
    );
    let theme = graphics::Theme {
        font_sizes: font_sizes,
        color_scheme: color_scheme,
    };

    let text_input = components::text_input::TextInput::new(
        String::from("textinput"),
        (0, 0),
        (1024, 600),
        widget::XOverflowBehavior::Wrap,
    );

    let mut application = application::Application::new(table, theme, Box::from(text_input));
    application.run_loop();

    // i keep my todo lists in my code, sue me
    // - tint non-focused/background components
    // - handle "un-drawing" closed widgets
    //   - fake an alpha channel?
    //   - just redraw up the whole UI stack
    // - improve performance
    //   - drawing backgrounds seems to slow the whole thing waaaaay down
    //   - memoize bitmaps for characters as they're used to limit allocations
    // - UIResult::Open(new_widget);
    // - implement resizing and tiling in MultiWidget
    // - UEFI watchdog timer
    // - rethink "subscriptions", not really using that whole system
    //   - maybe support regex subscriptions and non-Widget subscribers
    // - support a way to UIResult::Post() + UIResult::Close
    // - redo components::Menu to support scrolling long button lists

    info!("it's torn down now");

    return Status::SUCCESS;
}
