use crate::facade::{
    BridgeAspectChangeWideningCause, BridgeCommittedPatchEnvelope,
    BridgeCommittedPatchEnvelopeIdentity, BridgeCommittedPatchItem, BridgeCommittedPatchTarget,
    BridgeCommittedRecordChange, BridgeCommittedRecordChangeKind, BridgeProducerMetadata,
    BridgeRouteError, BridgeRouteErrorKind, BridgeSemanticAspectChange, TruthBranchIdentity,
    TruthCommitIdentity, TruthPatchIdentity, TruthSnapshotIdentity,
};
use worth_foundational::facade::{AspectLocator, LocatorAuthority};
#[cfg(test)]
use worth_proof::TransitionOutcome;
use worth_relational::facade::history::{BranchId, CommitId};
use worth_relational::facade::publication::{
    PublishedAspectChangePrecision, PublishedAuthoritativeAspectChange,
    PublishedAuthoritativePatchEnvelope, PublishedAuthoritativeRecordPatch, RecordStructuralChange,
};

use super::identities::record_ref_identity;
use super::lowering_precision::gate_lowering_precision;
use super::patch_lowering_denial::PatchLoweringDenial;
use super::RelationalBridgePublicationDenial;
use worth_execution::ExecutionRequest;

/// Lower a decoded publication the way a receipt is lowered: Relational's
/// consistency rules first, then the Bridge's own gates.
#[cfg(test)]
pub(crate) fn publication_patch_to_bridge_envelope(
    commit_id: CommitId,
    branch_id: &BranchId,
    snapshot_identity: TruthSnapshotIdentity,
    patch: &PublishedAuthoritativePatchEnvelope,
) -> TransitionOutcome<BridgeCommittedPatchEnvelope, RelationalBridgePublicationDenial> {
    let patch = patch.canonicalized();
    if let Err(denial) = patch.check_change_consistency() {
        return TransitionOutcome::Denied(super::lowering_precision::consistency_denial(&denial));
    }
    crate::host_execution::with_declared_request(
        crate::policy::BridgeExecutionPolicyBaseline::operational(),
        |execution| match lower_canonical_patch(
            RelationalBridgePatchPublicationRequest {
                commit_id,
                branch_id,
                snapshot_identity,
                patch: &patch,
                admitted_widening: None,
                producer_metadata: BridgeProducerMetadata::bridge_harness_fixture(),
                source_record_patches_examined: patch.authoritative_record_patches.len() as u64,
                source_record_patches_filtered_out: 0,
            },
            execution,
        ) {
            Ok(envelope) => TransitionOutcome::Success(envelope),
            Err(PatchLoweringDenial::Publication(denial)) => TransitionOutcome::Denied(denial),
            Err(PatchLoweringDenial::Execution(denial)) => panic!("test host refused: {denial:?}"),
        },
    )
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

/// Lower one canonical patch into a Bridge envelope under an optional
/// admitted widening.
pub(super) fn lower_canonical_patch(
    request: RelationalBridgePatchPublicationRequest<'_>,
    execution: ExecutionRequest<'_, '_>,
) -> Result<BridgeCommittedPatchEnvelope, PatchLoweringDenial> {
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
    let mut counters = gate_lowering_precision(patch, admitted_widening, execution)?;
    counters.source_record_patches_examined = source_record_patches_examined;
    counters.source_record_patches_filtered_out = source_record_patches_filtered_out;
    let record_changes =
        bridge_record_changes(&patch.authoritative_record_patches, execution, counters)?;
    let items = bridge_patch_items(
        &patch.authoritative_record_patches,
        admitted_widening,
        execution,
    )?;
    match BridgeCommittedPatchEnvelope::new_with_authoritative_lowering(
        identity,
        items,
        record_changes,
        counters,
    ) {
        Ok(envelope) => Ok(envelope),
        Err(error) => Err(RelationalBridgePublicationDenial::new(error, counters).into()),
    }
}

fn bridge_patch_items(
    records: &[PublishedAuthoritativeRecordPatch],
    admitted_widening: Option<BridgeAspectChangeWideningCause>,
    execution: ExecutionRequest<'_, '_>,
) -> Result<Vec<BridgeCommittedPatchItem>, PatchLoweringDenial> {
    let mut items = Vec::new();
    for record in records {
        execution.consult()?;
        let identity = record_ref_identity(&record.target);
        for change in &record.semantic_changes {
            execution.consult()?;
            items.push(bridge_patch_item(identity, change, admitted_widening));
        }
    }
    Ok(items)
}

fn bridge_patch_item(
    record_identity: crate::facade::RelationalBridgeRecordIdentityParts,
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

/// Lower each record's structural change. Relational may add structural
/// changes the Bridge does not know yet; such a patch is denied, not guessed.
fn bridge_record_changes(
    records: &[PublishedAuthoritativeRecordPatch],
    execution: ExecutionRequest<'_, '_>,
    counters: crate::facade::BridgeAuthoritativePatchLoweringCounters,
) -> Result<Vec<BridgeCommittedRecordChange>, PatchLoweringDenial> {
    records
        .iter()
        .map(|record| {
            execution.consult()?;
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
                // `RecordStructuralChange` is non-exhaustive outside Relational.
                unknown => {
                    return Err(RelationalBridgePublicationDenial::new(
                        BridgeRouteError::new(
                            BridgeRouteErrorKind::InvalidLoweringContract,
                            format!("record structural change {unknown:?} has no Bridge lowering"),
                        ),
                        counters,
                    )
                    .into())
                }
            };
            Ok(BridgeCommittedRecordChange::from_relational_publication(
                record_ref_identity(&record.target),
                kind,
            ))
        })
        .collect()
}
