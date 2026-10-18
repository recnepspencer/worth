use std::sync::Arc;

use crate::branch::{AdmittedRelationalBranchBasis, SelectedRelationalBranchState};
use crate::history::data::{CanonicalCommitEnvelope, CommitId, RelationalCommitReceipt};
use crate::history::RelationalCommitArtifact;
use crate::runtime::RelationalRuntime;

use super::validation::{
    validate_publication, PublicationRequest, PublicationSequence, ValidatedPublication,
};

pub(crate) struct RelationalPublicationAuthority<'runtime> {
    runtime: &'runtime RelationalRuntime,
}

pub(crate) struct RelationalPreparationPublicationAuthority<'runtime> {
    runtime: &'runtime crate::runtime::RelationalPreparationRuntime,
}

pub(crate) struct PreparedRelationalPublication {
    publication: crate::runtime::PreparedVersionedArtifactPublication,
}

pub(crate) struct PreparedRecoveredRelationalPublication {
    publication: crate::runtime::PreparedRecoveredVersionedArtifactPublication,
}

pub(crate) struct PreparedRelationalPublicationAccelerators {
    publication: crate::runtime::PreparedVersionedArtifactAccelerators,
    index_refresh_basis: PreparedIndexRefreshBasis,
}

#[derive(Clone)]
pub(crate) struct PreparedIndexRefreshBasis {
    branch_id: crate::history::data::BranchId,
    version_id: crate::identity::data::VersionId,
    root: Arc<crate::branch::RelationalBranchRoot>,
}

impl PreparedRelationalPublication {
    pub(crate) fn descriptive_touches(
        &self,
    ) -> &crate::history::data::RelationalDescriptiveTouchGraph {
        self.publication.envelope.descriptive_touches()
    }

    pub(crate) fn root(&self) -> &Arc<crate::branch::RelationalBranchRoot> {
        &self.publication.root
    }

    pub(crate) fn index_refresh_basis(&self) -> PreparedIndexRefreshBasis {
        PreparedIndexRefreshBasis {
            branch_id: self.publication.branch_id.clone(),
            version_id: self.publication.commit_reference.version_id,
            root: Arc::clone(&self.publication.root),
        }
    }

    pub(crate) fn into_canonical_parts(
        self,
    ) -> (
        crate::branch::RelationalBranchReferenceCell,
        Arc<crate::branch::RelationalBranchRoot>,
        Arc<CanonicalCommitEnvelope>,
        PreparedRelationalPublicationAccelerators,
    ) {
        let index_refresh_basis = self.index_refresh_basis();
        let (next_cell, root, envelope, publication) =
            self.publication.into_canonical_and_accelerators();
        (
            next_cell,
            root,
            envelope,
            PreparedRelationalPublicationAccelerators {
                publication,
                index_refresh_basis,
            },
        )
    }

    #[cfg(test)]
    pub(crate) fn materialization_counts(&self) -> (u64, u64) {
        let cost = self.publication.root.publication_cost();
        (cost.touched_regions, cost.reused_regions)
    }
}

impl PreparedRecoveredRelationalPublication {
    pub(crate) fn install_recovered(
        self,
        runtime: &RelationalRuntime,
        positioned: &crate::history::data::PositionedCanonicalCommit,
    ) -> Result<(), String> {
        runtime
            .history
            .install_prepared_recovered_versioned_artifact(self.publication, positioned)
    }
}

impl PreparedRelationalPublicationAccelerators {
    pub(crate) fn index_refresh_basis(&self) -> &PreparedIndexRefreshBasis {
        &self.index_refresh_basis
    }

    pub(crate) fn install(
        self,
        runtime: &RelationalRuntime,
        position: crate::publication::patch::data::PatchStreamPosition,
    ) {
        runtime
            .history
            .install_prepared_versioned_accelerators(self.publication, position);
    }
}

impl PreparedIndexRefreshBasis {
    pub(crate) fn branch_id(&self) -> &crate::history::data::BranchId {
        &self.branch_id
    }

    pub(crate) const fn version_id(&self) -> crate::identity::data::VersionId {
        self.version_id
    }

    pub(crate) fn root(&self) -> &crate::branch::RelationalBranchRoot {
        &self.root
    }
}

impl RelationalRuntime {
    pub(crate) fn mvcc_publication_authority(&self) -> RelationalPublicationAuthority<'_> {
        RelationalPublicationAuthority { runtime: self }
    }
}

impl crate::runtime::RelationalPreparationRuntime {
    pub(crate) fn mvcc_publication_authority(
        &self,
    ) -> RelationalPreparationPublicationAuthority<'_> {
        RelationalPreparationPublicationAuthority { runtime: self }
    }
}

trait PublicationPreparationRuntime {
    fn validate_publication_request(
        &self,
        request: PublicationRequest<'_>,
    ) -> Result<ValidatedPublication, String>;

    fn prepare_publication_root(
        &self,
        selected_branch_state: &SelectedRelationalBranchState,
        published_partition_delta: &crate::storage::RelationalPublishedPartitionDelta,
        envelope: &Arc<CanonicalCommitEnvelope>,
        schema_registry: &crate::schema::data::RelationalSchemaRegistry,
    ) -> Result<crate::branch::PreparedRelationalBranchRootCapture, String>;

    fn publication_runtime_instance_id(&self) -> u64;
}

impl PublicationPreparationRuntime for RelationalRuntime {
    fn validate_publication_request(
        &self,
        request: PublicationRequest<'_>,
    ) -> Result<ValidatedPublication, String> {
        validate_publication(self, request)
    }

    fn prepare_publication_root(
        &self,
        selected_branch_state: &SelectedRelationalBranchState,
        published_partition_delta: &crate::storage::RelationalPublishedPartitionDelta,
        envelope: &Arc<CanonicalCommitEnvelope>,
        schema_registry: &crate::schema::data::RelationalSchemaRegistry,
    ) -> Result<crate::branch::PreparedRelationalBranchRootCapture, String> {
        let previous_root = selected_branch_state.root().cloned();
        self.history
            .prepare_branch_root_capture(
                selected_branch_state.state(),
                published_partition_delta,
                previous_root.as_ref(),
                Arc::clone(envelope),
                schema_registry,
                &self.services.symbols.interner_snapshot(),
            )
            .map_err(|denial| format!("branch-root capture denied: {denial:?}"))
    }

    fn publication_runtime_instance_id(&self) -> u64 {
        self.history.runtime_instance_id
    }
}

impl PublicationPreparationRuntime for crate::runtime::RelationalPreparationRuntime {
    fn validate_publication_request(
        &self,
        request: PublicationRequest<'_>,
    ) -> Result<ValidatedPublication, String> {
        super::validation::validate_prepared_publication(self, request)
    }

    fn prepare_publication_root(
        &self,
        selected_branch_state: &SelectedRelationalBranchState,
        published_partition_delta: &crate::storage::RelationalPublishedPartitionDelta,
        envelope: &Arc<CanonicalCommitEnvelope>,
        schema_registry: &crate::schema::data::RelationalSchemaRegistry,
    ) -> Result<crate::branch::PreparedRelationalBranchRootCapture, String> {
        let previous_root = selected_branch_state.root().cloned();
        self.history
            .prepare_branch_root_capture(
                selected_branch_state.state(),
                published_partition_delta,
                previous_root.as_ref(),
                Arc::clone(envelope),
                schema_registry,
                &self.services.symbols.interner_snapshot(),
            )
            .map_err(|denial| format!("branch-root capture denied: {denial:?}"))
    }

    fn publication_runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id()
    }
}

#[allow(clippy::too_many_arguments)]
fn prepare_versioned_publication(
    runtime: &impl PublicationPreparationRuntime,
    commit_id: CommitId,
    commit_reference: RelationalCommitReceipt,
    binding: &AdmittedRelationalBranchBasis,
    selected_branch_state: &SelectedRelationalBranchState,
    published_partition_delta: crate::storage::RelationalPublishedPartitionDelta,
    envelope: Arc<CanonicalCommitEnvelope>,
    schema_registry: &crate::schema::data::RelationalSchemaRegistry,
    sequence: PublicationSequence,
    output_positions: Option<crate::history::data::RequiredOutputPublicationPositions>,
) -> Result<crate::runtime::PreparedVersionedArtifactPublication, String> {
    let validated = runtime.validate_publication_request(PublicationRequest {
        commit_id,
        commit_reference: &commit_reference,
        binding,
        envelope: envelope.as_ref(),
        sequence,
    })?;
    let prepared_root = runtime.prepare_publication_root(
        selected_branch_state,
        &published_partition_delta,
        &envelope,
        schema_registry,
    )?;
    let prepared_root = match output_positions {
        Some(positions) => prepared_root.bind_output_publication_positions(positions),
        None => prepared_root,
    };
    let target = crate::branch::RelationalBranchTarget::from_commit_receipt(
        runtime.publication_runtime_instance_id(),
        &commit_reference,
        prepared_root
            .root()
            .descriptor()
            .cloned()
            .expect("prepared branch root has a descriptor"),
    );
    validated
        .next_cell
        .replace_truth_target(worth_foundational::FoundationalBranchTarget::basis(target));
    let root = prepared_root.into_root();
    let artifact =
        RelationalCommitArtifact::from_envelope_with_root(Arc::clone(&envelope), Arc::clone(&root))
            .map_err(|denial| format!("prepared catalog artifact denied: {denial:?}"))?;
    let new_authoritative_bytes = root
        .publication_cost()
        .new_authoritative_bytes
        .saturating_add(
            artifact
                .authoritative_allocation_observations()
                .iter()
                .map(|allocation| allocation.authoritative_bytes)
                .sum(),
        );
    Ok(crate::runtime::PreparedVersionedArtifactPublication {
        commit_id,
        commit_reference,
        branch_id: validated.branch_id,
        next_cell: validated.next_cell,
        envelope,
        root,
        artifact,
        new_authoritative_bytes,
        recovery_readmission: matches!(sequence, PublicationSequence::RecoveryTruth),
    })
}

impl RelationalPreparationPublicationAuthority<'_> {
    pub(crate) fn prepare_commit(
        &self,
        commit_id: CommitId,
        commit_reference: RelationalCommitReceipt,
        binding: &AdmittedRelationalBranchBasis,
        selected_branch_state: &SelectedRelationalBranchState,
        published_partition_delta: crate::storage::RelationalPublishedPartitionDelta,
        envelope: Arc<CanonicalCommitEnvelope>,
        schema_registry: &crate::schema::data::RelationalSchemaRegistry,
        output_positions: Option<crate::history::data::RequiredOutputPublicationPositions>,
    ) -> Result<PreparedRelationalPublication, String> {
        let publication = prepare_versioned_publication(
            self.runtime,
            commit_id,
            commit_reference,
            binding,
            selected_branch_state,
            published_partition_delta,
            envelope,
            schema_registry,
            PublicationSequence::Truth,
            output_positions,
        )?;
        Ok(PreparedRelationalPublication { publication })
    }
}

impl<'runtime> RelationalPublicationAuthority<'runtime> {
    #[cfg(test)]
    pub(crate) fn validate_versioned_publication(
        &self,
        commit_id: CommitId,
        commit_reference: &RelationalCommitReceipt,
        binding: &AdmittedRelationalBranchBasis,
        envelope: &CanonicalCommitEnvelope,
    ) -> Result<(), String> {
        validate_publication(
            self.runtime,
            PublicationRequest {
                commit_id,
                commit_reference,
                binding,
                envelope,
                sequence: PublicationSequence::Truth,
            },
        )
        .map(|_| ())
    }

    pub(crate) fn prepare_recovered_commit(
        &self,
        commit_id: CommitId,
        commit_reference: RelationalCommitReceipt,
        binding: &AdmittedRelationalBranchBasis,
        published_partition_delta: crate::storage::RelationalPublishedPartitionDelta,
        envelope: Arc<CanonicalCommitEnvelope>,
    ) -> Result<PreparedRecoveredRelationalPublication, String> {
        let selected_branch_state = self
            .runtime
            .selected_branch_state(binding)
            .map_err(|error| error.detail())?;
        let schema_registry = selected_branch_state
            .root()
            .map(|root| root.schema_authority().registry().clone())
            .unwrap_or_else(|| self.runtime.config.schema.registry.clone());
        let prepared = prepare_versioned_publication(
            self.runtime,
            commit_id,
            commit_reference,
            binding,
            &selected_branch_state,
            published_partition_delta,
            envelope,
            &schema_registry,
            PublicationSequence::RecoveryTruth,
            None,
        )?;
        Ok(PreparedRecoveredRelationalPublication {
            publication:
                crate::runtime::PreparedRecoveredVersionedArtifactPublication::from_recovery_readmission(
                    prepared,
                )?,
        })
    }
}
