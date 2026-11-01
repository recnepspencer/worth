//! Stable document correspondence survives movement of the SDK owning wrapper.
use super::*;

/// Identifies one actual selection cell's document incarnation. It carries no
/// permission, source lease, selected step or navigation authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryDocumentHistoryIdentity(super::super::step::DocumentIncarnation);

impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>> WorthQueryDocumentHistory<'_, S, H> {
    pub fn history_document_identity(
        &self,
        _access: &WorthQueryProgramPublicationAccess,
    ) -> WorthQueryDocumentHistoryIdentity {
        WorthQueryDocumentHistoryIdentity(
            self.selection.lock().journal.current_step().id().document,
        )
    }
}
