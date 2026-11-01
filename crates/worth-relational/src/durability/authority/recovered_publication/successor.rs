//! Original successor descriptions come from this actual recovery's images.
use crate::branch::{
    RecordedRelationalComponentReference as Recorded, RelationalBranchBasisPosture,
};
use crate::durability::authority::{
    ReadmittedRelationalAssociatedEffect, RecoveredRelationalRuntimeAuthority,
};
use crate::runtime::RelationalRuntime;

#[derive(Debug)]
pub enum RelationalRecordedSuccessorRefusal {
    Request(crate::execution::RelationalExecutionDenialCause),
    ForeignRecoveryRuntime,
    OwnerUnavailable,
    SubjectMismatch,
    MissingRecoveredImage,
    AmbiguousRecoveredImage,
    CanonicalEncoding { detail: String },
    Description(Box<crate::branch::RelationalRecordedComponentRefusal>),
}
impl RecoveredRelationalRuntimeAuthority {
    /// Describe the exact original successor from owner-recovered observations.
    /// This borrows the source image and grants no basis, settlement or adoption.
    pub fn recorded_publication_successor(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        effect: &ReadmittedRelationalAssociatedEffect,
    ) -> Result<Recorded, RelationalRecordedSuccessorRefusal> {
        use RelationalRecordedSuccessorRefusal as Refusal;
        crate::execution::check_request(execution).map_err(Refusal::Request)?;
        if self.runtime_instance_id != runtime.runtime_instance_id()
            || !effect.subject().belongs_to_runtime(runtime)
        {
            return Err(Refusal::ForeignRecoveryRuntime);
        }
        let _operation = runtime
            .owner_binding()
            .admit()
            .ok_or(Refusal::OwnerUnavailable)?;
        if effect.subject().current_settlement(runtime).is_none() {
            return Err(Refusal::SubjectMismatch);
        }
        let entry = effect.subject().source_entry();
        let observations = self
            .recovered_roots
            .publication_observations(entry.commit.commit_id)
            .ok_or(Refusal::MissingRecoveredImage)?;
        let mut selected: Option<Recorded> = None;
        for descriptor in observations {
            if descriptor.branch_id() != &effect.source_envelope().branch_context
                || descriptor.posture() != RelationalBranchBasisPosture::Live
            {
                continue;
            }
            let worth_foundational::FoundationalBranchTarget::Basis(target) =
                descriptor.reference().target()
            else {
                continue;
            };
            if target.selected_commit_id() != entry.commit.commit_id.0
                || target.version_id() != entry.commit.version_id.0
                || target.roots() != &entry.root
                || target.parent_commit_ids().iter().copied().ne(entry
                    .commit
                    .parents
                    .iter()
                    .map(|id| id.0))
            {
                continue;
            }
            let candidate = Recorded::from_readmitted_source(execution, descriptor, entry)
                .map_err(|cause| Refusal::Description(Box::new(cause)))?;
            if selected
                .as_ref()
                .is_some_and(|previous| previous != &candidate)
            {
                return Err(Refusal::AmbiguousRecoveredImage);
            }
            selected = Some(candidate);
        }
        let selected = selected.ok_or(Refusal::MissingRecoveredImage)?;
        crate::execution::check_request(execution).map_err(Refusal::Request)?;
        Ok(selected)
    }
}
