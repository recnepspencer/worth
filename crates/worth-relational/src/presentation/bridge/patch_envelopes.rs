#[cfg(test)]
use crate::history::data::CanonicalCommitEnvelope;
use crate::history::data::{BranchId, CommitId};
use crate::publication::patch::data::{
    PublishedAspectChangePrecision, PublishedAuthoritativeAspectChange,
    PublishedAuthoritativePatchEnvelope, PublishedAuthoritativeRecordPatch, RecordStructuralChange,
};
use worth_foundational::facade::{AspectLocator, LocatorAuthority};
use worth_proof::TransitionOutcome;
use worth_runtime_bridge::facade::{
    BridgeAspectChangeWideningCause, BridgeCommittedPatchEnvelope,
    BridgeCommittedPatchEnvelopeIdentity, BridgeCommittedPatchItem, BridgeCommittedPatchTarget,
    BridgeCommittedRecordChange, BridgeCommittedRecordChangeKind, BridgeProducerMetadata,
    BridgeSemanticAspectChange, TruthBranchIdentity, TruthCommitIdentity, TruthPatchIdentity,
    TruthSnapshotIdentity,
};

#[cfg(test)]
use super::identities::bridge_snapshot_identity_for_commit;
use super::identities::record_ref_identity;
use super::patch_semantic_validation::validate_authoritative_patch_semantics;
use super::RelationalBridgePublicationDenial;

#[cfg(test)]
pub(crate) fn publication_patch_to_bridge_envelope(
    commit_id: CommitId,
    branch_id: &BranchId,
    snapshot_identity: TruthSnapshotIdentity,
    patch: &PublishedAuthoritativePatchEnvelope,
) -> TransitionOutcome<BridgeCommittedPatchEnvelope, RelationalBridgePublicationDenial> {
    publication_patch_to_bridge_envelope_with_widening(RelationalBridgePatchPublicationRequest {
        commit_id,
        branch_id,
        snapshot_identity,
        patch,
        admitted_widening: None,
        producer_metadata: BridgeProducerMetadata::bridge_harness_fixture(),
        source_record_patches_examined: patch.authoritative_record_patches.len() as u64,
        source_record_patches_filtered_out: 0,
    })
}

pub(super) struct RelationalBridgePatchPublicationRequest<'a> {
    pub(super) commit_id: CommitId,
    pub(super) branch_id: &'a BranchId,
    pub(super) snapshot_identity: TruthSnapshotIdentity,
    pub(super) patch: &'a PublishedAuthoritativePatchEnvelope,
    pub(super) admitted_widening: Option<BridgeAspectChangeWideningCause>,
    pub(super) producer_metadata: BridgeProducerMetadata,
    pub(super) source_record_patches_examined: u64,
    pub(super) source_record_patches_filtered_out: u64,
}

pub(super) fn publication_patch_to_bridge_envelope_with_widening(
    request: RelationalBridgePatchPublicationRequest<'_>,
) -> TransitionOutcome<BridgeCommittedPatchEnvelope, RelationalBridgePublicationDenial> {
    let RelationalBridgePatchPublicationRequest {
        commit_id,
        branch_id,
        snapshot_identity,
        patch,
        admitted_widening,
        producer_metadata,
        source_record_patches_examined,
        source_record_patches_filtered_out,
    } = request;
    let identity = BridgeCommittedPatchEnvelopeIdentity::new_with_metadata(
        producer_metadata,
        TruthCommitIdentity::from_relational_commit_id(commit_id.0),
        TruthPatchIdentity::from_relational_patch_position(patch.position.0),
        snapshot_identity,
        TruthBranchIdentity::from_relational_branch_id(branch_id.0.clone()),
    );
    let canonical = patch.canonicalized();
    let mut counters = match validate_authoritative_patch_semantics(&canonical, admitted_widening) {
        Ok(counters) => counters,
        Err(denial) => return TransitionOutcome::Denied(denial),
    };
    counters.source_record_patches_examined = source_record_patches_examined;
    counters.source_record_patches_filtered_out = source_record_patches_filtered_out;
    let items = bridge_patch_items(&canonical.authoritative_record_patches, admitted_widening);
    let record_changes = bridge_record_changes(&canonical.authoritative_record_patches);
    match BridgeCommittedPatchEnvelope::new_with_authoritative_lowering(
        identity,
        items,
        record_changes,
        counters,
    ) {
        Ok(envelope) => TransitionOutcome::Success(envelope),
        Err(error) => {
            TransitionOutcome::Denied(RelationalBridgePublicationDenial::new(error, counters))
        }
    }
}

#[cfg(test)]
pub(crate) fn commit_envelope_to_bridge_envelope(
    envelope: &CanonicalCommitEnvelope,
    position: crate::publication::patch::data::PatchStreamPosition,
) -> TransitionOutcome<BridgeCommittedPatchEnvelope, RelationalBridgePublicationDenial> {
    let patch = PublishedAuthoritativePatchEnvelope::from_canonical(position, &envelope.patch);
    publication_patch_to_bridge_envelope(
        envelope.commit.commit_id,
        &envelope.commit.branch_id,
        bridge_snapshot_identity_for_commit(envelope.commit.commit_id, envelope.commit.version_id),
        &patch,
    )
}

fn bridge_patch_items(
    records: &[PublishedAuthoritativeRecordPatch],
    admitted_widening: Option<BridgeAspectChangeWideningCause>,
) -> Vec<BridgeCommittedPatchItem> {
    records
        .iter()
        .flat_map(|record| {
            let record_identity = record_ref_identity(&record.target);
            record
                .semantic_changes
                .iter()
                .map(move |change| bridge_patch_item(record_identity, change, admitted_widening))
        })
        .collect()
}

fn bridge_patch_item(
    record_identity: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
    change: &PublishedAuthoritativeAspectChange,
    admitted_widening: Option<BridgeAspectChangeWideningCause>,
) -> BridgeCommittedPatchItem {
    BridgeCommittedPatchItem::with_relational_semantic_change(
        record_identity,
        bridge_patch_target(change),
        bridge_semantic_change(change, admitted_widening),
    )
}

fn bridge_patch_target(change: &PublishedAuthoritativeAspectChange) -> BridgeCommittedPatchTarget {
    let locator = AspectLocator::new(LocatorAuthority::Authoritative, change.aspect_key().clone());
    match change.field_path() {
        Some(path) => BridgeCommittedPatchTarget::entity_field_path(locator, path.clone()),
        None if matches!(
            change.kind(),
            worth_foundational::facade::AuthoritativeAspectChangeKind::RelationSourceEndpoint
                | worth_foundational::facade::AuthoritativeAspectChangeKind::RelationTargetEndpoint
        ) =>
        {
            BridgeCommittedPatchTarget::entity_relation_endpoint(locator)
        }
        None => match change.binding() {
            worth_foundational::facade::AspectBinding::StructuralRegion => {
                BridgeCommittedPatchTarget::entity_region(locator)
            }
            worth_foundational::facade::AspectBinding::StructuralPartition => {
                BridgeCommittedPatchTarget::entity_partition(locator)
            }
            worth_foundational::facade::AspectBinding::StructuralFacet => {
                BridgeCommittedPatchTarget::entity_facet(locator)
            }
            worth_foundational::facade::AspectBinding::LifecycleTransition => {
                BridgeCommittedPatchTarget::lifecycle_transition(locator)
            }
            _ => BridgeCommittedPatchTarget::authoritative_aspect(locator),
        },
    }
}

fn bridge_semantic_change(
    change: &PublishedAuthoritativeAspectChange,
    admitted_widening: Option<BridgeAspectChangeWideningCause>,
) -> BridgeSemanticAspectChange {
    match (change.precision(), change.kind(), admitted_widening) {
        (
            PublishedAspectChangePrecision::Exact,
            worth_foundational::facade::AuthoritativeAspectChangeKind::Opaque,
            Some(cause),
        ) => BridgeSemanticAspectChange::from_declared_authoritative_widening(
            change.aspect_key().clone(),
            change.aspect_identity(),
            change.contract_revision(),
            change.binding().clone(),
            change.kind(),
            change.field_path().cloned(),
            cause,
        ),
        _ => BridgeSemanticAspectChange::from_authoritative_publication(
            change.aspect_key().clone(),
            change.aspect_identity(),
            change.contract_revision(),
            change.binding().clone(),
            change.kind(),
            change.field_path().cloned(),
        ),
    }
}

fn bridge_record_changes(
    records: &[PublishedAuthoritativeRecordPatch],
) -> Vec<BridgeCommittedRecordChange> {
    records
        .iter()
        .map(|record| {
            let kind = match record.structural_change {
                RecordStructuralChange::Created => BridgeCommittedRecordChangeKind::Created,
                RecordStructuralChange::Updated => BridgeCommittedRecordChangeKind::Updated,
                RecordStructuralChange::Deleted => BridgeCommittedRecordChangeKind::Deleted,
                RecordStructuralChange::RetainedForAudit => {
                    BridgeCommittedRecordChangeKind::RetainedForAudit
                }
                RecordStructuralChange::MaterializationSuspended => {
                    BridgeCommittedRecordChangeKind::MaterializationSuspended
                }
                RecordStructuralChange::Rematerialized => {
                    BridgeCommittedRecordChangeKind::Rematerialized
                }
            };
            BridgeCommittedRecordChange::from_relational_publication(
                record_ref_identity(&record.target),
                kind,
            )
        })
        .collect()
}
