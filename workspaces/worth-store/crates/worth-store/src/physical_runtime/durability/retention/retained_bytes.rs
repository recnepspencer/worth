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
    /// publication. A generation map would collide with data-frame generations
    /// and would hide the second charge.
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
