use worth_store_physical_format::RecordFrameCoordinate;

use crate::physical_runtime::record_serving::residency::candidate_frame_residency::{
    CandidateFrameCoordinate, CandidateFrameFailurePosture, CandidateFrameWriteFailure,
};
use crate::physical_runtime::{
    IndeterminatePhysicalDataDispatch, PhysicalDataDispatchFailureCause,
    PhysicalDataDispatchOutcome, PhysicalDataEffectSettlement, PhysicalRecordPressureBasis,
    PhysicalRecordPressureEvidence, PhysicalRecordResidencyFailure, PhysicalWorkEffectFate,
    RecordAppendDenial, WalDurablePhysicalMutation,
};

pub(super) fn map_canonical_failure(
    failure: CandidateFrameWriteFailure<
        crate::physical_runtime::record_serving::CanonicalRecordMutationFailure,
    >,
    generation: crate::physical_runtime::LifecycleGeneration,
    basis: PhysicalRecordPressureBasis,
) -> DispatchFailure {
    match failure {
        CandidateFrameWriteFailure::Contract { violation, posture } => posture_failure(
            PhysicalDataDispatchFailureCause::CandidateFrameContract(violation),
            posture,
        ),
        CandidateFrameWriteFailure::Residency { denial, posture } => posture_failure(
            project_candidate_admission_failure(denial, generation, basis),
            posture,
        ),
        CandidateFrameWriteFailure::Effect(failure) => DispatchFailure::Settled {
            cause: PhysicalDataDispatchFailureCause::Canonical(failure.evidence()),
            fate: failure.effect_fate(),
        },
    }
}

pub(super) fn map_writeback_failure(
    failure: CandidateFrameWriteFailure<
        crate::physical_runtime::PhysicalRecordWritebackFailureEvidence,
    >,
    generation: crate::physical_runtime::LifecycleGeneration,
    basis: PhysicalRecordPressureBasis,
) -> DispatchFailure {
    match failure {
        CandidateFrameWriteFailure::Contract { violation, posture } => posture_failure(
            PhysicalDataDispatchFailureCause::CandidateFrameContract(violation),
            posture,
        ),
        CandidateFrameWriteFailure::Residency { denial, posture } => posture_failure(
            project_candidate_admission_failure(denial, generation, basis),
            posture,
        ),
        CandidateFrameWriteFailure::Effect(failure) => DispatchFailure::Settled {
            cause: failure.pressure(generation).map_or(
                PhysicalDataDispatchFailureCause::ExistingArtifactWriteback(failure),
                PhysicalDataDispatchFailureCause::PhysicalPressure,
            ),
            fate: failure.effect_fate(),
        },
    }
}

pub(super) fn pressure_basis(
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    coordinate: CandidateFrameCoordinate,
    length: u32,
) -> Option<PhysicalRecordPressureBasis> {
    let frame = RecordFrameCoordinate::new(coordinate.artifact(), coordinate.offset(), length)?;
    Some(PhysicalRecordPressureBasis::for_store(store).with_frame_coordinate(frame))
}

pub(super) enum DispatchFailure {
    ProvenNoEffect(PhysicalDataDispatchFailureCause),
    Settled {
        cause: PhysicalDataDispatchFailureCause,
        fate: PhysicalWorkEffectFate,
    },
    Uncertain(PhysicalDataDispatchFailureCause),
}

pub(super) fn classify_dispatch_failure(
    durable: WalDurablePhysicalMutation,
    effects: Vec<PhysicalDataEffectSettlement>,
    failure: DispatchFailure,
) -> PhysicalDataDispatchOutcome {
    match failure {
        DispatchFailure::ProvenNoEffect(cause) if effects.is_empty() => {
            PhysicalDataDispatchOutcome::NotStarted { durable, cause }
        }
        DispatchFailure::Settled {
            cause,
            fate: PhysicalWorkEffectFate::ProvenNoEffect,
        } if effects.is_empty() => PhysicalDataDispatchOutcome::NotStarted { durable, cause },
        DispatchFailure::ProvenNoEffect(cause)
        | DispatchFailure::Settled {
            cause,
            fate: PhysicalWorkEffectFate::ProvenNoEffect,
        } => PhysicalDataDispatchOutcome::Suspended(
            crate::physical_runtime::SuspendedPhysicalDataDispatch::new(durable, effects, cause),
        ),
        DispatchFailure::Settled { cause, .. } | DispatchFailure::Uncertain(cause) => {
            PhysicalDataDispatchOutcome::Indeterminate(IndeterminatePhysicalDataDispatch::new(
                durable, effects, cause,
            ))
        }
    }
}

pub(super) fn project_candidate_admission_failure(
    denial: RecordAppendDenial,
    generation: crate::physical_runtime::LifecycleGeneration,
    basis: PhysicalRecordPressureBasis,
) -> PhysicalDataDispatchFailureCause {
    match denial {
        RecordAppendDenial::ResidencyUnavailable(failure) => {
            project_residency_failure(failure, generation, basis)
        }
        denial => PhysicalDataDispatchFailureCause::CandidateAdmission(denial),
    }
}

pub(super) fn project_residency_failure(
    failure: PhysicalRecordResidencyFailure,
    generation: crate::physical_runtime::LifecycleGeneration,
    basis: PhysicalRecordPressureBasis,
) -> PhysicalDataDispatchFailureCause {
    match PhysicalRecordPressureEvidence::from_failure(failure, generation, basis) {
        Some(evidence) => PhysicalDataDispatchFailureCause::PhysicalPressure(evidence),
        None => PhysicalDataDispatchFailureCause::RecordResidency(failure),
    }
}

fn posture_failure(
    cause: PhysicalDataDispatchFailureCause,
    posture: CandidateFrameFailurePosture,
) -> DispatchFailure {
    match posture {
        CandidateFrameFailurePosture::ProvenNoEffect => DispatchFailure::ProvenNoEffect(cause),
        CandidateFrameFailurePosture::UnsettledBeforeEffect
        | CandidateFrameFailurePosture::EffectPossible => DispatchFailure::Uncertain(cause),
    }
}
