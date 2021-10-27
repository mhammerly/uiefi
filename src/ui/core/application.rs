use uefi::prelude::Boot;
use uefi::prelude::SystemTable;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::proto::console::text::Key;
use uefi::ResultExt;

use no_std_compat::prelude::v1::{vec, Box};
use no_std_compat::vec::Vec;

use crate::devices::kbd;
use crate::graphics::{Graphics, Theme};
use crate::ui::core::UIResult;
use crate::widget::Widget;

type UIStack = Vec<Box<dyn Widget>>;

/// `Application` is the top-level component. It takes ownership of the UEFI
/// `SystemTable`.
///
/// `Application` implements `run_loop()` which reads keystroke after keystroke
/// and forwards them to the top `Widget` on the UI stack. When it receives a
/// `UIResult::POST()` it will run up the stack and forward it to any `Widget`
/// that subscribes to the id it was posted with.
pub struct Application<'a> {
    table: SystemTable<Boot>,
    graphics: Graphics<'a>,
    pub ui_stack: UIStack,
}

impl<'_static> Application<'static> {
    /// Create an `Application`.
    ///
    /// `table`: the UEFI `SystemTable`, moved into `Application`
    /// `theme`: a theme defining some colors and font scaling factors
    /// `initial_ui` the `Widget` at the bottom of the UI stack. The homepage as it were
    pub fn new(
        table: SystemTable<Boot>,
        theme: Theme,
        initial_ui: Box<dyn Widget>,
    ) -> Application<'_static> {
        let gop = table
            .boot_services()
            .locate_protocol::<GraphicsOutput>()
            .expect_success("failed to load graphics protocol");
        let gop = unsafe { &mut *gop.get() };

        // ALERT: technically Graphics still has a mutable ref to part of SystemTable
        // which means we're being naughty when we later take a mutable borrow of
        // SystemTable to await keystrokes
        let graphics = Graphics::new(gop, theme);

        Application {
            table: table,
            graphics: graphics,
            ui_stack: vec![initial_ui],
        }
    }

    /// Draw every component from the bottom of the stack to the top.
    fn draw(&mut self) {
        for i in 0..self.ui_stack.len() - 1 {
            self.ui_stack[i].draw(&mut self.graphics, false);
        }
        self.ui_stack
            .last_mut()
            .expect("UIStack should not be empty")
            .draw(&mut self.graphics, true);
    }

    /// Handle a keypress by giving it to the `Widget` at the top of the `UIStack`.
    fn handle_key(&mut self, k: Key) -> UIResult {
        return self
            .ui_stack
            .last_mut()
            .expect("UIStack should not be empty")
            .handle_key(k, &mut self.graphics);
    }

    /// Set the resolution here (for lack of a better place) and then loop:
    /// Listen for keystroke after keystroke, forward them to the top of the UI
    /// stack, and handle the `UIResult` values they return.
    pub fn run_loop(&mut self) {
        self.graphics.set_resolution((1024, 600));
        self.draw();

        loop {
            let c = kbd::read_char_raw(&mut self.table);
            let result = self.handle_key(c);

            match result {
                UIResult::OK => {}
                UIResult::CLOSE => {
                    self.ui_stack.pop();
                    if self.ui_stack.len() == 0 {
                        return;
                    }
                    self.draw();
                }
                UIResult::POST(id, data) => {
                    for widget in &mut self.ui_stack {
                        if widget.get_subscriptions().contains(&id) {
                            let (cloned_id, cloned_data) = (id.clone(), data.clone());
                            widget.handle_post(cloned_id, cloned_data);
                        }
                    }
                }
            }
        }
    }
}
