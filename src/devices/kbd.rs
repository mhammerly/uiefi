use uefi::prelude::Boot;
use uefi::prelude::SystemTable;
use uefi::proto::console::text::Key;
use uefi::ResultExt;

use no_std_compat::string::String;

/// Return raw keypresses. Some printable, some like LEFT/RIGHT/ESCAPE.
pub fn read_char_raw(table: &mut SystemTable<Boot>) -> Key {
    let events = &mut [table.stdin().wait_for_key_event()];
    table
        .boot_services()
        .wait_for_event(events)
        .expect_success("didn't get key event");
    return table
        .stdin()
        .read_key()
        .expect_success("probably a device error")
        .expect("no key was in the input buffer");
}

/// Unused, but returns keypresses if they're printable else None
#[allow(dead_code)]
pub fn read_char_printable(table: &mut SystemTable<Boot>) -> Option<char> {
    let c = read_char_raw(table);
    match c {
        Key::Printable(value) => {
            return Option::from(char::from(value));
        }
        Key::Special(_value) => {
            return Option::None;
        }
    }
}

/// Unused, but reads an entire line of printable chars at once
#[allow(dead_code)]
pub fn read_line(table: &mut SystemTable<Boot>) -> String {
    let mut s = String::new();
    loop {
        let c = read_char_printable(table);
        match c {
            Option::Some(value) => {
                let char_value = char::from(value);
                if char_value == '\r' || char_value == '\n' {
                    break;
                }
                s.push(value.into());
            }
            Option::None => {}
        }
    }
    return s;
}
