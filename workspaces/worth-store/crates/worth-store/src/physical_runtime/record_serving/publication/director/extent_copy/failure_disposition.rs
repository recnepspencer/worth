use super::{super::RecordPublicationDirector, SharedCopyObligation};
use crate::physical_runtime::PreparedPhysicalMutation;
use std::sync::Arc;

/// Captured before the prepared carrier moves. Keeping this exact ledger alive
/// also covers a concurrent cancellation that removes the director's slot.
pub(in crate::physical_runtime::record_serving::publication::director) struct CopyFailureObligation
{
    obligation: SharedCopyObligation,
}

impl RecordPublicationDirector {
    pub(in crate::physical_runtime::record_serving::publication::director) fn copy_failure_obligation(
        &self,
        prepared: &PreparedPhysicalMutation,
    ) -> Option<CopyFailureObligation> {
        let source = prepared.extent_copy_source()?;
        let slot = self
            .copy_obligation
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let obligation = slot.as_ref()?;
        let state = obligation.lock().unwrap_or_else(|e| e.into_inner());
        if state.operation != prepared.idempotency_identity().bytes()
            || state.intent.map(|(intent, _, _)| intent.source()) != Some(source)
        {
            return None;
        }
        Some(CopyFailureObligation {
            obligation: Arc::clone(obligation),
        })
    }
}

impl CopyFailureObligation {
    /// The final publication has not escaped and no transient carrier can
    /// resume it. Any earlier copy effects remain owned by this independent
    /// ledger (or its completed cancellation), not by an exclusive root fence.
    pub(in crate::physical_runtime::record_serving::publication::director) fn permits_root_release(
        &self,
    ) -> bool {
        let state = self.obligation.lock().unwrap_or_else(|e| e.into_inner());
        state.publication_lsn.is_none()
            && state.published_root.is_none()
            && !state.inspection
            && !state.carrier_alive
    }
}
