//! A verified post-checkpoint WAL drop is a pending batch, never a fabricated
//! tag-7 certificate on the selected NoRelease checkpoint.

use worth_store_physical_format::{
    BlobReclaimDescriptorV3, PersistedRecordIdentity, ReleasedDropCumulativeEvidenceV1,
    ReleasedDropTipProvenanceV1, ReleasedDropWalFateWitnessV1, MAX_CHECKPOINT_CERTIFICATE_BYTES,
    MAX_CHECKPOINT_CERTIFICATE_RECORDS, RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES,
    RELEASE_CHECKPOINT_BATCH_WIRE_BYTES,
};
use worth_store_recovery_physics::{
    VerifiedEffectiveReleaseHeadRosterV14, VerifiedOrderedHistoricalReleaseCustody,
    VerifiedOrderedPendingWalReleaseBatch, VerifiedOrderedRootEdge,
    VerifiedPendingWalReleaseCustody,
};

use super::{
    heads::SelectedReleaseHeadStep,
    reopen::{encoding::marker_digest_with_resident, RecoveredReleaseLedgerDenial as Denial},
    ReleaseLedgerState, SelectedNoReleaseMarkerBasis, SelectedReleaseBatchBasis,
    SelectedReleaseCustodyLedger, CERTIFICATE_FRAME_OVERHEAD,
};
use crate::physical_runtime::recovery_residency::StoreRejoinResidentLedger;

impl ReleaseLedgerState {
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn from_verified_ordered_historical(
        verified: &VerifiedOrderedHistoricalReleaseCustody,
        effective: &VerifiedEffectiveReleaseHeadRosterV14,
        resident: &mut StoreRejoinResidentLedger,
    ) -> Result<Self, Denial> {
        let mut ledger = match (verified.marker(), verified.selected_head_v2()) {
            (Some(marker), None)
                if effective.checkpoint_source_heads().is_empty()
                    && effective.checkpoint_source_root().is_none() =>
            {
                let mut ledger = SelectedReleaseCustodyLedger::trusted_genesis();
                ledger.no_release_marker = Some(SelectedNoReleaseMarkerBasis::Selected {
                    checkpoint: marker.checkpoint(),
                    root_sha256: marker.root_sha256(),
                    marker_payload_sha256: marker_digest_with_resident(marker, resident)?,
                });
                ledger.used_records = 1;
                ledger.used_bytes = u32::try_from(
                    RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES as u64
                        + CERTIFICATE_FRAME_OVERHEAD,
                )
                .map_err(|_| Denial::SelectedFactMismatch)?;
                ledger
            }
            (None, Some(base)) => {
                let Self::Selected(ledger) = Self::from_verified_v2_source(
                    base.accumulator_v2(),
                    effective.checkpoint_source_root(),
                    effective.checkpoint_source_heads().iter().copied(),
                    resident,
                )?
                else {
                    return Err(Denial::SelectedFactMismatch);
                };
                ledger
            }
            _ => return Err(Denial::SelectedFactMismatch),
        };
        if verified.selected_root_frame_sha256() != effective.effective_root_frame_sha256()
            || verified.selected_root().release_custody_head_root()
                != Some(effective.effective_root())
            || verified.selected_root().next_release_custody_head_block()
                != effective.effective_next_block()
        {
            return Err(Denial::SelectedFactMismatch);
        }
        let mut cursor = 0;
        for (index, edge) in verified.history().edges().iter().enumerate() {
            let VerifiedOrderedRootEdge::Released(edge) = edge else {
                continue;
            };
            let batch = verified
                .released_batches()
                .get(cursor)
                .ok_or(Denial::SelectedFactMismatch)?;
            if batch.edge_index() != index {
                return Err(Denial::SelectedFactMismatch);
            }
            let (attached_index, replay) = effective
                .ordered_replays()
                .get(cursor)
                .ok_or(Denial::SelectedFactMismatch)?;
            if *attached_index != index {
                return Err(Denial::SelectedFactMismatch);
            }
            ledger.append_pending_batch(BatchEvidence::ordered(batch, edge), resident)?;
            let step = SelectedReleaseHeadStep::from_effect(replay.replay().effect());
            ledger.effective_heads.apply_transition_admitted(
                replay.replay().effect().source_root(),
                Some(replay.replay().effect().result_root()),
                replay.replay().effect().mutation(),
                resident,
            )?;
            resident.grow_vec(&mut ledger.pending_head_steps, 1)?;
            ledger.pending_head_steps.push(step);
            ledger
                .pending_batches
                .last_mut()
                .ok_or(Denial::SelectedFactMismatch)?
                .head_step = Some(step);
            cursor += 1;
        }
        if cursor == verified.released_batches().len()
            && cursor == effective.ordered_replays().len()
            && cursor != 0
            && ledger.effective_heads.matches_selected(
                Some(effective.effective_root()),
                effective.effective_heads(),
                effective.effective_digest(),
            )?
        {
            Ok(Self::Selected(ledger))
        } else {
            Err(Denial::SelectedFactMismatch)
        }
    }

    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn from_verified_pending_wal(
        verified: &VerifiedPendingWalReleaseCustody,
        effective: &VerifiedEffectiveReleaseHeadRosterV14,
        resident: &mut StoreRejoinResidentLedger,
    ) -> Result<Self, Denial> {
        let root = verified
            .published_root()
            .ok_or(Denial::SelectedFactMismatch)?;
        let root_sha256 = verified
            .published_root_sha256()
            .ok_or(Denial::SelectedFactMismatch)?;
        let replay = verified
            .selected_head_replay()
            .ok_or(Denial::SelectedFactMismatch)?;
        verified.topologies().ok_or(Denial::SelectedFactMismatch)?;
        if effective.effective_root_frame_sha256() != root_sha256
            || root.release_custody_head_root() != Some(effective.effective_root())
            || root.next_release_custody_head_block() != effective.effective_next_block()
            || replay.effect().result_root() != effective.effective_root()
        {
            return Err(Denial::SelectedFactMismatch);
        }
        let mut ledger = match (verified.marker(), verified.selected_head_v2()) {
            (Some(marker), None)
                if effective.checkpoint_source_heads().is_empty()
                    && effective.checkpoint_source_root().is_none() =>
            {
                let mut ledger = SelectedReleaseCustodyLedger::trusted_genesis();
                ledger.no_release_marker = Some(SelectedNoReleaseMarkerBasis::Selected {
                    checkpoint: marker.checkpoint(),
                    root_sha256: marker.root_sha256(),
                    marker_payload_sha256: marker_digest_with_resident(marker, resident)?,
                });
                ledger.used_records = 1;
                ledger.used_bytes = u32::try_from(
                    RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES as u64
                        + CERTIFICATE_FRAME_OVERHEAD,
                )
                .map_err(|_| Denial::SelectedFactMismatch)?;
                ledger
            }
            (None, Some(base)) => {
                let Self::Selected(ledger) = Self::from_verified_v2_source(
                    base.accumulator_v2(),
                    effective.checkpoint_source_root(),
                    effective.checkpoint_source_heads().iter().copied(),
                    resident,
                )?
                else {
                    return Err(Denial::SelectedFactMismatch);
                };
                ledger
            }
            _ => return Err(Denial::SelectedFactMismatch),
        };
        let mut cursor = 0;
        if let Some(history) = verified.ordered_history() {
            for (index, edge) in history.edges().iter().enumerate() {
                let VerifiedOrderedRootEdge::Released(edge) = edge else {
                    continue;
                };
                let batch = verified
                    .ordered_released_batches()
                    .get(cursor)
                    .ok_or(Denial::SelectedFactMismatch)?;
                let (attached_index, attached) = effective
                    .ordered_replays()
                    .get(cursor)
                    .ok_or(Denial::SelectedFactMismatch)?;
                if batch.edge_index() != index || *attached_index != index {
                    return Err(Denial::SelectedFactMismatch);
                }
                ledger.append_pending_batch(BatchEvidence::ordered(batch, edge), resident)?;
                let step = SelectedReleaseHeadStep::from_effect(attached.replay().effect());
                ledger.effective_heads.apply_transition_admitted(
                    attached.replay().effect().source_root(),
                    Some(attached.replay().effect().result_root()),
                    attached.replay().effect().mutation(),
                    resident,
                )?;
                resident.grow_vec(&mut ledger.pending_head_steps, 1)?;
                ledger.pending_head_steps.push(step);
                ledger
                    .pending_batches
                    .last_mut()
                    .ok_or(Denial::SelectedFactMismatch)?
                    .head_step = Some(step);
                cursor += 1;
            }
            if cursor != verified.ordered_released_batches().len()
                || cursor != effective.ordered_replays().len()
            {
                return Err(Denial::SelectedFactMismatch);
            }
        } else if !effective.ordered_replays().is_empty() {
            return Err(Denial::SelectedFactMismatch);
        }
        if replay.effect().source_root() != ledger.effective_heads.root() {
            return Err(Denial::SelectedFactMismatch);
        }
        ledger.append_pending_batch(
            BatchEvidence {
                descriptor: verified.descriptor(),
                descriptor_record: verified.descriptor_record(),
                descriptor_frame_sha256: verified.descriptor_frame_sha256(),
                reservation_record: verified.reservation_record(),
                reservation_frame_sha256: verified.reservation_frame_sha256(),
                candidate_root_generation: root.generation(),
                candidate_root_sha256: root_sha256,
                fate: verified.wal_fate(),
            },
            resident,
        )?;
        let step = SelectedReleaseHeadStep::from_effect(replay.effect());
        ledger.effective_heads.apply_transition_admitted(
            replay.effect().source_root(),
            Some(replay.effect().result_root()),
            replay.effect().mutation(),
            resident,
        )?;
        resident.grow_vec(&mut ledger.pending_head_steps, 1)?;
        ledger.pending_head_steps.push(step);
        ledger
            .pending_batches
            .last_mut()
            .ok_or(Denial::SelectedFactMismatch)?
            .head_step = Some(step);
        if !ledger.effective_heads.matches_selected(
            Some(effective.effective_root()),
            effective.effective_heads(),
            effective.effective_digest(),
        )? {
            return Err(Denial::SelectedFactMismatch);
        }
        Ok(Self::Selected(ledger))
    }
}

struct BatchEvidence {
    descriptor: BlobReclaimDescriptorV3,
    descriptor_record: PersistedRecordIdentity,
    descriptor_frame_sha256: [u8; 32],
    reservation_record: PersistedRecordIdentity,
    reservation_frame_sha256: [u8; 32],
    fate: ReleasedDropWalFateWitnessV1,
    candidate_root_generation: u64,
    candidate_root_sha256: [u8; 32],
}

impl BatchEvidence {
    fn ordered(
        batch: &VerifiedOrderedPendingWalReleaseBatch,
        edge: &worth_store_recovery_physics::VerifiedReleasedRootEdge,
    ) -> Self {
        Self {
            descriptor: batch.descriptor(),
            descriptor_record: batch.descriptor_frame().record(),
            descriptor_frame_sha256: batch.descriptor_frame().payload_sha256(),
            reservation_record: batch.reservation_frame().record(),
            reservation_frame_sha256: batch.reservation_frame().payload_sha256(),
            fate: batch.wal_fate(),
            candidate_root_generation: edge.candidate_root_generation(),
            candidate_root_sha256: edge.result_root_frame_sha256(),
        }
    }
}

impl SelectedReleaseCustodyLedger {
    fn append_pending_batch(
        &mut self,
        batch: BatchEvidence,
        resident: &mut StoreRejoinResidentLedger,
    ) -> Result<(), Denial> {
        // Terminality belongs to this descriptor's released object. Another
        // object can append a distinct batch even when the previous tip was
        // terminal; its per-object predecessor is checked in Store rejoin.
        if self.pending_batches.len() as u64 >= MAX_CHECKPOINT_CERTIFICATE_RECORDS {
            return Err(Denial::SelectedFactMismatch);
        }
        let base = batch.descriptor.base();
        let request = batch.descriptor.custody().request();
        let tip = ReleasedDropTipProvenanceV1::new(
            batch.descriptor_record,
            batch.descriptor_frame_sha256,
            batch.reservation_record,
            batch.reservation_frame_sha256,
            request,
            batch.fate,
            batch.candidate_root_generation,
            batch.candidate_root_sha256,
        )
        .map_err(|_| Denial::SelectedFactMismatch)?;
        let evidence = ReleasedDropCumulativeEvidenceV1::new(
            batch.descriptor_record,
            batch.descriptor_frame_sha256,
            batch.descriptor.custody_digest(),
            batch.reservation_record,
            batch.reservation_frame_sha256,
            batch.fate,
            batch.candidate_root_generation,
            batch.candidate_root_sha256,
            base.predecessor(),
            base.manifest_count(),
            base.terminal(),
        )
        .map_err(|_| Denial::SelectedFactMismatch)?;
        let (cumulative_dropped, cumulative_digest) = evidence
            .advance(self.cumulative_dropped, self.cumulative_digest)
            .map_err(|_| Denial::SelectedFactMismatch)?;
        let new_records = self
            .used_records
            .checked_add(1)
            .ok_or(Denial::SelectedFactMismatch)?;
        let new_bytes = self
            .used_bytes
            .checked_add(
                u32::try_from(
                    RELEASE_CHECKPOINT_BATCH_WIRE_BYTES as u64 + CERTIFICATE_FRAME_OVERHEAD,
                )
                .map_err(|_| Denial::SelectedFactMismatch)?,
            )
            .ok_or(Denial::SelectedFactMismatch)?;
        if u64::from(new_records) > MAX_CHECKPOINT_CERTIFICATE_RECORDS
            || u64::from(new_bytes) > MAX_CHECKPOINT_CERTIFICATE_BYTES
        {
            return Err(Denial::SelectedFactMismatch);
        }
        resident.grow_vec(&mut self.pending_batches, 1)?;
        self.pending_batches.push(SelectedReleaseBatchBasis {
            head_step: None,
            descriptor_record: batch.descriptor_record,
            descriptor_frame_sha256: batch.descriptor_frame_sha256,
            custody_digest: batch.descriptor.custody_digest(),
            reservation_record: batch.reservation_record,
            reservation_frame_sha256: batch.reservation_frame_sha256,
            request,
            fate: batch.fate,
            candidate_root_generation: batch.candidate_root_generation,
            candidate_root_sha256: batch.candidate_root_sha256,
            predecessor: base.predecessor(),
            cumulative_dropped,
            cumulative_digest,
            terminal: base.terminal(),
        });
        self.used_records = new_records;
        self.used_bytes = new_bytes;
        self.cumulative_dropped = cumulative_dropped;
        self.cumulative_digest = cumulative_digest;
        self.selected_tip = Some(tip);
        self.terminal = base.terminal();
        Ok(())
    }
}
