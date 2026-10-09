use super::certificate_capacity::{CheckpointCustodyState, SealedTierEpochCustodyBasis};
use super::PhysicalCurrentRootOwner;
use crate::physical_runtime::durability::{
    NamespaceDurableTierEpochRoot, PhysicalPublicationAdmissionDenial, PhysicalRetirementDenial,
    PhysicalRootPublicationIdentity, PhysicalRootPublicationTransition,
    PhysicalRootPublicationTransitionDenial, RetainedPhysicalRoot,
};
use crate::physical_runtime::{PhysicalMutationIdentity, PhysicalRootNamespaceDurabilityEvidence};
use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    durable_artifact_checksum, DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    PhysicalRecordFormatDeclaration, TierEpochActivationV1, TierEpochWalFrameWitnessV1,
};

impl PhysicalCurrentRootOwner {
    /// Seals checkpoint custody only after the root is selected and both
    /// typed maintenance frames have durable scheduler/barrier receipts.
    pub(in crate::physical_runtime) fn seal_tier_epoch_checkpoint_basis(
        &self,
        intent: TierEpochActivationV1,
        intent_frame: TierEpochWalFrameWitnessV1,
        completed_frame: TierEpochWalFrameWitnessV1,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<(), PhysicalRetirementDenial> {
        let mut state = self.lock_publication_state();
        let root_digest: [u8; 32] = Sha256::digest(state.current_root.encode(format)).into();
        if state.current_root.generation() != intent.candidate_root_generation()
            || root_digest != intent.candidate_root_sha256()
            || state.current_root.tier_epoch_anchor() != Some(intent.epoch_anchor())
            || state.free_space.tier_epoch_start() != Some(intent.tier_epoch_start())
            || !matches!(
                &state.checkpoint_custody,
                CheckpointCustodyState::VerifiedLegacyNoCertificates
            )
        {
            return Err(PhysicalRetirementDenial::WalPlan);
        }
        let basis = SealedTierEpochCustodyBasis::new(intent, intent_frame, completed_frame)
            .ok_or(PhysicalRetirementDenial::WalPlan)?;
        state.checkpoint_custody = CheckpointCustodyState::CertifiedTier(basis);
        Ok(())
    }

    pub(in crate::physical_runtime) fn register_tier_epoch_pending(
        &self,
        operation: PhysicalMutationIdentity,
    ) -> Result<
        crate::physical_runtime::durability::PendingPublicationLease,
        PhysicalPublicationAdmissionDenial,
    > {
        let state = self.lock_publication_state();
        if state.free_space.tier_epoch_start().is_some()
            || state.current_root.tier_epoch_anchor().is_some()
            || self.lock_reclaim().is_some()
        {
            return Err(PhysicalPublicationAdmissionDenial::ReclaimFenced);
        }
        self.publication.register_exclusive_pending(operation)
    }

    pub(in crate::physical_runtime) fn begin_tier_epoch_root(
        &self,
        identity: PhysicalRootPublicationIdentity,
        source: DurablePhysicalRootManifest,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<PhysicalRootPublicationTransition, PhysicalRootPublicationTransitionDenial> {
        let state = self.lock_publication_state();
        let Some((_, intent, _)) = identity.tier_epoch_basis() else {
            return Err(PhysicalRootPublicationTransitionDenial::CurrentRootMismatch);
        };
        let source_digest: [u8; 32] = Sha256::digest(state.current_root.encode(format)).into();
        let free_digest: [u8; 32] = Sha256::digest(state.free_space.encode(format)).into();
        if self.lock_reclaim().is_some()
            || state.free_space.tier_epoch_start().is_some()
            || state.current_root.tier_epoch_anchor().is_some()
            || intent.source_root_generation() != state.current_root.generation()
            || intent.source_root_sha256() != source_digest
            || intent.source_free_sha256() != free_digest
            || intent.tier_epoch_start() != state.free_space.next_arena()
        {
            return Err(PhysicalRootPublicationTransitionDenial::CurrentRootMismatch);
        }
        self.transition.begin(identity, &state.current_root, source)
    }

    pub(in crate::physical_runtime) fn advance_tier_epoch_root(
        &self,
        durable: NamespaceDurableTierEpochRoot,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<(), PhysicalRetirementDenial> {
        let mut state = self.lock_publication_state();
        let Some((operation, intent, wal_digest)) =
            durable.transition.identity().tier_epoch_basis()
        else {
            return Err(PhysicalRetirementDenial::WalPlan);
        };
        let candidate = durable.candidate.successor_root();
        let candidate_digest: [u8; 32] = Sha256::digest(candidate.encode(format)).into();
        let source_digest: [u8; 32] = Sha256::digest(state.current_root.encode(format)).into();
        let free_digest: [u8; 32] = Sha256::digest(state.free_space.encode(format)).into();
        if durable.transition.source_root() != &state.current_root
            || durable.candidate.source_root() != &state.current_root
            || intent.source_root_generation() != state.current_root.generation()
            || intent.source_root_sha256() != source_digest
            || intent.source_free_sha256() != free_digest
            || intent.candidate_root_generation() != candidate.generation()
            || intent.candidate_root_sha256() != candidate_digest
            || intent.tier_epoch_start() != state.free_space.next_arena()
            || state.current_root.tier_epoch_anchor().is_some()
            || candidate.tier_epoch_anchor() != Some(intent.epoch_anchor())
            || wal_digest != durable.receipt.payload_digest()
        {
            return Err(PhysicalRetirementDenial::WalPlan);
        }
        let expected_free = successor_free(
            &state.free_space,
            candidate.generation(),
            intent.tier_epoch_start(),
        )
        .ok_or(PhysicalRetirementDenial::WalPlan)?;
        let (source, free, root, _, _) = durable.candidate.into_root_parts();
        if free != expected_free
            || root.tree_identity() != source.tree_identity()
            || root.node_capacity() != source.node_capacity()
            || root.record_count() != source.record_count()
            || root.next_block() != source.next_block()
            || root.next_segment_block() != source.next_segment_block()
            || root.next_release_custody_head_block() != source.next_release_custody_head_block()
            || root.routing_root() != source.routing_root()
            || root.release_custody_head_root() != source.release_custody_head_root()
            || root.segment_root() != source.segment_root()
            || root.free_space_root() != source.free_space_root()
            || root.latest_blob_publication() != source.latest_blob_publication()
            || root.latest_blob_quarantine() != source.latest_blob_quarantine()
            || root.derived_family_directory() != source.derived_family_directory()
            || root.last_inline_record() != source.last_inline_record()
            || root.last_inline_segment() != source.last_inline_segment()
            || root.free_space_checksum() != durable_artifact_checksum(&free.encode(format))
            || root.tier_epoch_anchor() != Some(intent.epoch_anchor())
        {
            return Err(PhysicalRetirementDenial::WalPlan);
        }
        state.namespace_evidence = PhysicalRootNamespaceDurabilityEvidence::TierEpochCurrentRoot {
            operation,
            epoch: intent.tier_epoch_start(),
            source_generation: source.generation(),
            current_generation: root.generation(),
            replacement: durable.replacement,
            namespace_synchronization: durable.namespace,
        };
        state.previous_root = Some(RetainedPhysicalRoot::from_manifest(source));
        state.current_root = root;
        state.free_space = free;
        durable.transition.release();
        Ok(())
    }
}

fn successor_free(
    free: &DurableFreeSpaceManifestHeader,
    generation: u64,
    epoch: u64,
) -> Option<DurableFreeSpaceManifestHeader> {
    if free.tier_epoch_start().is_some() || free.next_arena() != epoch {
        return None;
    }
    DurableFreeSpaceManifestHeader::new_with_tier_epoch(
        generation,
        free.tree_identity(),
        free.node_capacity(),
        free.segment_page_capacity(),
        free.entry_count(),
        free.next_segment(),
        free.next_page(),
        free.next_extent(),
        free.next_arena(),
        Some(epoch),
        free.arena_capacity(),
        free.arena_alignment(),
        free.next_block(),
        free.root(),
    )
}
