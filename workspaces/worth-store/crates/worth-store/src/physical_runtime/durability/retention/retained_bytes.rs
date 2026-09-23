use std::sync::{Arc, Weak};

use super::{PhysicalPublicationAdmission, PhysicalRetentionGrowthDenial};

pub(in crate::physical_runtime) struct RetainedByteLease {
    admission: Weak<PhysicalPublicationAdmission>,
    bytes: u64,
}

impl PhysicalPublicationAdmission {
    /// Charges bytes that are not keyed by an artifact generation.
    ///
    /// WAL frames and root-publication metadata are new retained bytes on every
    /// publication. The charge stays until the WAL segment that carried the frames
    /// is reclaimed.
    pub(in crate::physical_runtime) fn reserve_retained_bytes(
        self: &Arc<Self>,
        bytes: u64,
    ) -> Result<RetainedByteLease, PhysicalRetentionGrowthDenial> {
        let mut state = self.lock();
        if bytes == 0 {
            drop(state);
            return Ok(RetainedByteLease {
                admission: Weak::new(),
                bytes: 0,
            });
        }
        let remaining = state.remaining_bytes();
        if bytes > remaining {
            return Err(PhysicalRetentionGrowthDenial {
                requested_bytes: bytes,
                requested_entries: 0,
                remaining_bytes: remaining,
                remaining_entries: state.remaining_entries(),
            });
        }
        state.charged_bytes = state.charged_bytes.saturating_add(bytes);
        drop(state);
        Ok(RetainedByteLease {
            admission: Arc::downgrade(self),
            bytes,
        })
    }
}

impl RetainedByteLease {
    pub(in crate::physical_runtime) fn seal(mut self) {
        self.admission = Weak::new();
    }
}

impl Drop for RetainedByteLease {
    fn drop(&mut self) {
        let Some(admission) = self.admission.upgrade() else {
            return;
        };
        let mut state = admission.lock();
        state.charged_bytes = state.charged_bytes.saturating_sub(self.bytes);
    }
}

impl PhysicalPublicationAdmission {
    pub(in crate::physical_runtime) fn note_sealed_publication(
        &self,
        segment: u64,
        generation: u64,
        bytes: u64,
    ) {
        if bytes == 0 {
            return;
        }
        self.lock()
            .sealed_publications
            .push((segment, generation, bytes));
    }

    /// Drops the publication charge once the WAL segment that carried it is gone.
    pub(in crate::physical_runtime) fn release_sealed_publication(
        &self,
        segment: u64,
        generation: u64,
    ) {
        let mut state = self.lock();
        let mut released = 0_u64;
        state
            .sealed_publications
            .retain(|(sealed_segment, sealed_generation, bytes)| {
                if *sealed_segment == segment && *sealed_generation == generation {
                    released = released.saturating_add(*bytes);
                    false
                } else {
                    true
                }
            });
        state.charged_bytes = state.charged_bytes.saturating_sub(released);
    }
}
