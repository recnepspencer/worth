//! Operational evidence joined to an actual selected directory, never an envelope.
use super::read_image::{CheckpointAdmissionReadImage, CheckpointOriginalRow};
use super::refusal::{projected_stop, RelationalCheckpointAdmissionRefusal as Refusal};
use super::row_custody::AdmissionCustody;
use crate::branch::root::RelationalBranchRootSchemaAuthority;
use crate::branch::root::RelationalBranchVisibilityCommitment;
use crate::history::commit::{RelationalCommitIndexEntry, RetainedCanonicalIndex};
use serde::Serialize;
use std::sync::{Arc, Weak};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
pub mod index_predecessor;

/// Only Native's selected-directory consumer can construct this owner.
/// It cannot be converted into an original positioned canonical frame.
#[derive(Debug, Clone)]
pub struct CheckpointAdmittedCommit(Arc<AdmittedRow>);

#[derive(Debug)]
struct AdmittedRow {
    owner: Weak<AdmissionCustody>,
    image: CheckpointAdmissionReadImage,
    index: RetainedCanonicalIndex,
    schema: Arc<RelationalBranchRootSchemaAuthority>,
    visibility: RelationalBranchVisibilityCommitment,
    _memory: Option<ExecutionMemoryReservation>,
}

impl CheckpointAdmittedCommit {
    pub fn retain(
        owner: Weak<AdmissionCustody>,
        image: CheckpointAdmissionReadImage,
        index: RetainedCanonicalIndex,
        schema: Arc<RelationalBranchRootSchemaAuthority>,
        visibility: RelationalBranchVisibilityCommitment,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Refusal> {
        let layout = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<AdmittedRow>())
            .map_err(|_| Refusal::CapacityOverflow)?
            .0
            .pad_to_align();
        let memory = request
            .reserve_retained_memory(layout.size() as u64)
            .map_err(|cause| Refusal::Request(cause.into()))?;
        Ok(Self(Arc::new(AdmittedRow {
            owner,
            image,
            index,
            schema,
            visibility,
            _memory: memory,
        })))
    }

    pub fn original_row(&self) -> CheckpointOriginalRow<'_> {
        CheckpointOriginalRow::new(&self.0.image, &self.0.index)
    }

    pub fn scope(&self) -> &worth_foundational::facade::CheckpointAdmissionScope {
        self.0.image.cut().scope()
    }
    pub fn require_publication_axes(
        &self,
        predecessor: &crate::branch::basis::RelationalBranchBasisDescriptor,
        successor: &crate::branch::basis::RelationalBranchBasisDescriptor,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Refusal> {
        use super::recorded_reference::recorded_reference;
        use crate::durability::canonical_commit_artifact::borrowed_payload::BorrowedPayload;
        let row = self
            .0
            .image
            .row(self.0.index.commit.commit_id)
            .ok_or(Refusal::InvalidProjectedMeaning)?;
        let axes = BorrowedPayload(self.0.image.group(&row.groups[1]));
        let field = |name| axes.field(name, request).map_err(projected_stop);
        let prior = field("predecessor")?
            .field("reference", request)
            .map_err(projected_stop)?;
        require_encoded(&recorded_reference(predecessor), prior.0, request)?;
        require_encoded(successor.branch_id(), field("branch")?.0, request)?;
        require_encoded(
            &successor.reference().generation(),
            field("generation")?.0,
            request,
        )?;
        require_encoded(&successor.truth_version(), field("truth")?.0, request)?;
        require_encoded(&successor.posture(), field("posture")?.0, request)
    }

    pub fn require_pruned_publication_axes(
        &self,
        expected_branch: &crate::history::data::BranchId,
        expected_predecessor_commit: crate::history::data::CommitId,
        successor: &crate::branch::basis::RelationalBranchBasisDescriptor,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Refusal> {
        use crate::durability::canonical_commit_artifact::borrowed_payload::BorrowedPayload;
        use worth_foundational::facade::{
            NativeCheckpointBranchReference as Reference, NativeCheckpointBranchTarget as Target,
        };
        let row = self
            .0
            .image
            .row(self.0.index.commit.commit_id)
            .ok_or(Refusal::InvalidProjectedMeaning)?;
        let axes = BorrowedPayload(self.0.image.group(&row.groups[1]));
        let field = |name| axes.field(name, request).map_err(projected_stop);
        let prior_bytes = field("predecessor")?
            .field("reference", request)
            .map_err(projected_stop)?
            .0;
        crate::durability::artifact_payload_emission::request::account_work(
            request,
            prior_bytes.len() as u64,
        )
        .map_err(super::refusal::native_stop)?;
        let prior: Reference<'_> =
            rmp_serde::from_slice(prior_bytes).map_err(|_| Refusal::InvalidProjectedMeaning)?;
        if prior.observation().branch() != expected_branch.0.as_str() {
            return Err(Refusal::InvalidProjectedMeaning);
        }
        let Target::Canonical { commit, .. } = prior.observation().target() else {
            return Err(Refusal::InvalidProjectedMeaning);
        };
        if *commit != expected_predecessor_commit.0 {
            return Err(Refusal::InvalidProjectedMeaning);
        }
        require_encoded(successor.branch_id(), field("branch")?.0, request)?;
        require_encoded(
            &successor.reference().generation(),
            field("generation")?.0,
            request,
        )?;
        require_encoded(&successor.truth_version(), field("truth")?.0, request)?;
        require_encoded(&successor.posture(), field("posture")?.0, request)
    }
    pub fn index(&self) -> &RelationalCommitIndexEntry {
        &self.0.index
    }
    pub fn schema(&self) -> &Arc<RelationalBranchRootSchemaAuthority> {
        &self.0.schema
    }
    pub fn visibility(&self) -> RelationalBranchVisibilityCommitment {
        self.0.visibility
    }
    pub fn positions(&self) -> Option<&crate::history::data::RequiredOutputPublicationPositions> {
        self.0.image.positions(self.0.index.commit.commit_id)
    }
    pub fn same_owner(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    pub fn belongs_to(&self, owner: &Arc<AdmissionCustody>) -> bool {
        Weak::ptr_eq(&self.0.owner, &Arc::downgrade(owner))
    }
    pub fn links_artifact(
        &self,
        artifact: &crate::history::commit::RelationalCommitArtifact,
    ) -> bool {
        let Some(owner) = self.0.owner.upgrade() else {
            return false;
        };
        let Some((entry, source)) = artifact.envelope_residency().source_placement() else {
            return false;
        };
        entry.as_ref() == self.index() && owner.selects_source(&source)
    }
}

pub fn require_encoded(
    value: &impl Serialize,
    actual: &[u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Refusal> {
    let expected = crate::durability::artifact_payload_emission::request::encode_named_charged(
        &[],
        &super::binary_encoding::BinaryFixedArrays(value),
        request,
    )
    .map_err(super::refusal::native_stop)?;
    if expected.as_ref() != actual {
        return Err(Refusal::InvalidProjectedMeaning);
    }
    Ok(())
}
