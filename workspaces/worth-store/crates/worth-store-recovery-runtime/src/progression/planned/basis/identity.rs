use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, CurrentPhysicalRecordPlacement,
    PhysicalCheckpointIdentity, RecordArtifactFile,
};
use worth_store_recovery_physics::{
    HistoricalConsumedOperationSet, ImmutablePhysicalRedoPlan, PhysicalRedoDecisionKind,
    PhysicalRedoDecisionPrior, PhysicalRedoDecisionView, PhysicalRedoTarget,
    PhysicalRedoTargetIdentity, PhysicalSourceSelection, ReconciledOperationFates,
    RecoveryOperationFate, RecoveryPageSource,
};

use super::derivation::closeout::candidate_denial;
use super::{
    ExecutionBasisDenial, PlanningResidentAllowance, RecoveryPublicationCandidateArtifact,
    RecoveryStagingLayoutPlan,
};
use worth_store::physical_runtime::StoreRecoveryBindingFreshnessSample;

pub(super) fn plan_identity(
    store: StableStoreIdentity,
    checkpoint: PhysicalCheckpointIdentity,
    selection: &PhysicalSourceSelection,
    freshness: &StoreRecoveryBindingFreshnessSample,
    fates: &ReconciledOperationFates,
    redo: &ImmutablePhysicalRedoPlan,
    historical_consumed: &HistoricalConsumedOperationSet,
    staging: &RecoveryStagingLayoutPlan,
    validated_manifest_cleanup: Option<crate::orchestration::ValidatedManifestResidueCleanup>,
    allowance: &mut PlanningResidentAllowance,
) -> Result<[u8; 32], ExecutionBasisDenial> {
    let mut digest = Sha256::new();
    digest.update(b"worth.store.recovery.execution-plan.v2");
    digest.update(store.bytes());
    digest.update(checkpoint.sequence().get().to_le_bytes());
    digest.update(staging.source_generation.to_le_bytes());
    digest.update(staging.staging_generation.to_le_bytes());
    digest.update(selection.wal_tail().frame_count().to_le_bytes());
    digest.update(selection.wal_tail().byte_count().to_le_bytes());
    digest.update(selection.root().selected().selector().encode());
    hash_root(
        &mut digest,
        selection.root().selected().manifest(),
        selection.root().selected().selector().format(),
        allowance,
    )?;
    digest.update(freshness.sealed_basis_identity());
    digest.update(freshness.policy_identity());
    hash_freshness(&mut digest, freshness);
    hash_fates(&mut digest, fates);
    for operation in historical_consumed.operations() {
        digest.update(operation);
        digest.update(
            historical_consumed
                .group_for(operation)
                .expect("consumed operation has group"),
        );
        digest.update(
            historical_consumed
                .descriptor_for(operation)
                .expect("consumed operation has descriptor"),
        );
    }
    if let Some(cleanup) = validated_manifest_cleanup {
        digest.update(cleanup.intent().encode());
    }
    for decision in redo.resolved_decisions() {
        hash_decision(&mut digest, decision);
    }
    for action in &staging.actions {
        digest.update(action.ordinal.to_le_bytes());
        digest.update(action.destination_generation.to_le_bytes());
        hash_target(&mut digest, &action.source);
        for step in &action.steps {
            digest.update(step.operation);
            digest.update(step.record_index.to_le_bytes());
            digest.update(step.target_index.to_le_bytes());
            digest.update(step.record_lsn.to_le_bytes());
        }
    }
    for command in &staging.commands {
        digest.update(command.ordinal.to_le_bytes());
        digest.update(command.artifact.canonical_file_name().as_bytes());
        digest.update(command.offset.to_le_bytes());
        digest.update(command.payload_digest);
        digest.update((command.bytes.len() as u64).to_le_bytes());
    }
    for copy in &staging.source_copies {
        digest.update(b"source-copy");
        let record = worth_store_physical_format::PhysicalExtentCopyRecord::Intent(copy.intent());
        let frame = allowance
            .reserve::<u8>(record.encoded_bytes())
            .map_err(|denial| candidate_denial(denial.into()))?;
        let frame = record
            .encode_in_reserved(frame)
            .ok_or(ExecutionBasisDenial::Invalid)?;
        digest.update(&frame);
        let backing = PlanningResidentAllowance::vector_bytes(&frame)?;
        drop(frame);
        allowance.release(backing);
        digest.update(copy.intent_lsn().to_le_bytes());
        digest.update(copy.intent_digest());
    }
    for action in &staging.base.actions {
        digest.update([u8::from(action.is_projected())]);
        digest.update(action.ordinal().to_le_bytes());
        hash_base_placement(&mut digest, action.placement());
    }
    for action in &staging.base.segment_updates {
        let update = action.update();
        digest.update(action.ordinal().to_le_bytes());
        digest.update(update.page_cell().segment_id().get().to_le_bytes());
        digest.update(update.page().get().to_le_bytes());
        digest.update(update.page_generation().to_le_bytes());
        digest.update(update.data_generation().to_le_bytes());
        digest.update(update.data_page_count().to_le_bytes());
        digest.update(update.frame_index().to_le_bytes());
    }
    for action in &staging.base.manifests {
        digest.update(action.ordinal().to_le_bytes());
        digest.update(action.artifact().canonical_file_name().as_bytes());
        digest.update(action.coordinate().offset().to_le_bytes());
        digest.update(action.coordinate().length().to_le_bytes());
    }
    for state in &staging.base.root_states {
        digest.update(state.root_publication_allocation_bytes().to_le_bytes());
        digest.update([state.manifest_capacity_transition()]);
        digest.update(state.successor_manifest_capacity().to_le_bytes());
        for allocation in state.inline_allocations() {
            digest.update(allocation.segment().segment_id().get().to_le_bytes());
            digest.update(allocation.segment().generation().get().to_le_bytes());
            digest.update(allocation.page_capacity().to_le_bytes());
            digest.update(allocation.used_pages().to_le_bytes());
        }
        if let Some(record) = state.last_inline_record() {
            digest.update(record.allocation_epoch());
            digest.update(record.ordinal().to_le_bytes());
        }
        if let Some(segment) = state.last_inline_segment() {
            digest.update(segment.segment_id().get().to_le_bytes());
            digest.update(segment.generation().get().to_le_bytes());
        }
    }
    digest.update((staging.base.source_artifacts.len() as u64).to_le_bytes());
    for artifact in &staging.base.source_artifacts {
        hash_artifact(&mut digest, *artifact);
    }
    Ok(digest.finalize().into())
}

pub(super) fn bind_publication_candidates(
    basis: [u8; 32],
    root: &worth_store_physical_format::DurablePhysicalRootManifest,
    format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    referenced_artifacts: &[RecordArtifactFile],
    candidates: &[RecoveryPublicationCandidateArtifact],
    allowance: &mut PlanningResidentAllowance,
) -> Result<[u8; 32], ExecutionBasisDenial> {
    let mut digest = Sha256::new();
    digest.update(b"worth.store.recovery.execution-plan.publication.v1");
    digest.update(basis);
    hash_root(&mut digest, root, format, allowance)?;
    digest.update((referenced_artifacts.len() as u64).to_le_bytes());
    for artifact in referenced_artifacts {
        hash_artifact(&mut digest, *artifact);
    }
    digest.update((candidates.len() as u64).to_le_bytes());
    for candidate in candidates {
        hash_artifact(&mut digest, candidate.artifact());
        digest.update(candidate.byte_count().to_le_bytes());
        digest.update(candidate.payload_digest());
    }
    Ok(digest.finalize().into())
}

fn hash_root(
    digest: &mut Sha256,
    root: &worth_store_physical_format::DurablePhysicalRootManifest,
    format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    allowance: &mut PlanningResidentAllowance,
) -> Result<(), ExecutionBasisDenial> {
    let frame = super::publication_candidate::encoding::root_manifest(root, format, allowance)
        .map_err(candidate_denial)?;
    digest.update(&frame);
    let backing = PlanningResidentAllowance::vector_bytes(&frame)?;
    drop(frame);
    allowance.release(backing);
    Ok(())
}

fn hash_freshness(digest: &mut Sha256, freshness: &StoreRecoveryBindingFreshnessSample) {
    for operation in freshness.operations() {
        digest.update(operation.idempotency_identity());
        digest.update(operation.request_fingerprint().bytes());
        digest.update(operation.lease_issuance_generation().to_le_bytes());
        digest.update(operation.lease_expiry_generation().to_le_bytes());
        digest.update(operation.attempt_binding_identity().unwrap_or([0; 32]));
    }
    for member in freshness.wal_members() {
        digest.update(member.lsn_range().start().get().to_le_bytes());
        digest.update(member.lsn_range().end_exclusive().get().to_le_bytes());
        digest.update(member.operation_identity());
        digest.update(member.canonical_redo());
    }
}

fn hash_fates(digest: &mut Sha256, fates: &ReconciledOperationFates) {
    for fate in fates.operations() {
        let identity = fate.identity();
        digest.update(identity.store());
        digest.update(identity.runtime().to_le_bytes());
        digest.update(identity.lifecycle().to_le_bytes());
        digest.update(identity.operation().to_le_bytes());
        digest.update(identity.idempotency());
        digest.update(fate.request_fingerprint());
        digest.update([fate_tag(fate.fate())]);
    }
}

fn hash_decision(digest: &mut Sha256, decision: PhysicalRedoDecisionView<'_>) {
    let tag = match decision.kind() {
        PhysicalRedoDecisionKind::Apply => 1,
        PhysicalRedoDecisionKind::SkipPageAlreadyAtOrBeyondLsn => 2,
        PhysicalRedoDecisionKind::SkipOperationAlreadyMaterialized => 3,
        PhysicalRedoDecisionKind::SkipHistoricallyReleasedTarget => 4,
    };
    digest.update([tag]);
    digest.update(decision.operation());
    match decision.prior() {
        PhysicalRedoDecisionPrior::OperationFate(fate) => digest.update([1, fate_tag(fate)]),
        PhysicalRedoDecisionPrior::Page(prior) => {
            digest.update([2, u8::from(prior.is_absent_prior())]);
            hash_target_identity(digest, prior.target());
            digest.update(prior.page_lsn().to_le_bytes());
            digest.update(prior.frame_digest());
            hash_page_source(digest, prior.source());
        }
    }
    digest.update(decision.record().lsn().get().to_le_bytes());
    digest.update(decision.record().bytes());
    hash_target(digest, decision.target());
}

fn hash_page_source(digest: &mut Sha256, source: RecoveryPageSource) {
    let (tag, coordinate, identity) = match source {
        RecoveryPageSource::Materialized {
            coordinate,
            routing_identity,
        } => (1, coordinate, routing_identity),
        RecoveryPageSource::AbsentTarget {
            coordinate,
            root_membership_identity,
        } => (2, coordinate, root_membership_identity),
        RecoveryPageSource::PlannedResult {
            coordinate,
            causal_identity,
        } => (3, coordinate, causal_identity),
        RecoveryPageSource::HistoricalReleasedDrop {
            coordinate,
            selected_root_identity,
            descriptor_operation,
            old_operation,
            wal_target_digest,
        } => {
            digest.update(descriptor_operation);
            digest.update(old_operation);
            digest.update(wal_target_digest);
            (4, coordinate, selected_root_identity)
        }
    };
    digest.update([tag]);
    digest.update(coordinate.artifact().canonical_file_name().as_bytes());
    digest.update(coordinate.offset().to_le_bytes());
    digest.update(coordinate.length().to_le_bytes());
    digest.update(identity);
}

fn fate_tag(fate: RecoveryOperationFate) -> u8 {
    match fate {
        RecoveryOperationFate::AcknowledgedDurable => 1,
        RecoveryOperationFate::DurableUnacknowledged => 2,
        RecoveryOperationFate::ProvenNoEffect => 3,
        RecoveryOperationFate::Indeterminate => 4,
    }
}

fn hash_target(digest: &mut Sha256, target: &PhysicalRedoTarget) {
    hash_target_identity(digest, target.identity());
    hash_artifact(digest, target.artifact());
    if let Some(coordinate) = target.extent_coordinate() {
        digest.update(coordinate.allocation_epoch());
        digest.update(coordinate.record_ordinal().to_le_bytes());
        digest.update(coordinate.logical_bytes().to_le_bytes());
        digest.update(coordinate.logical_offset().to_le_bytes());
    }
    digest.update(target.artifact_offset().to_le_bytes());
    digest.update(target.artifact_length().to_le_bytes());
    digest.update(target.resulting_digest());
}

fn hash_artifact(digest: &mut Sha256, artifact: RecordArtifactFile) {
    let name = artifact.canonical_file_name();
    digest.update((name.as_bytes().len() as u64).to_le_bytes());
    digest.update(name.as_bytes());
}

fn hash_target_identity(digest: &mut Sha256, identity: PhysicalRedoTargetIdentity) {
    match identity {
        PhysicalRedoTargetIdentity::InlinePage {
            segment,
            page,
            generation,
        } => {
            digest.update([1]);
            digest.update(segment.to_le_bytes());
            digest.update(page.to_le_bytes());
            digest.update(generation.to_le_bytes());
        }
        PhysicalRedoTargetIdentity::ExtentChunk {
            extent,
            generation,
            chunk,
        } => {
            digest.update([2]);
            digest.update(extent.to_le_bytes());
            digest.update(generation.to_le_bytes());
            digest.update(chunk.to_le_bytes());
        }
    }
}

fn hash_base_placement(digest: &mut Sha256, placement: CurrentPhysicalRecordPlacement) {
    digest.update(placement.record().allocation_epoch());
    digest.update(placement.record().ordinal().to_le_bytes());
    match placement {
        CurrentPhysicalRecordPlacement::Inline(inline) => {
            digest.update([1]);
            digest.update(inline.segment().get().to_le_bytes());
            digest.update(inline.segment_generation().to_le_bytes());
            digest.update(inline.page().get().to_le_bytes());
            digest.update(inline.page_generation().to_le_bytes());
            digest.update(inline.slot().get().to_le_bytes());
            digest.update(inline.slot_generation().to_le_bytes());
            digest.update(inline.segment_page_capacity().to_le_bytes());
            digest.update(inline.payload_bytes().to_le_bytes());
        }
        CurrentPhysicalRecordPlacement::Extent(extent) => {
            digest.update([2]);
            digest.update(extent.extent().get().to_le_bytes());
            digest.update(extent.extent_generation().to_le_bytes());
            digest.update(extent.payload_bytes().to_le_bytes());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};

    #[test]
    fn sealed_source_coordinate_and_routing_identity_are_plan_identity_causal() {
        let first = source_digest(1, 0, [1; 32]);
        assert_ne!(first, source_digest(2, 0, [1; 32]));
        assert_ne!(first, source_digest(1, 4096, [1; 32]));
        assert_ne!(first, source_digest(1, 0, [2; 32]));
    }

    fn source_digest(generation: u64, offset: u64, routing: [u8; 32]) -> [u8; 32] {
        let coordinate = RecordFrameCoordinate::new(
            RecordArtifactFile::Segment {
                segment: 1,
                generation,
            },
            offset,
            4096,
        )
        .unwrap();
        let mut digest = Sha256::new();
        hash_page_source(
            &mut digest,
            RecoveryPageSource::Materialized {
                coordinate,
                routing_identity: routing,
            },
        );
        digest.finalize().into()
    }
}
