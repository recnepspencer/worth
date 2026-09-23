use std::sync::Arc;

use super::super::inventory::PhysicalWalSegmentInventoryEntry;
use super::super::PhysicalWalRuntimeOwner;
use crate::physical_runtime::CompletedPhysicalWalReclamationAction;

impl PhysicalWalRuntimeOwner {
    pub(in crate::physical_runtime) fn recovery_tail(
        &self,
    ) -> crate::physical_runtime::PhysicalRecoveryWalTail {
        let state = self
            .shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        crate::physical_runtime::PhysicalRecoveryWalTail::from_inventory(
            state.durable_lsn_end,
            state.segments.entries(),
            state.sealed,
        )
    }

    pub(in crate::physical_runtime) fn bind_publication_retention(
        &self,
        admission: Arc<
            crate::physical_runtime::durability::retention::PhysicalPublicationAdmission,
        >,
    ) {
        *self
            .publication
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(admission);
    }

    pub(in crate::physical_runtime) fn note_sealed_publication(
        &self,
        segment: u64,
        generation: u64,
        bytes: u64,
    ) {
        let admission = self
            .publication
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        if let Some(admission) = admission {
            admission.note_sealed_publication(segment, generation, bytes);
        }
    }

    fn release_sealed_publication(&self, segment: u64, generation: u64) {
        let admission = self
            .publication
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        if let Some(admission) = admission {
            admission.release_sealed_publication(segment, generation);
        }
    }

    pub(super) fn complete_reclamation(
        &self,
        expected: PhysicalWalSegmentInventoryEntry,
        completed: &CompletedPhysicalWalReclamationAction,
    ) -> bool {
        if completed.segment() != expected.identity()
            || completed.lsn_range() != expected.lsn_range()
            || completed.byte_count() != expected.byte_count()
        {
            self.seal_for_inspection();
            return false;
        }
        let identity = expected.identity();
        let byte_count = expected.byte_count();
        {
            let mut state = self
                .shared
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if state.segments.consume_reclaimed_head(expected).is_err() {
                state.sealed = true;
                return false;
            }
            state.segment_count = state.segment_count.saturating_sub(1);
            state.reclaimed_segments = state.reclaimed_segments.saturating_add(1);
            state.reclaimed_bytes = state.reclaimed_bytes.saturating_add(byte_count);
        }
        self.release_sealed_publication(identity.segment().get(), identity.generation().get());
        true
    }
}
