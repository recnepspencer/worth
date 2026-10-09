use worth_store_physical_format::{PersistedRecordIdentity, RootPublicationCell};

use super::{
    PhysicalMutationIdentity, PhysicalReclaimAttempt, ReclaimFenceState, ReclaimPhase,
    ReclaimPurpose,
};

impl PhysicalReclaimAttempt {
    pub(super) fn lock_fence(&self) -> std::sync::MutexGuard<'_, Option<ReclaimFenceState>> {
        self.fence
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(in crate::physical_runtime) const fn bytes(&self) -> [u8; 16] {
        self.id.0
    }

    pub(in crate::physical_runtime) fn expected_root(&self) -> Option<RootPublicationCell> {
        self.lock_fence()
            .as_ref()
            .filter(|fence| fence.id == self.id)
            .map(|fence| fence.expected_root)
    }

    pub(in crate::physical_runtime) fn register_mutation(
        &self,
        mutation: PhysicalMutationIdentity,
    ) -> bool {
        let mut state = self.lock_fence();
        let Some(fence) = state.as_mut().filter(|fence| fence.id == self.id) else {
            return false;
        };
        if fence.purpose != ReclaimPurpose::PayloadDrop {
            return false;
        }
        match fence.phase {
            ReclaimPhase::BeforeEffect if fence.manifest_mutation.is_none() => {
                fence.manifest_mutation = Some(mutation);
                true
            }
            ReclaimPhase::ManifestPublished if fence.reservation_mutation.is_none() => {
                fence.reservation_mutation = Some(mutation);
                true
            }
            ReclaimPhase::ReservePublished if fence.drop_mutation.is_none() => {
                fence.drop_mutation = Some(mutation);
                true
            }
            _ => false,
        }
    }

    pub(in crate::physical_runtime) fn register_drop_records(
        &self,
        records: &[PersistedRecordIdentity],
    ) -> bool {
        if records.is_empty()
            || records.len() > 1024
            || records.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return false;
        }
        let mut state = self.lock_fence();
        let Some(fence) = state.as_mut().filter(|fence| fence.id == self.id) else {
            return false;
        };
        if fence.purpose != ReclaimPurpose::PayloadDrop
            || fence.phase != ReclaimPhase::ReservePublished
            || fence.drop_mutation.is_some()
            || !fence.drop_records.is_empty()
            || fence.drop_records.try_reserve_exact(records.len()).is_err()
        {
            return false;
        }
        fence.drop_records.extend_from_slice(records);
        true
    }

    pub(in crate::physical_runtime) fn clear_unprepared_drop_records(&self) {
        let mut state = self.lock_fence();
        if let Some(fence) = state.as_mut().filter(|fence| fence.id == self.id) {
            if fence.purpose == ReclaimPurpose::PayloadDrop
                && fence.phase == ReclaimPhase::ReservePublished
                && fence.drop_mutation.is_none()
            {
                fence.drop_records.clear();
            }
        }
    }

    pub(in crate::physical_runtime) fn mark_effect_started(&self) {
        let mut state = self.lock_fence();
        if let Some(fence) = state.as_mut().filter(|fence| fence.id == self.id) {
            if fence.purpose == ReclaimPurpose::PayloadDrop {
                fence.phase = match fence.phase {
                    ReclaimPhase::BeforeEffect => ReclaimPhase::ManifestEffect,
                    ReclaimPhase::ManifestPublished => ReclaimPhase::ReserveEffect,
                    ReclaimPhase::ReservePublished => ReclaimPhase::DropEffect,
                    other => other,
                };
            }
        }
    }

    /// Only a proved effect-free manifest attempt can return this fence to a
    /// droppable pre-effect posture. A published manifest or uncertain effect
    /// cannot be silently converted into an ordinary retry.
    pub(in crate::physical_runtime) fn prove_manifest_no_effect(
        &self,
        proof: &crate::physical_runtime::ProvenNoEffectPhysicalMutation,
    ) -> bool {
        let mut state = self.lock_fence();
        let Some(fence) = state.as_mut().filter(|fence| fence.id == self.id) else {
            return false;
        };
        if fence.purpose != ReclaimPurpose::PayloadDrop
            || fence.phase != ReclaimPhase::ManifestEffect
            || fence.manifest_mutation != Some(proof.mutation_identity())
        {
            return false;
        }
        fence.phase = ReclaimPhase::BeforeEffect;
        fence.manifest_mutation = None;
        if let Some(pending) = fence.release_certificate_pending.as_mut() {
            pending.effect_may_exist = false;
        }
        true
    }

    pub(in crate::physical_runtime) fn complete(self) -> bool {
        let mut state = self.lock_fence();
        if state.as_ref().is_some_and(|fence| {
            fence.id == self.id
                && fence.purpose == ReclaimPurpose::PayloadDrop
                && fence.phase == ReclaimPhase::DropPublished
                && fence.release_certificate_pending.is_none()
        }) {
            *state = None;
            true
        } else {
            false
        }
    }
}

impl Drop for PhysicalReclaimAttempt {
    fn drop(&mut self) {
        // Match the owner's publication-then-reclaim lock order. A pending
        // certificate is restored only for this exact pre-effect attempt;
        // an effect-started or uncertain attempt keeps its fence and denies
        // checkpoint capture until recovered evidence settles it.
        let owner_state = self.state.upgrade();
        let mut publication = owner_state.as_ref().map(|state| {
            state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
        });
        let mut state = self.lock_fence();
        let proven_pre_effect = state.as_ref().is_some_and(|fence| {
            fence.id == self.id
                && fence.phase == ReclaimPhase::BeforeEffect
                && fence
                    .release_certificate_pending
                    .as_ref()
                    .is_none_or(|pending| !pending.effect_may_exist)
        });
        if !proven_pre_effect {
            return;
        }
        if let Some(publication) = publication.as_mut() {
            use super::super::certificate_capacity::CheckpointCustodyState;
            match &publication.checkpoint_custody {
                CheckpointCustodyState::ReleaseCertificatePending { attempt, .. }
                    if *attempt == self.id.0 =>
                {
                    if publication
                        .checkpoint_custody
                        .restore_proven_no_effect(self.id.0)
                    {
                        *state = None;
                    }
                }
                CheckpointCustodyState::ReleaseCertificatePending { .. } => {}
                _ => *state = None,
            }
        } else {
            *state = None;
        }
    }
}
