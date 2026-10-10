use crate::facade::RelationalBridgeSourceError;
use std::sync::Arc;
use worth_execution::ExecutionRequest;

use crate::facade::{
    BridgeAspectChangeWideningCause, BridgeAuthoritativeSourceProvenance, BridgeProducerMetadata,
    TruthSnapshotIdentity,
};
use worth_foundational::facade::TruthPartitionRole;
use worth_proof::TransitionOutcome;

use super::lowering_precision::consistency_denial;
use super::patch_envelopes::{lower_canonical_patch, RelationalBridgePatchPublicationRequest};
use super::publication_outcome::{
    RelationalBridgePatchPublication, RelationalBridgePublicationDeferred,
    RelationalBridgePublicationOutcome, RelationalBridgePublicationStale,
};
use worth_relational::facade::change_source::{
    RelationalChangeReceipt, RelationalChangeReceiptDeferred, RelationalChangeReceiptOutcome,
    RelationalChangeReceiptStale,
};

/// Runtime-affine admission for the one supported loss of precision. The
/// opaque token cannot be assembled from a widening label or copied fields.
pub struct RelationalOpaqueAspectWideningAdmission {
    runtime_instance_id: u64,
    graph_role: Arc<str>,
    cause: BridgeAspectChangeWideningCause,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationalOpaqueAspectWideningAdmissionDenial {
    InvalidGraphRole,
}

impl RelationalOpaqueAspectWideningAdmission {
    pub(super) fn admit(
        runtime_instance_id: u64,
        graph_role: Arc<str>,
    ) -> Result<Self, RelationalOpaqueAspectWideningAdmissionDenial> {
        if graph_role.is_empty()
            || graph_role.trim() != graph_role.as_ref()
            || graph_role.chars().any(char::is_whitespace)
        {
            return Err(RelationalOpaqueAspectWideningAdmissionDenial::InvalidGraphRole);
        }
        Ok(Self {
            runtime_instance_id,
            graph_role,
            cause: BridgeAspectChangeWideningCause::OpaquePayloadToWholeAspect,
        })
    }

    pub(super) fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }

    pub(super) fn graph_role(&self) -> &Arc<str> {
        &self.graph_role
    }

    pub(super) fn cause(&self) -> BridgeAspectChangeWideningCause {
        self.cause
    }
}

/// The Bridge concepts a receipt is lowered under.
pub(super) struct ChangeLoweringContext<'a> {
    pub(super) graph_role: &'a Arc<str>,
    pub(super) partition_role: Option<&'a TruthPartitionRole>,
    pub(super) widening: Option<BridgeAspectChangeWideningCause>,
}

/// Lower a minted receipt, or carry its outcome family across unchanged.
pub(super) fn lower_change_receipt_outcome(
    outcome: RelationalChangeReceiptOutcome,
    snapshot_identity: TruthSnapshotIdentity,
    context: &ChangeLoweringContext<'_>,
    execution: ExecutionRequest<'_, '_>,
) -> Result<RelationalBridgePublicationOutcome, RelationalBridgeSourceError> {
    Ok(match outcome {
        TransitionOutcome::Success(receipt) => {
            return lower_change_receipt(receipt, snapshot_identity, context, execution);
        }
        TransitionOutcome::Denied(denial) => TransitionOutcome::Denied(consistency_denial(&denial)),
        TransitionOutcome::Deferred(RelationalChangeReceiptDeferred::CommitVisibilityPending) => {
            TransitionOutcome::Deferred(
                RelationalBridgePublicationDeferred::CommitVisibilityPending,
            )
        }
        TransitionOutcome::Stale(RelationalChangeReceiptStale::RuntimeAuthority) => {
            TransitionOutcome::Stale(RelationalBridgePublicationStale::RuntimeAuthority)
        }
        TransitionOutcome::Stale(RelationalChangeReceiptStale::CommitNotRetained) => {
            TransitionOutcome::Stale(RelationalBridgePublicationStale::CommitNotRetained)
        }
        TransitionOutcome::RebindRequired(never) => match never {},
        TransitionOutcome::Failed(never) => match never {},
    })
}

/// The only way to mint a committed-patch envelope with Relational
/// provenance: consume one receipt.
fn lower_change_receipt(
    receipt: RelationalChangeReceipt,
    snapshot_identity: TruthSnapshotIdentity,
    context: &ChangeLoweringContext<'_>,
    execution: ExecutionRequest<'_, '_>,
) -> Result<RelationalBridgePublicationOutcome, RelationalBridgeSourceError> {
    let source_basis = change_source_basis(&receipt, context);
    let adapter_semantic_identity =
        super::identities::relational_bridge_adapter_semantic_identity();
    let provenance = source_provenance(
        receipt.runtime_instance_id(),
        context,
        adapter_semantic_identity.clone(),
        source_basis.clone(),
    );
    let outcome = lower_canonical_patch(
        RelationalBridgePatchPublicationRequest {
            commit_id: receipt.commit_id(),
            branch_id: receipt.selected_branch_id(),
            snapshot_identity,
            patch: receipt.patch(),
            admitted_widening: context.widening,
            producer_metadata: BridgeProducerMetadata::registered_authoritative_source()
                .with_authoritative_source(provenance),
            source_record_patches_examined: receipt.records_examined(),
            source_record_patches_filtered_out: receipt.records_filtered_out(),
        },
        execution,
    );
    match outcome {
        Ok(envelope) => Ok(TransitionOutcome::Success(
            RelationalBridgePatchPublication::mint(
                &receipt,
                envelope,
                context,
                adapter_semantic_identity,
                source_basis,
            ),
        )),
        Err(denial) => denial.into_outcome(),
    }
}

/// The provenance source basis. It interleaves the receipt's Relational
/// fields with the Bridge's graph role and truth partition, byte for byte as
/// the adapter has always written it.
fn change_source_basis(
    receipt: &RelationalChangeReceipt,
    context: &ChangeLoweringContext<'_>,
) -> Arc<str> {
    Arc::from(format!(
        "runtime={};commit={};version={};selected-branch={};authoring-branch={};graph-role={};relational-partition={};truth-partition={}",
        receipt.runtime_instance_id(),
        receipt.commit_id().0,
        receipt.version_id().0,
        receipt.selected_branch_id().0,
        receipt.authoring_branch_id().0,
        context.graph_role,
        receipt
            .partition_id()
            .map_or_else(|| "all".to_string(), |partition| partition.as_u32().to_string()),
        context.partition_role.map_or("all", TruthPartitionRole::as_str),
    ))
}

fn source_provenance(
    runtime_instance_id: u64,
    context: &ChangeLoweringContext<'_>,
    adapter_semantic_identity: Arc<str>,
    source_basis: Arc<str>,
) -> BridgeAuthoritativeSourceProvenance {
    match context.partition_role {
        Some(partition_role) => {
            BridgeAuthoritativeSourceProvenance::from_owner_partition_publication(
                runtime_instance_id,
                context.graph_role.clone(),
                adapter_semantic_identity,
                source_basis,
                partition_role.clone(),
            )
        }
        None => BridgeAuthoritativeSourceProvenance::from_owner_publication(
            runtime_instance_id,
            context.graph_role.clone(),
            adapter_semantic_identity,
            source_basis,
        ),
    }
}
