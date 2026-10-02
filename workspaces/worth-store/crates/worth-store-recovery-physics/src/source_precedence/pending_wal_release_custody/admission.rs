use super::*;

impl VerifiedPendingWalReleaseCustody {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn admit_with_base(
        selected: &PhysicalSourceSelection,
        base_custody: PendingReleaseCheckpointBase,
        tier: Option<&VerifiedSelectedTierEpochCustody>,
        projection: &PhysicalRedoProjection,
        redo: &ImmutablePhysicalRedoPlan,
        reservation_frame: &WitnessedSelectedControlFrame,
        manifest_frame: &WitnessedSelectedControlFrame,
        wal_fate: ReleasedDropWalFateWitnessV1,
        member_redo_digest: [u8; 32],
        fates: &ReconciledOperationFates,
        policy: [u8; 32],
        selected_head_replay: Option<VerifiedSelectedReleaseHeadReplayV14>,
    ) -> Result<Self, PendingWalReleaseCustodyDenial> {
        let source_root = selected.root().selected().manifest().clone();
        use PendingWalReleaseCustodyDenial as Denial;
        let checkpoint = selected.checkpoint().ok_or(Denial::CheckpointMarker)?;
        let source = selected.root().selected();
        if !redo.admits_exact_member_redo_digest(projection, member_redo_digest) {
            return Err(Denial::DurableWalFate);
        }
        let source_root_sha256: [u8; 32] =
            Sha256::digest(source_root.encode(source.selector().format())).into();
        let checkpoint_matches = match &base_custody {
            PendingReleaseCheckpointBase::NoRelease(marker) => {
                marker.checkpoint() == checkpoint.checkpoint().source().identity()
                    && marker.root_sha256() == checkpoint.source_root_frame_sha256()
            }
            PendingReleaseCheckpointBase::Released(base) => {
                base.selected_root() == &source_root
                    && base.selected_root_sha256() == source_root_sha256
                    && base.checkpoint().source() == checkpoint.checkpoint().source()
                    && base.checkpoint().encoded_bytes() == checkpoint.checkpoint().encoded_bytes()
                    && base.checkpoint().encoded_digest()
                        == checkpoint.checkpoint().encoded_digest()
                    && base.source_root_sha256() == checkpoint.source_root_frame_sha256()
            }
            PendingReleaseCheckpointBase::ReleasedAddressed(base) => {
                base.checkpoint_root_frame_sha256() == checkpoint.source_root_frame_sha256()
                    && base.checkpoint().source() == checkpoint.checkpoint().source()
                    && base.checkpoint().encoded_bytes() == checkpoint.checkpoint().encoded_bytes()
                    && base.checkpoint().encoded_digest()
                        == checkpoint.checkpoint().encoded_digest()
            }
            PendingReleaseCheckpointBase::ReleasedHeadV2(base) => {
                base.source_root_sha256() == checkpoint.source_root_frame_sha256()
                    && base.checkpoint().source() == checkpoint.checkpoint().source()
                    && base.checkpoint().encoded_bytes() == checkpoint.checkpoint().encoded_bytes()
                    && base.checkpoint().encoded_digest()
                        == checkpoint.checkpoint().encoded_digest()
                    && base.selected_root() == &source_root
                    && base.selected_root_sha256() == source_root_sha256
            }
        };
        let tier_matches = match (source_root.tier_epoch_anchor(), tier) {
            (None, None) => true,
            (Some(anchor), Some(tier)) => {
                anchor == tier.tier_epoch_anchor()
                    && tier.selected_root() == &source_root
                    && tier.selected_root_sha256() == source_root_sha256
                    && tier.checkpoint().source().identity()
                        == checkpoint.checkpoint().source().identity()
                    && tier.checkpoint().encoded_digest()
                        == checkpoint.checkpoint().encoded_digest()
                    && tier.checkpoint().encoded_bytes() == checkpoint.checkpoint().encoded_bytes()
                    && tier.checkpoint_source_root_sha256() == checkpoint.source_root_frame_sha256()
            }
            _ => false,
        };
        if !tier_matches
            || !checkpoint_matches
            || matches!(&base_custody, PendingReleaseCheckpointBase::NoRelease(marker)
                if marker.root_generation() > source_root.generation())
            || source.manifest() != &source_root
        {
            return Err(Denial::CheckpointMarker);
        }
        let PersistedPhysicalRecoveryOperation::RecordsDropped { binding, .. } =
            projection.materialization().operation()
        else {
            return Err(Denial::DurableWalFate);
        };
        let descriptor_bytes = redo
            .blob_semantic_record_bytes(projection.operation())
            .ok_or(Denial::DurableWalFate)?;
        let BlobRecordV1::ReclaimDescriptorV3(descriptor) =
            decode_blob_record(descriptor_bytes).map_err(|_| Denial::ControlBinding)?
        else {
            return Err(Denial::ControlBinding);
        };
        let base = descriptor.base();
        if let Some(replay) = selected_head_replay.as_ref() {
            if !matches!(
                base_custody,
                PendingReleaseCheckpointBase::ReleasedHeadV2(_)
                    | PendingReleaseCheckpointBase::NoRelease(_)
            ) || replay.operation() != projection.operation()
                || replay.group() != projection.group()
                || replay.fate() != projection.fate()
                || replay.canonical_redo_sha256() != member_redo_digest
                || replay.effect().source_root() != source_root.release_custody_head_root()
                || replay.effect().source_next_block()
                    != source_root.next_release_custody_head_block()
            {
                return Err(Denial::SourceBinding);
            }
        } else if matches!(
            base_custody,
            PendingReleaseCheckpointBase::ReleasedHeadV2(_)
        ) || matches!(
            projection.materialization().operation(),
            PersistedPhysicalRecoveryOperation::RecordsDropped {
                head_effect: Some(_),
                ..
            }
        ) {
            return Err(Denial::SourceBinding);
        }
        if descriptor.encode() != descriptor_bytes
            || binding.record_payload_sha256() != <[u8; 32]>::from(Sha256::digest(descriptor_bytes))
            || base.store() != source.selector().store_identity().bytes()
            || base.source_kind() != BlobReclaimSourceKind::ReleasedGeneration
            || base.source_root_generation() != source_root.generation()
            || base.candidate_root_generation() != binding.candidate_root_generation()
            || source_root.generation().checked_add(1) != Some(base.candidate_root_generation())
            || (matches!(&base_custody, PendingReleaseCheckpointBase::NoRelease(_))
                && base.predecessor().is_some())
            || descriptor.custody().source_root_frame_sha256() != source_root_sha256
            || projection.materialization().source_root_generation() != source_root.generation()
            || projection.operation() != descriptor.custody().request().idempotency()
        {
            return Err(Denial::SourceBinding);
        }
        if let Some(replay) = selected_head_replay.as_ref() {
            let worth_store_physical_format::ReleaseCustodyHeadMutationV1::Upsert { next, .. } =
                replay.effect().mutation()
            else {
                return Err(Denial::ControlBinding);
            };
            if next.descriptor_record() != binding.record()
                || next.descriptor_frame_sha256() != binding.record_payload_sha256()
                || next.manifest_record() != base.manifest_record()
                || next.manifest_frame_sha256() != base.manifest_frame_sha256()
            {
                return Err(Denial::ControlBinding);
            }
        }
        let reservation_bytes = reservation_frame
            .selected(selected, BlobRecordKind::OriginalDropReserved)
            .map_err(|_| Denial::ControlBinding)?;
        let manifest_bytes = manifest_frame
            .selected(selected, BlobRecordKind::DropSetManifestV3)
            .map_err(|_| Denial::ControlBinding)?;
        let BlobRecordV1::OriginalDropReserved(reservation) =
            decode_blob_record(reservation_bytes).map_err(|_| Denial::ControlBinding)?
        else {
            return Err(Denial::ControlBinding);
        };
        let BlobRecordV1::DropSetManifestV3(manifest) =
            decode_blob_record(manifest_bytes).map_err(|_| Denial::ControlBinding)?
        else {
            return Err(Denial::ControlBinding);
        };
        verify_controls(
            descriptor,
            reservation,
            &manifest,
            reservation_frame,
            manifest_frame,
            match &base_custody {
                PendingReleaseCheckpointBase::NoRelease(_) => None,
                PendingReleaseCheckpointBase::Released(base) => Some(base.as_ref()),
                PendingReleaseCheckpointBase::ReleasedAddressed(_) => None,
                PendingReleaseCheckpointBase::ReleasedHeadV2(_) => None,
            },
            match &base_custody {
                PendingReleaseCheckpointBase::ReleasedAddressed(base) => Some(base.as_ref()),
                _ => None,
            },
            selected_head_replay.as_ref(),
        )?;
        let operation_fate = verify_fate(
            descriptor,
            wal_fate,
            fates,
            policy,
            projection.operation(),
            checkpoint
                .checkpoint()
                .compaction_cutover()
                .wal_cutoff_lsn_exclusive(),
        )?;
        if operation_fate != projection.fate() {
            return Err(Denial::DurableWalFate);
        }
        Ok(Self {
            checkpoint: *checkpoint.checkpoint(),
            base: base_custody,
            checkpoint_source_root_sha256: checkpoint.source_root_frame_sha256(),
            source_root,
            source_root_sha256,
            published_root: None,
            published_root_sha256: None,
            verified_transition: None,
            historical_batches: Box::new([]),
            ordered_history: None,
            ordered_released_batches: Box::new([]),
            descriptor_record: binding.record(),
            descriptor_frame_sha256: binding.record_payload_sha256(),
            descriptor,
            reservation_record: reservation_frame.selected_placement().record(),
            reservation_frame_sha256: reservation_frame.selected_payload_sha256(),
            manifest_record: manifest_frame.selected_placement().record(),
            manifest_frame_sha256: manifest_frame.selected_payload_sha256(),
            wal_fate,
            member_group: projection.group(),
            member_redo_digest,
            operation_fate,
            selected_head_replay,
            prepared_effective_heads: None,
        })
    }
}
