//! Explicit authenticated close preserves the actual native owner on every refusal.
use super::super::application_authentication::authenticate;
use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequest, WorthQueryApplicationRequestExt,
};

impl WorthUiStatusMutation {
    /// Shutdown may abandon an unexecuted command only after authentic native close.
    /// Completed delivery and prior committed evidence stay owned for their ordinary
    /// continuation; cleanup release alone would not prove that publication was presented.
    pub fn close_without_commit(
        self,
        scope: &WorthQueryRequestScope,
    ) -> Result<WorthUiStatusMutationClose, Self> {
        match self.state.native.as_ref().expect("one retained original owner") {
            NativeCustody::Source(native::State::Completed(_)) | NativeCustody::Action(native::State::Completed(_)) =>
                Err(self.refused(WorthUiStatusMutationStop::PublicationPending)),
            NativeCustody::Source(native::State::Terminal(terminal)) if matches!(terminal.native_outcome(), Some(worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::AlreadyCommitted(_) | worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::PreviouslyCommitted(_))) =>
                Err(self.refused(WorthUiStatusMutationStop::PublicationPending)),
            NativeCustody::Action(native::State::Terminal(terminal)) if matches!(terminal.native_outcome(), Some(worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::AlreadyCommitted(_) | worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::PreviouslyCommitted(_))) =>
                Err(self.refused(WorthUiStatusMutationStop::PublicationPending)),
            _ => self.close(scope),
        }
    }

    /// Close under fresh authority. Unresolved native work refuses without discarding custody.
    pub fn close(
        mut self,
        scope: &WorthQueryRequestScope,
    ) -> Result<WorthUiStatusMutationClose, Self> {
        let principal = match authenticate(self.application.installed_schema(), scope) {
            Ok(principal) => principal,
            Err(denial) => return Err(self.refused(WorthUiStatusMutationStop::Setup(denial))),
        };
        let request = self.application.request(&principal, scope);
        let result = self.state.close_native(&request);
        match result {
            Ok(operation) => Ok(WorthUiStatusMutationClose {
                operation,
                live: self.state.live_close.take(),
            }),
            Err(denial) => Err(self.refused(WorthUiStatusMutationStop::Close(denial))),
        }
    }
}
impl MutationState {
    pub(super) fn close_native(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, WorthUiApplicationSchema>,
    ) -> Result<Option<WorthQueryClosedOperation>, WorthQueryOperationCloseDenial> {
        let native = self.native.take().expect("one retained original owner");
        match native.close(request) {
            Ok(receipt) => Ok(receipt),
            Err((native, denial)) => {
                self.native = Some(native);
                Err(denial)
            }
        }
    }
}
