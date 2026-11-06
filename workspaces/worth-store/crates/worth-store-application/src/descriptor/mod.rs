pub(crate) mod binding;
mod codec;
mod transition_required;

use worth_execution::{ExecutionRequest, WorkCeilingDenial};
use worth_foundational::facade::{
    ApplicationHomeBinding, DurableArtifactRetention, HomeIdentity, HomeIncarnation, RetentionClass,
};
use worth_relational::facade::runtime::RelationalRuntimeDefinition;
use worth_store::physical_runtime::{
    AnchoredRecordName, AnchoredRecordPublication, AnchoredRecordPublicationFailure,
    AnchoredRecordReadFailure, AnchoredRecordsDenial, PhysicalMutationDeadline,
    ServingPhysicalRuntime,
};

const MAXIMUM_DESCRIPTOR_BYTES: usize = 64 << 10;

pub(crate) struct HomeDescriptor {
    pub(crate) identity: HomeIdentity,
    pub(crate) incarnation: HomeIncarnation,
    pub(crate) binding: ApplicationHomeBinding,
    pub(crate) build: worth_foundational::facade::ApplicationBuildIdentity,
}

impl DurableArtifactRetention for HomeDescriptor {
    const RETENTION_CLASS: RetentionClass = RetentionClass::Authoritative;
}

#[derive(Debug)]
pub enum ApplicationHomeDescriptorDenial {
    Unavailable(AnchoredRecordsDenial),
    Read(AnchoredRecordReadFailure),
    Publication(AnchoredRecordPublicationFailure),
    Execution(WorkCeilingDenial),
    Deadline(crate::ApplicationHomeDeadlineDenial),
    Malformed,
    SizeLimit,
    UnsupportedVersion {
        observed: u16,
    },
    UnsupportedBinding {
        tag: u16,
    },
    IdentityMismatch,
    ApplicationBindingMissing,
    BuildBindingMissing,
    BindingMismatch,
    TransitionPredecessorNotRostered,
    TransitionPredecessorMismatch,
    TransitionRequired {
        recorded: Box<ApplicationHomeBinding>,
        requested: Box<ApplicationHomeBinding>,
        _funding: Option<worth_execution::ExecutionMemoryReservation>,
    },
    IncompleteInitialization,
    IncarnationExhausted,
}

pub(crate) struct PreparedDescriptor {
    descriptor: HomeDescriptor,
    payload: crate::frame_encoding::EncodedHomeFrame,
    generation: u64,
    definition: RelationalRuntimeDefinition,
    transition: crate::open::readmission::transition::HomeTransitionAdmission,
}
impl PreparedDescriptor {
    pub(crate) fn descriptor(&self) -> &HomeDescriptor {
        &self.descriptor
    }
    pub(crate) fn home_identity(&self) -> HomeIdentity {
        self.descriptor.identity
    }
}

mod admission;
pub(crate) use admission::prepare_admission;

pub(crate) fn publish(
    serving: &ServingPhysicalRuntime,
    prepared: PreparedDescriptor,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<PublishedHomeDescriptor, ApplicationHomeDescriptorDenial> {
    use ApplicationHomeDescriptorDenial as Denial;
    let records = serving.anchored_records().map_err(Denial::Unavailable)?;
    let anchor = AnchoredRecordName::new(crate::registry::HOME_ANCHOR).expect("static home anchor");
    let PreparedDescriptor {
        descriptor,
        payload,
        generation,
        definition,
        transition,
    } = prepared;
    let milliseconds =
        crate::deadline::milliseconds(execution, 30_000).map_err(Denial::Deadline)?;
    let deadline = PhysicalMutationDeadline::after_milliseconds(milliseconds)
        .expect("admitted positive descriptor duration");
    execution.consult().map_err(Denial::Execution)?;
    let crate::frame_encoding::EncodedHomeFrame {
        bytes,
        memory: _memory,
    } = payload;
    records
        .publish(AnchoredRecordPublication::new(
            anchor,
            generation,
            bytes,
            crate::open::physical_admission::placement(),
            deadline,
        ))
        .map_err(Denial::Publication)?;
    Ok(PublishedHomeDescriptor {
        descriptor,
        definition,
        transition,
    })
}

pub(crate) struct PublishedHomeDescriptor {
    pub(crate) descriptor: HomeDescriptor,
    pub(crate) definition: RelationalRuntimeDefinition,
    pub(crate) transition: crate::open::readmission::transition::HomeTransitionAdmission,
}
