//! Explicit authenticated close preserves the actual native owner on every refusal.
use super::*;
use worth_query_host::facade::application_entry::WorthQueryApplicationRequest;
use super::super::application_authentication::authenticate;

impl WorthUiStatusMutation {
    /// Close under fresh authority. Unresolved native work refuses without discarding custody.
    pub fn close(mut self, scope: &WorthQueryRequestScope) -> Result<WorthUiStatusMutationClose, Self> {
        let principal = match authenticate(self.application.installed_schema(), scope) {
            Ok(principal) => principal,
            Err(denial) => return Err(self.refused(WorthUiStatusMutationStop::Setup(denial))),
        };
        let request = self.application.request(&principal, scope);
        let result = self.state.close_native(&request);
        match result {
            Ok(operation) => Ok(WorthUiStatusMutationClose { operation, live: self.state.live_close.take() }),
            Err(denial) => Err(self.refused(WorthUiStatusMutationStop::Close(denial))),
        }
    }
}
impl MutationState {
    pub(super) fn close_native(&mut self, request: &WorthQueryApplicationRequest<'_, '_, '_, WorthUiApplicationSchema>) -> Result<Option<WorthQueryClosedOperation>, WorthQueryOperationCloseDenial> {
        let native = self.native.take().expect("one retained original owner");
        match native.close(request) {
            Ok(receipt) => Ok(receipt),
            Err((native, denial)) => { self.native = Some(native); Err(denial) }
        }
    }
}
