use winit::event_loop::ActiveEventLoop;

use super::client_invocation::UiNativeEventLoopClientInvocation;
use super::{UiNativeEventLoopApplication, UiNativeEventLoopClient, UiNativeEventLoopRunDenial};

impl<Client: UiNativeEventLoopClient> UiNativeEventLoopApplication<Client> {
    pub(super) fn handle_close_requested(&mut self, event_loop: &ActiveEventLoop) {
        let directive = self.client_or_denied().and_then(|client| {
            client
                .invoke_external_close_requested()
                .map_err(UiNativeEventLoopRunDenial::ClientCallback)
        });
        match directive {
            Ok(directive) => {
                self.apply_client_directive(event_loop, directive);
            }
            Err(denial) => self.fail(event_loop, denial),
        }
    }
}
