use worth_proof::TransitionOutcome;
use worth_runtime_bridge::facade::{
    BridgeAspectChangeWideningCause, BridgeAuthoritativePatchLoweringCounters,
};

use super::authoritative_publication_witness::{
    admit_lowered_publication, begin_publication, resolve_publication, PublicationUnresolvedRecipe,
    RelationalPublicationBasis, RelationalPublicationRequest,
};
use super::publication_outcome::{
    RelationalBridgePatchPublication, RelationalBridgePublicationDeferred,
    RelationalBridgePublicationDenial, RelationalBridgePublicationOutcome,
    RelationalBridgePublicationRebindRequired, RelationalBridgePublicationStale,
};
use crate::capabilities::CommitEnvelopeSource;
use crate::runtime::RelationalRuntime;

/// Runtime-affine admission for the one supported loss of precision. The
/// opaque token cannot be assembled from a widening label or copied fields.
pub struct RelationalOpaqueAspectWideningAdmission {
    runtime_instance_id: u64,
    graph_role: std::sync::Arc<str>,
    cause: BridgeAspectChangeWideningCause,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationalOpaqueAspectWideningAdmissionDenial {
    InvalidGraphRole,
}

use super::runtime_source::RelationalBridgeSelectedCommitObservation;

impl RelationalRuntime {
    pub fn admit_opaque_aspect_bridge_widening(
        &self,
        graph_role: impl Into<std::sync::Arc<str>>,
    ) -> Result<
        RelationalOpaqueAspectWideningAdmission,
        RelationalOpaqueAspectWideningAdmissionDenial,
    > {
        let graph_role = graph_role.into();
        if graph_role.is_empty()
            || graph_role.trim() != graph_role.as_ref()
            || graph_role.chars().any(char::is_whitespace)
        {
            return Err(RelationalOpaqueAspectWideningAdmissionDenial::InvalidGraphRole);
        }
        Ok(RelationalOpaqueAspectWideningAdmission {
            runtime_instance_id: self.runtime_instance_id(),
            graph_role,
            cause: BridgeAspectChangeWideningCause::OpaquePayloadToWholeAspect,
        })
    }

    pub(in crate::presentation::bridge) fn publish_commit_for_bridge_with_widening_at_observation(
        &self,
        selected_commit: RelationalBridgeSelectedCommitObservation,
        graph_role: impl Into<std::sync::Arc<str>>,
        admission: &RelationalOpaqueAspectWideningAdmission,
    ) -> RelationalBridgePublicationOutcome {
        if admission.runtime_instance_id != self.runtime_instance_id() {
            return TransitionOutcome::Stale(RelationalBridgePublicationStale::RuntimeAuthority);
        }
        let graph_role = graph_role.into();
        if admission.graph_role != graph_role {
            return TransitionOutcome::RebindRequired(
                RelationalBridgePublicationRebindRequired::GraphRole,
            );
        }
        self.publish_commit_for_bridge_inner(
            graph_role,
            None,
            Some(admission.cause),
            selected_commit,
        )
    }

    pub(in crate::presentation::bridge) fn publish_commit_for_bridge_graph_role_at_observation(
        &self,
        selected_commit: RelationalBridgeSelectedCommitObservation,
        graph_role: std::sync::Arc<str>,
    ) -> RelationalBridgePublicationOutcome {
        self.publish_commit_for_bridge_inner(graph_role, None, None, selected_commit)
    }

    pub(in crate::presentation::bridge) fn publish_commit_for_bridge_graph_partition_at_observation(
        &self,
        selected_commit: RelationalBridgeSelectedCommitObservation,
        graph_role: std::sync::Arc<str>,
        relational_partition_id: crate::identity::data::PartitionId,
        partition_role: worth_foundational::facade::TruthPartitionRole,
    ) -> RelationalBridgePublicationOutcome {
        self.publish_commit_for_bridge_inner(
            graph_role,
            Some((relational_partition_id, partition_role)),
            None,
            selected_commit,
        )
    }

    fn publish_commit_for_bridge_inner(
        &self,
        graph_role: std::sync::Arc<str>,
        partition: Option<(
            crate::identity::data::PartitionId,
            worth_foundational::facade::TruthPartitionRole,
        )>,
        admitted_widening: Option<BridgeAspectChangeWideningCause>,
        selected_commit: RelationalBridgeSelectedCommitObservation,
    ) -> RelationalBridgePublicationOutcome {
        let (commit_id, snapshot_identity, selected_branch) = selected_commit.into_parts();
        let unresolved = begin_publication(RelationalPublicationRequest {
            runtime_instance_id: self.runtime_instance_id(),
            commit_id,
            graph_role,
            relational_partition_id: partition.as_ref().map(|(partition, _)| *partition),
            partition_role: partition.map(|(_, role)| role),
            widening: admitted_widening,
        });
        if unresolved.payload().graph_role.trim().is_empty() {
            return TransitionOutcome::Denied(RelationalBridgePublicationDenial::new(
                worth_runtime_bridge::facade::BridgeRouteError::new(
                    worth_runtime_bridge::facade::BridgeRouteErrorKind::UnsupportedProducerEnvelope,
                    "Relational Bridge publication requires an explicit logical graph role",
                ),
                BridgeAuthoritativePatchLoweringCounters::default(),
            ));
        }
        let Some(envelope) = self.commit_envelope(unresolved.payload().commit_id) else {
            return if unresolved.payload().commit_id >= self.history().next_commit_id() {
                TransitionOutcome::Deferred(
                    RelationalBridgePublicationDeferred::CommitVisibilityPending,
                )
            } else {
                TransitionOutcome::Stale(RelationalBridgePublicationStale::CommitNotRetained)
            };
        };
        let Some(position) = self
            .history()
            .canonical_stream_position(envelope.commit.commit_id)
        else {
            return TransitionOutcome::Deferred(
                RelationalBridgePublicationDeferred::CommitVisibilityPending,
            );
        };
        self.lower_retained_publication(
            unresolved,
            &envelope,
            position,
            snapshot_identity,
            selected_branch,
        )
    }

    fn lower_retained_publication(
        &self,
        unresolved: PublicationUnresolvedRecipe,
        envelope: &crate::history::data::CanonicalCommitEnvelope,
        patch_position: crate::publication::patch::data::PatchStreamPosition,
        snapshot_identity: worth_runtime_bridge::facade::TruthSnapshotIdentity,
        selected_branch: crate::history::data::BranchId,
    ) -> RelationalBridgePublicationOutcome {
        let source_basis = publication_source_basis(
            self.runtime_instance_id(),
            &unresolved,
            envelope,
            &selected_branch,
        );
        let adapter_semantic_identity =
            super::identities::relational_bridge_adapter_semantic_identity();
        let resolved = resolve_publication(
            unresolved,
            RelationalPublicationBasis {
                version_id: envelope.commit.version_id.0,
                branch_id: std::sync::Arc::from(selected_branch.0.clone()),
                adapter_semantic_identity: adapter_semantic_identity.clone(),
                source_basis: source_basis.clone(),
            },
        );
        let source = publication_source_provenance(
            self.runtime_instance_id(),
            resolved.payload(),
            adapter_semantic_identity,
            source_basis,
        );
        let metadata =
            worth_runtime_bridge::facade::BridgeProducerMetadata::registered_authoritative_source()
                .with_authoritative_source(source);
        let patch =
            crate::publication::patch::data::PublishedAuthoritativePatchEnvelope::from_canonical(
                patch_position,
                &envelope.patch,
            );
        let projection = super::partition_projection::project_patch_partition(
            &patch,
            resolved.payload().relational_partition_id,
        );
        let outcome = super::patch_envelopes::publication_patch_to_bridge_envelope_with_widening(
            super::patch_envelopes::RelationalBridgePatchPublicationRequest {
                commit_id: envelope.commit.commit_id,
                branch_id: &selected_branch,
                snapshot_identity,
                patch: &projection.patch,
                admitted_widening: resolved.payload().widening,
                producer_metadata: metadata,
                source_record_patches_examined: projection.records_examined,
                source_record_patches_filtered_out: projection.records_filtered_out,
            },
        );
        match outcome {
            TransitionOutcome::Success(envelope) => {
                TransitionOutcome::Success(RelationalBridgePatchPublication::mint(
                    admit_lowered_publication(resolved),
                    envelope,
                ))
            }
            TransitionOutcome::Denied(denial) => TransitionOutcome::Denied(denial),
        }
    }
}

fn publication_source_basis(
    runtime_instance_id: u64,
    unresolved: &PublicationUnresolvedRecipe,
    envelope: &crate::history::data::CanonicalCommitEnvelope,
    selected_branch: &crate::history::data::BranchId,
) -> std::sync::Arc<str> {
    let source_basis: std::sync::Arc<str> = std::sync::Arc::from(format!(
            "runtime={};commit={};version={};selected-branch={};authoring-branch={};graph-role={};relational-partition={};truth-partition={}",
            runtime_instance_id,
            envelope.commit.commit_id.0,
            envelope.commit.version_id.0,
            selected_branch.0,
            envelope.commit.branch_id.0,
            unresolved.payload().graph_role,
            unresolved
                .payload()
                .relational_partition_id
                .map_or_else(|| "all".to_string(), |partition| partition.as_u32().to_string()),
            unresolved
                .payload()
                .partition_role
                .as_ref()
                .map_or("all", |role| role.as_str()),
        ));
    source_basis
}

fn publication_source_provenance(
    runtime_instance_id: u64,
    request: &RelationalPublicationRequest,
    adapter_semantic_identity: std::sync::Arc<str>,
    source_basis: std::sync::Arc<str>,
) -> worth_runtime_bridge::facade::BridgeAuthoritativeSourceProvenance {
    match request.partition_role.clone() {
            Some(partition_role) => worth_runtime_bridge::facade::BridgeAuthoritativeSourceProvenance::from_owner_partition_publication(
                runtime_instance_id,
                request.graph_role.clone(),
                adapter_semantic_identity,
                source_basis,
                partition_role,
            ),
            None => worth_runtime_bridge::facade::BridgeAuthoritativeSourceProvenance::from_owner_publication(
                runtime_instance_id,
                request.graph_role.clone(),
                adapter_semantic_identity,
                source_basis,
            ),
    }
}
