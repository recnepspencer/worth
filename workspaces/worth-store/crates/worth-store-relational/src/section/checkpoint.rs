//! Capture and placement preparation, retaining the actual selection fence.
use super::RelationalStoreSection;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, WorkCeilingDenial};
use worth_relational::facade::durability::{
    RelationalCheckpointArtifactPolicy, RelationalNativeCheckpoint,
    RelationalNativeCheckpointCaptureDenial,
};
use worth_store::physical_runtime::{AnchoredRecordMember, AnchoredRecordMemberLocator};

mod complete;
mod members;
mod publication;
pub use complete::{
    PreparedCompleteRelationalSection, RelationalSectionCompleteCheckpointPreparation,
};
use publication::SectionPublication;

#[derive(Debug)]
pub enum RelationalSectionCheckpointDenial {
    OwnerUnavailable,
    Capture(RelationalNativeCheckpointCaptureDenial),
    Execution(WorkCeilingDenial),
    MalformedSection,
    ReadProtection(worth_relational::facade::durability::RelationalDurableOwnedReadRefusal),
    MemberSelection(crate::RelationalStoreMemberSelectionDenial),
    StaleCapture,
    MissingCarriedMember,
    SizeLimit,
    Allocation(std::collections::TryReserveError),
    Acknowledgement(
        worth_relational::facade::durability::RelationalCheckpointAcknowledgementDenial,
    ),
    PublicationAdmission(
        worth_relational::facade::durability::RelationalCheckpointAcknowledgementDenial,
    ),
}

/// A native capture and the read-custody fence that must outlive publication.
/// No Clone/Copy: only selection may acknowledge this capture.
///
/// ```compile_fail,E0599
/// use worth_store_relational::PreparedRelationalSection;
/// use worth_store::physical_runtime::AnchoredRecordMembership;
/// fn pair_descriptions(section: PreparedRelationalSection, receipt: &AnchoredRecordMembership) {
///     section.select(receipt);
/// }
/// ```
///
/// ```compile_fail,E0599
/// use worth_relational::facade::durability::RelationalNativeCheckpoint;
/// fn public_fate_is_not_selection(native: RelationalNativeCheckpoint) {
///     native.acknowledge_publication();
/// }
/// ```
///
/// ```compile_fail,E0432
/// use worth_relational::facade::durability::ActualCheckpointSelection;
/// ```
///
/// ```
/// use worth_store_relational::PreparedRelationalSection;
/// fn observe(section: &PreparedRelationalSection) {
///     let _description = section.publication_subject();
/// }
/// ```
pub struct PreparedRelationalSection {
    checkpoint: RelationalNativeCheckpoint,
    publication: SectionPublication,
}

/// Lifetime-only transport to an accepted physical step. No publication or
/// acknowledgement authority, source map, or Native capture is exposed.
#[derive(Clone)]
pub struct RelationalCheckpointSelectionProtection {
    _retained: crate::cold_source::RetainedMemberPublication,
}

impl RelationalStoreSection {
    pub fn complete_checkpoint_preparation(
        &self,
    ) -> RelationalSectionCompleteCheckpointPreparation {
        RelationalSectionCompleteCheckpointPreparation {
            native: self.runtime.complete_checkpoint_service(),
            source: std::sync::Arc::downgrade(&self.cold_source),
            selection: std::sync::Arc::downgrade(&self.checkpoint_selection),
        }
    }

    pub fn prepare_checkpoint(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<PreparedRelationalSection, RelationalSectionCheckpointDenial> {
        let mut prepared = prepare(&self.runtime, &self.cold_source, execution)?;
        prepared.publication.selection = Some(self.checkpoint_selection.clone());
        Ok(prepared)
    }
}

pub(super) fn prepare(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    source: &std::sync::Arc<crate::cold_source::StoreColdSource>,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<PreparedRelationalSection, RelationalSectionCheckpointDenial> {
    use RelationalSectionCheckpointDenial as Denial;
    let members = source
        .member_locators_for_request(execution)
        .map_err(Denial::ReadProtection)?;
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &members,
                source: Some(source.clone()),
            },
        )
        .map_err(Denial::Capture)?;
    prepare_captured(source, checkpoint, execution)
}

pub(crate) fn prepare_captured(
    source: &std::sync::Arc<crate::cold_source::StoreColdSource>,
    checkpoint: RelationalNativeCheckpoint,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<PreparedRelationalSection, RelationalSectionCheckpointDenial> {
    let publication = SectionPublication::prepare(source, &checkpoint, execution)?;
    Ok(PreparedRelationalSection {
        checkpoint,
        publication,
    })
}

impl PreparedRelationalSection {
    pub fn validate_publication(&self) -> Result<(), RelationalSectionCheckpointDenial> {
        self.publication.validate_publication(&self.checkpoint)
    }
    pub fn bytes(&self) -> &[u8] {
        self.publication.bytes()
    }
    pub fn covered(&self) -> worth_relational::facade::publication::PatchStreamPosition {
        self.checkpoint
            .covered_position()
            .expect("encoded section has canonical coverage")
    }
    pub fn covered_transitions(
        &self,
    ) -> worth_relational::facade::branch::RelationalBranchTransitionSequence {
        self.checkpoint
            .covered_branch_transitions()
            .expect("encoded section has transition coverage")
    }
    pub fn validate_previous(
        &self,
        previous: &[u8],
    ) -> Result<(), RelationalSectionCheckpointDenial> {
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
        RelationalSectionCheckpointDenial,
    > {
        self.publication
            .prepare_members(&self.checkpoint, previous, execution)
    }
    pub fn publication_subject(
        &self,
    ) -> Option<&worth_relational::facade::durability::OpaqueCapturedCheckpointSubject> {
        self.checkpoint.publication_subject()
    }
    pub fn selection_protection(&self) -> RelationalCheckpointSelectionProtection {
        self.publication.selection_protection()
    }
    pub(super) fn with_selection(
        mut self,
        selection: std::sync::Arc<dyn super::RelationalCheckpointSelectionSource>,
    ) -> Self {
        self.publication.selection = Some(selection);
        self
    }
    pub fn acknowledge_selected(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<(), RelationalSectionCheckpointDenial> {
        self.publication
            .acknowledge_selected(&self.checkpoint, execution, context)
    }
}

#[cfg(test)]
mod journal_tests;
