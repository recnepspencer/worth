use super::{PhysicalReclaimAttempt, ReclaimPhase};
use crate::physical_runtime::ProvenNoEffectPhysicalMutation;

impl PhysicalReclaimAttempt {
    /// Manifest has published, but no descriptor mutation was prepared. The
    /// manifest remains durable custody; no record route has been dropped.
    pub(in crate::physical_runtime) fn settle_manifest_without_drop(&self) -> bool {
        let mut state = self.lock_fence();
        if state.as_ref().is_some_and(|fence| {
            fence.id == self.id
                && fence.phase == ReclaimPhase::ManifestPublished
                && fence.reservation_mutation.is_none()
                && fence.drop_records.is_empty()
        }) {
            *state = None;
            true
        } else {
            false
        }
    }

    /// A selected reservation remains durable, but no descriptor was
    /// prepared. Release the runtime fence for exact same-attempt recovery.
    pub(in crate::physical_runtime) fn settle_reservation_without_drop(&self) -> bool {
        let mut state = self.lock_fence();
        if state.as_ref().is_some_and(|fence| {
            fence.id == self.id
                && fence.phase == ReclaimPhase::ReservePublished
                && fence.drop_mutation.is_none()
                && fence.drop_records.is_empty()
        }) {
            *state = None;
            true
        } else {
            false
        }
    }

    pub(in crate::physical_runtime) fn prove_reservation_no_effect(
        &self,
        terminal: &ProvenNoEffectPhysicalMutation,
    ) -> bool {
        let mut state = self.lock_fence();
        if state.as_ref().is_some_and(|fence| {
            fence.id == self.id
                && fence.reservation_mutation == Some(terminal.mutation_identity())
                && matches!(
                    fence.phase,
                    ReclaimPhase::ManifestPublished | ReclaimPhase::ReserveEffect
                )
        }) {
            *state = None;
            true
        } else {
            false
        }
    }

    /// Only the exact registered descriptor's terminal no-effect fact may
    /// release its fence. An indeterminate descriptor remains inspection-bound.
    pub(in crate::physical_runtime) fn prove_drop_no_effect(
        &self,
        terminal: &ProvenNoEffectPhysicalMutation,
    ) -> bool {
        let mut state = self.lock_fence();
        if state.as_ref().is_some_and(|fence| {
            fence.id == self.id
                && fence.drop_mutation == Some(terminal.mutation_identity())
                && matches!(
                    fence.phase,
                    ReclaimPhase::ReservePublished | ReclaimPhase::DropEffect
                )
        }) {
            *state = None;
            true
        } else {
            false
        }
    }
}
