use super::loop_control::UiNativeLoopControl;

use super::{UiNativeEventLoopApplication, UiNativeEventLoopRunDenial};

impl<Client> UiNativeEventLoopApplication<Client> {
    pub(super) fn fail(
        &mut self,
        event_loop: &dyn UiNativeLoopControl,
        denial: UiNativeEventLoopRunDenial,
    ) {
        self.failure = Some(denial);
        event_loop.exit();
    }
}
