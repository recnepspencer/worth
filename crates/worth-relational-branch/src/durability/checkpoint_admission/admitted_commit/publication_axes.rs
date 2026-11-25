//! Decode original publication axes from an admitted checkpoint row.
use super::{CheckpointAdmittedCommit, Refusal};
use crate::branch::basis_descriptor::RelationalBranchBasisPosture;
use crate::branch::reference::observation::{
    relational_branch_observation, RelationalBranchReferenceObservation,
};
use worth_relational_contracts::internal_port::branch::reference_checkpoint_data::CANONICAL_RUNTIME_INSTANCE_ID;
use worth_relational_contracts::internal_port::branch::target::{
    RelationalBranchRootDescriptor, RelationalBranchTarget,
};
use worth_relational_contracts::internal_port::branch::target::RelationalBranchTargetOwnerPort as _;
use worth_relational_contracts::internal_port::branch::version::RelationalBranchVersion;
use crate::durability::canonical_commit_artifact::borrowed_payload::BorrowedPayload;
use crate::history::data::BranchId;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    NativeCheckpointBranchReference as Reference, NativeCheckpointBranchTarget as Target,
};
use worth_foundational::{FoundationalBranchReferenceGeneration, FoundationalBranchTarget};

pub struct DecodedPublicationAxes {
    pub predecessor_branch: BranchId,
    pub predecessor_observation: RelationalBranchReferenceObservation,
    pub predecessor_truth: RelationalBranchVersion,
    pub branch: BranchId,
    pub generation: FoundationalBranchReferenceGeneration,
    pub truth: RelationalBranchVersion,
    pub posture: RelationalBranchBasisPosture,
}

impl CheckpointAdmittedCommit {
    pub fn decode_publication_axes(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<DecodedPublicationAxes, Refusal> {
        let row = self
            .0
            .image
            .row(self.0.index.commit.commit_id)
            .ok_or(Refusal::InvalidProjectedMeaning)?;
        let axes = BorrowedPayload(self.0.image.group(&row.groups[1]));
        let field = |name| {
            axes.field(name, request)
                .map_err(worth_relational_contracts::internal_port::durability::checkpoint_admission::refusal::projected_stop)
        };
        let prior_bytes = field("predecessor")?
            .field("reference", request)
            .map_err(worth_relational_contracts::internal_port::durability::checkpoint_admission::refusal::projected_stop)?
            .0;
        let branch_bytes = field("branch")?.0;
        let generation_bytes = field("generation")?.0;
        let truth_bytes = field("truth")?.0;
        let posture_bytes = field("posture")?.0;
        let total = prior_bytes
            .len()
            .saturating_add(branch_bytes.len())
            .saturating_add(generation_bytes.len())
            .saturating_add(truth_bytes.len())
            .saturating_add(posture_bytes.len()) as u64;
        worth_relational_contracts::internal_port::durability::data::recovery_stop::account_work(request, total)
            .map_err(worth_relational_contracts::internal_port::durability::checkpoint_admission::refusal::native_stop)?;
        let prior: Reference<'_> =
            rmp_serde::from_slice(prior_bytes).map_err(|_| Refusal::InvalidProjectedMeaning)?;
        let branch: BranchId =
            rmp_serde::from_slice(branch_bytes).map_err(|_| Refusal::InvalidProjectedMeaning)?;
        let generation: FoundationalBranchReferenceGeneration =
            rmp_serde::from_slice(generation_bytes)
                .map_err(|_| Refusal::InvalidProjectedMeaning)?;
        let truth: RelationalBranchVersion =
            rmp_serde::from_slice(truth_bytes).map_err(|_| Refusal::InvalidProjectedMeaning)?;
        let posture: RelationalBranchBasisPosture =
            rmp_serde::from_slice(posture_bytes).map_err(|_| Refusal::InvalidProjectedMeaning)?;
        let predecessor_branch = BranchId(prior.observation().branch().to_owned());
        let target = match prior.observation().target() {
            Target::Empty => FoundationalBranchTarget::empty(),
            Target::Canonical {
                commit,
                version,
                parents,
                truth_root,
                schema_root,
            } => FoundationalBranchTarget::basis(RelationalBranchTarget::new(
                CANONICAL_RUNTIME_INSTANCE_ID,
                *commit,
                *version,
                parents.to_vec(),
                RelationalBranchRootDescriptor::new(*truth_root, *schema_root),
            )),
        };
        let predecessor_observation = relational_branch_observation(
            CANONICAL_RUNTIME_INSTANCE_ID,
            &predecessor_branch.0,
            target,
            FoundationalBranchReferenceGeneration::new(prior.observation().generation()),
        )
        .map_err(|_| Refusal::InvalidProjectedMeaning)?;
        Ok(DecodedPublicationAxes {
            predecessor_branch,
            predecessor_observation,
            predecessor_truth: RelationalBranchVersion::new(prior.truth_version()),
            branch,
            generation,
            truth,
            posture,
        })
    }
}
