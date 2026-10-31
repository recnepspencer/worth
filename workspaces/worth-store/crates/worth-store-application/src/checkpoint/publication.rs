use worth_execution::{ExecutionRequest, WorkCeilingDenial};
use worth_relational::facade::branch::RelationalBranchTransitionSequence;
use worth_relational::facade::publication::PatchStreamPosition;
use worth_store::physical_runtime::{
    AnchoredRecordName, AnchoredRecordPublication, AnchoredRecordPublicationFailure,
    AnchoredRecordPublished, AnchoredRecordReadFailure, AnchoredRecordsDenial,
    PhysicalMutationDeadline, ServingPhysicalRuntime,
};
use worth_store_relational::{PreparedRelationalSection, RelationalSectionCheckpointDenial};

#[derive(Debug)]
pub enum ApplicationHomeCheckpointDenial {
    Unavailable(AnchoredRecordsDenial),
    Read(AnchoredRecordReadFailure),
    Manifest(super::HomeManifestDenial),
    Native(RelationalSectionCheckpointDenial),
    Execution(WorkCeilingDenial),
    Deadline(crate::ApplicationHomeDeadlineDenial),
    SizeLimit,
    Publication(AnchoredRecordPublicationFailure),
    PhysicalCheckpoint {
        detail: String,
    },
    /// The manifest is already selected; the original request stopped before
    /// the optional physical WAL checkpoint could start.
    PhysicalCheckpointRequestStopped {
        cause: crate::ApplicationHomeDeadlineDenial,
    },
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ApplicationHomeCheckpoint {
    pub covered: PatchStreamPosition,
    pub covered_transitions: RelationalBranchTransitionSequence,
    pub generation: u64,
    pub bytes: usize,
}

impl ApplicationHomeCheckpoint {
    pub fn covered(&self) -> PatchStreamPosition {
        self.covered
    }
    pub fn covered_transitions(&self) -> RelationalBranchTransitionSequence {
        self.covered_transitions
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn bytes(&self) -> usize {
        self.bytes
    }
}

impl crate::ReadmittedApplicationHome {
    pub fn checkpoint(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ApplicationHomeCheckpoint, ApplicationHomeCheckpointDenial> {
        checkpoint(&self.context.serving, execution, || {
            self.relational.prepare_checkpoint(execution)
        })
    }
}

pub(crate) fn checkpoint<'section>(
    serving: &ServingPhysicalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    capture: impl FnOnce() -> Result<
        PreparedRelationalSection<'section>,
        RelationalSectionCheckpointDenial,
    >,
) -> Result<ApplicationHomeCheckpoint, ApplicationHomeCheckpointDenial> {
    let prepared = super::preparation::capture(serving, execution, capture)?;
    publish(serving, prepared, execution)
}

fn publish(
    serving: &ServingPhysicalRuntime,
    prepared: super::preparation::PreparedHomeCheckpoint<'_>,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<ApplicationHomeCheckpoint, ApplicationHomeCheckpointDenial> {
    use ApplicationHomeCheckpointDenial as Denial;
    let super::preparation::PreparedHomeCheckpoint {
        native: prepared,
        payload,
        members,
        member_memory: _member_memory,
        generation,
    } = prepared;
    let records = serving.anchored_records().map_err(Denial::Unavailable)?;
    let anchor = AnchoredRecordName::new(crate::registry::CHECKPOINT_ANCHOR)
        .expect("static checkpoint anchor");
    let milliseconds =
        crate::deadline::milliseconds(execution, 120_000).map_err(Denial::Deadline)?;
    let deadline = PhysicalMutationDeadline::after_milliseconds(milliseconds)
        .expect("admitted positive checkpoint duration");
    execution.consult().map_err(Denial::Execution)?;
    let crate::frame_encoding::EncodedHomeFrame {
        bytes,
        memory: _memory,
    } = payload;
    let published = records
        .publish(
            AnchoredRecordPublication::new(
                anchor,
                generation,
                bytes,
                crate::open::physical_admission::placement(),
                deadline,
            )
            .with_members(members),
        )
        .map_err(Denial::Publication)?;
    let membership = match published {
        AnchoredRecordPublished::Published(membership)
        | AnchoredRecordPublished::AlreadyPublished(membership) => membership,
    };
    let result = ApplicationHomeCheckpoint {
        covered: prepared.covered(),
        covered_transitions: prepared.covered_transitions(),
        generation: membership.version().generation(),
        bytes: prepared.bytes().len(),
    };
    prepared.select(&membership);
    super::physical_checkpoint::complete(serving, result.generation, execution)?;
    Ok(result)
}
