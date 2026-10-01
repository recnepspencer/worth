//! Joins the selected V3 route, reserved control frame, completed C9 member,
//! and exact active drop fence before adding a batch to checkpoint custody.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    PhysicalRecordFormatDeclaration, ReleaseCustodyHeadMutationV1,
    ReleasedDropCumulativeEvidenceV1, ReleasedDropTipProvenanceV1, ReleasedDropWalFateWitnessV1,
    RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES, RELEASE_CHECKPOINT_BATCH_WIRE_BYTES,
};

use super::heads::SelectedReleaseHeadStep;
use super::{
    ReleaseCertificateCapacityDenial, SelectedReleaseBatchBasis, CERTIFICATE_FRAME_OVERHEAD,
};
use crate::physical_runtime::{
    blob::reclaim::released::SelectedReleasedDescriptorObservation,
    durability::{PhysicalCurrentRootOwner, PhysicalReclaimAttempt},
    CompletedPhysicalMutation,
};

impl PhysicalCurrentRootOwner {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::physical_runtime) fn commit_selected_release_certificate(
        &self,
        attempt: &PhysicalReclaimAttempt,
        selected: &SelectedReleasedDescriptorObservation,
        completed: &CompletedPhysicalMutation,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<(), ReleaseCertificateCapacityDenial> {
        let mut state = self.lock_publication_state();
        let mut fence = self.lock_reclaim();
        let active = fence
            .as_mut()
            .filter(|active| active.matches_attempt(attempt.bytes()))
            .ok_or(ReleaseCertificateCapacityDenial::ReclaimFenceMismatch)?;
        let pending = active
            .release_certificate_pending
            .as_ref()
            .filter(|pending| pending.effect_may_exist && pending.needed_records >= 2)
            .ok_or(ReleaseCertificateCapacityDenial::ReclaimFenceMismatch)?;
        let descriptor = selected.descriptor();
        let base = descriptor.base();
        let reservation = selected.reservation();
        let request = reservation.request();
        let current = &state.current_root;
        let root_sha256: [u8; 32] = Sha256::digest(current.encode(format)).into();
        let current_generation = current.generation();
        let current_cell = current.root_cell();
        let current_head_root = current.release_custody_head_root();
        let current_tree_identity = current.tree_identity();
        let head_effect = completed
            .selected_head_transition()
            .ok_or(ReleaseCertificateCapacityDenial::SelectedFactMismatch)?;
        let ReleaseCustodyHeadMutationV1::Upsert { next, .. } = head_effect.mutation() else {
            return Err(ReleaseCertificateCapacityDenial::SelectedFactMismatch);
        };
        let Some((start, end, identity_digest, payload_digest)) =
            completed.wal_frame_header_witness()
        else {
            return Err(ReleaseCertificateCapacityDenial::SelectedFactMismatch);
        };
        let fate = ReleasedDropWalFateWitnessV1::new(start, end, identity_digest, payload_digest)
            .map_err(|_| ReleaseCertificateCapacityDenial::SelectedFactMismatch)?;
        if !active.is_selected_drop_mutation(completed.mutation_identity())
            || active.selected_root_cell() != current_cell
            || selected.root().runtime() != self.runtime_identity
            || selected.root().root() != current_cell
            || base.reclaim_attempt() != attempt.bytes()
            || base.candidate_root_generation() != current_generation
            || completed.completed_breadth().current_root_generation() != current_generation
            || completed.persisted_records() != [selected.record()]
            || completed.idempotency_identity().bytes() != request.idempotency()
            || completed.request_fingerprint().bytes() != request.fingerprint()
            || descriptor.custody().request() != request
            || reservation.reclaim_attempt() != attempt.bytes()
            || reservation.reserved_selected_generation() != base.source_root_generation()
            || reservation.manifest_record() != base.manifest_record()
            || reservation.manifest_frame_sha256() != base.manifest_frame_sha256()
            || reservation.source_basis_digest() != base.source_basis_digest()
            || head_effect.tree_identity() != current_tree_identity
            || current_head_root != Some(head_effect.result_root())
            || next.descriptor_record() != selected.record()
            || next.descriptor_frame_sha256() != selected.frame_sha256()
            || next.manifest_record() != base.manifest_record()
            || next.manifest_frame_sha256() != base.manifest_frame_sha256()
            || next.reservation_record() != selected.reservation_record()
            || next.reservation_frame_sha256() != selected.reservation_frame_sha256()
            || next.source_basis_digest() != base.source_basis_digest()
            || next.predecessor() != base.predecessor()
            || next.source_root_generation() != base.source_root_generation()
            || next.cumulative_dropped() != base.cumulative_dropped()
            || next.terminal() != base.terminal()
            || !matches!(
                &state.checkpoint_custody,
                super::super::certificate_capacity::CheckpointCustodyState::ReleaseCertificatePending { attempt: pending, .. }
                    if *pending == attempt.bytes()
            )
        {
            return Err(ReleaseCertificateCapacityDenial::SelectedFactMismatch);
        }
        let tip = ReleasedDropTipProvenanceV1::new(
            selected.record(),
            selected.frame_sha256(),
            selected.reservation_record(),
            selected.reservation_frame_sha256(),
            request,
            fate,
            current_generation,
            root_sha256,
        )
        .map_err(|_| ReleaseCertificateCapacityDenial::SelectedFactMismatch)?;
        let ledger = state
            .release_ledger
            .selected_mut()
            .ok_or(ReleaseCertificateCapacityDenial::SelectedLedgerUnavailable)?;
        let head_step = SelectedReleaseHeadStep::from_effect(head_effect);
        let mut effective_heads = ledger.effective_heads.clone();
        head_step.apply(&mut effective_heads)?;
        let evidence = ReleasedDropCumulativeEvidenceV1::new(
            selected.record(),
            selected.frame_sha256(),
            descriptor.custody_digest(),
            selected.reservation_record(),
            selected.reservation_frame_sha256(),
            fate,
            current_generation,
            root_sha256,
            base.predecessor(),
            base.manifest_count(),
            base.terminal(),
        )
        .map_err(|_| ReleaseCertificateCapacityDenial::SelectedFactMismatch)?;
        let (cumulative_dropped, cumulative_digest) = evidence
            .advance(ledger.cumulative_dropped, ledger.cumulative_digest)
            .map_err(|_| ReleaseCertificateCapacityDenial::SelectedFactMismatch)?;
        // V3's cumulative count is scoped to one released generation; this
        // ratchet count is Store-wide and may include other generations.
        if ledger.pending_batches.len() >= 63 {
            return Err(ReleaseCertificateCapacityDenial::SelectedFactMismatch);
        }
        let next_count = ledger.pending_batches.len() as u64 + 1;
        let section_bytes = next_count
            .checked_mul(RELEASE_CHECKPOINT_BATCH_WIRE_BYTES as u64 + CERTIFICATE_FRAME_OVERHEAD)
            .and_then(|bytes| {
                bytes.checked_add(
                    RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES as u64
                        + CERTIFICATE_FRAME_OVERHEAD,
                )
            })
            .ok_or(ReleaseCertificateCapacityDenial::CapacityExhausted)?;
        if section_bytes > worth_store_physical_format::MAX_CHECKPOINT_CERTIFICATE_BYTES
            || section_bytes
                > u64::from(ledger.used_bytes) + u64::from(pending.worst_case_encoded_bytes)
        {
            return Err(ReleaseCertificateCapacityDenial::CapacityExhausted);
        }
        ledger
            .pending_batches
            .try_reserve_exact(1)
            .map_err(|_| ReleaseCertificateCapacityDenial::CapacityExhausted)?;
        ledger
            .pending_head_steps
            .try_reserve_exact(1)
            .map_err(|_| ReleaseCertificateCapacityDenial::CapacityExhausted)?;
        ledger.pending_batches.push(SelectedReleaseBatchBasis {
            head_step: Some(head_step),
            descriptor_record: selected.record(),
            descriptor_frame_sha256: selected.frame_sha256(),
            custody_digest: descriptor.custody_digest(),
            reservation_record: selected.reservation_record(),
            reservation_frame_sha256: selected.reservation_frame_sha256(),
            request,
            fate,
            candidate_root_generation: current_generation,
            candidate_root_sha256: root_sha256,
            predecessor: base.predecessor(),
            cumulative_dropped,
            cumulative_digest,
            terminal: base.terminal(),
        });
        ledger.used_records = u16::try_from(next_count + 1).expect("bounded certificate count");
        ledger.used_bytes = u32::try_from(section_bytes).expect("bounded certificate bytes");
        ledger.cumulative_dropped = cumulative_dropped;
        ledger.cumulative_digest = cumulative_digest;
        ledger.selected_tip = Some(tip);
        ledger.pending_head_steps.push(head_step);
        ledger.effective_heads = effective_heads;
        ledger.terminal = base.terminal();
        assert!(state
            .checkpoint_custody
            .fulfill_selected_release_certificate(attempt.bytes()));
        active.release_certificate_pending = None;
        Ok(())
    }
}
