//! Genuine Complete capture, issued only by the original Section's weak binding.
use super::{
    RelationalSectionCheckpointDenial as Denial, RelationalStoreSection, SectionPublication,
};
use std::sync::Weak;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_relational::facade::durability::{
    CompleteRelationalCheckpointPreparation, CompleteRelationalNativeCheckpoint,
    RelationalCheckpointArtifactPolicy,
};
use worth_store::physical_runtime::{AnchoredRecordMember, AnchoredRecordMemberLocator};

/// A genuine Native Complete capture and its SAME Store framing/selection owner.
/// Ordinary prepared sections cannot construct or become this result.
pub struct PreparedCompleteRelationalSection {
    complete: CompleteRelationalNativeCheckpoint,
    publication: SectionPublication,
}

/// Preparation access to the original Section; idle clones retain no service,
/// source, selection, or lifecycle owner. A successful call owns its capture.
#[derive(Clone)]
pub struct RelationalSectionCompleteCheckpointPreparation {
    pub(super) native: CompleteRelationalCheckpointPreparation,
    pub(super) source: Weak<crate::cold_source::StoreColdSource>,
    pub(super) selection: Weak<dyn super::super::RelationalCheckpointSelectionSource>,
}

impl std::fmt::Debug for RelationalSectionCompleteCheckpointPreparation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("RelationalSectionCompleteCheckpointPreparation")
    }
}

impl RelationalStoreSection {
    pub fn prepare_complete_checkpoint(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<PreparedCompleteRelationalSection, Denial> {
        self.complete_checkpoint_preparation().prepare(execution)
    }
}

impl RelationalSectionCompleteCheckpointPreparation {
    /// Descriptive counts from the SAME weak source used by this operation.
    pub fn cold_storage_reads(&self) -> Option<crate::RelationalStoreColdReadCounters> {
        self.source.upgrade().map(|source| source.counters())
    }
    pub fn prepare(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<PreparedCompleteRelationalSection, Denial> {
        execution.consult().map_err(Denial::Execution)?;
        // Keep both original Store owners through the Native operation and
        // framing. Neither an upgraded runtime nor its services are retained here.
        let source = self.source.upgrade().ok_or(Denial::OwnerUnavailable)?;
        let selection = self.selection.upgrade().ok_or(Denial::OwnerUnavailable)?;
        let members = source
            .member_locators_for_request(execution)
            .map_err(Denial::ReadProtection)?;
        let complete = self
            .native
            .prepare(
                execution,
                RelationalCheckpointArtifactPolicy::ExternalMembers {
                    current_members: &members,
                    source: Some(source.clone()),
                },
            )
            .map_err(Denial::Capture)?;
        let mut publication = SectionPublication::prepare_complete(&source, &complete, execution)?;
        publication.selection = Some(selection);
        Ok(PreparedCompleteRelationalSection {
            complete,
            publication,
        })
    }
}

impl PreparedCompleteRelationalSection {
    pub fn validate_publication(&self) -> Result<(), Denial> {
        self.publication
            .validate_publication(self.complete.checkpoint())
    }
    pub fn bytes(&self) -> &[u8] {
        self.publication.bytes()
    }
    pub fn covered(&self) -> worth_relational::facade::publication::PatchStreamPosition {
        self.complete
            .checkpoint()
            .covered_position()
            .expect("encoded section has canonical coverage")
    }
    pub fn covered_transitions(
        &self,
    ) -> worth_relational::facade::branch::RelationalBranchTransitionSequence {
        self.complete
            .checkpoint()
            .covered_branch_transitions()
            .expect("encoded section has transition coverage")
    }
    pub fn validate_previous(&self, previous: &[u8]) -> Result<(), Denial> {
        self.publication.validate_previous(previous)
    }
    pub fn prepare_members(
        &self,
        previous: &[AnchoredRecordMemberLocator],
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<
        (
            Vec<AnchoredRecordMember>,
            Option<ExecutionMemoryReservation>,
        ),
        Denial,
    > {
        self.publication
            .prepare_members(self.complete.checkpoint(), previous, execution)
    }
    pub fn publication_subject(
        &self,
    ) -> Option<&worth_relational::facade::durability::OpaqueCapturedCheckpointSubject> {
        self.complete.checkpoint().publication_subject()
    }
    pub fn selection_protection(&self) -> super::RelationalCheckpointSelectionProtection {
        self.publication.selection_protection()
    }
    pub fn acknowledge_selected(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<(), Denial> {
        self.publication
            .acknowledge_selected(self.complete.checkpoint(), execution, context)
    }
}
