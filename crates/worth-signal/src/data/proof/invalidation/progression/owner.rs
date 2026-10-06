use worth_proof::{Binding, TransitionOutcome};

use super::committed::DependencyAdmissionDenial;
use super::source::SourceAdmissionDenial;
use super::structural::StructuralAdmissionDenial;
use super::{
    AdmittedDependencyRecompute, AdmittedSourceRecompute, AdmittedStructuralRecompute,
    ExecutedInvalidationBatch, InvalidationOriginAdmissionOutcome, InvalidationOriginBindingAxes,
    InvalidationProgressionDenial, InvalidationReadinessEpoch, InvalidationStageOrder,
    InvalidationWorkBatch, InvalidationWorkBindingAxes, LoweredInvalidationBatch,
    ReadyInvalidationBatch, ResolvedInvalidationWork,
};
use crate::data::proof::invalidation::revalidation::CanonicalInvalidationOrigin;

/// The sole crate-visible transition door for Signal's invalidation phases.
pub(crate) struct InvalidationProgressionOwner;

impl InvalidationProgressionOwner {
    pub(crate) fn admit_graph_batch(
        expected: super::GraphBatchBindingAxes,
        current: super::GraphBatchBindingAxes,
    ) -> Result<super::DisjointGraphBatch, crate::data::error::SignalError> {
        super::DisjointGraphBatch::admit(expected, current)
    }

    pub(crate) fn resolve_origin(
        batch: InvalidationWorkBatch,
        binding: InvalidationOriginBindingAxes,
    ) -> InvalidationOriginAdmissionOutcome {
        match batch.first().input().origin() {
            CanonicalInvalidationOrigin::SourceRecompute => {
                map_source(AdmittedSourceRecompute::admit(batch, binding))
            }
            CanonicalInvalidationOrigin::DependencyCommit => {
                map_dependency(AdmittedDependencyRecompute::admit(batch, binding))
            }
            CanonicalInvalidationOrigin::StructuralRecompute => {
                map_structural(AdmittedStructuralRecompute::admit(batch, binding))
            }
        }
    }

    pub(crate) fn lower(
        resolved: ResolvedInvalidationWork,
        readiness_epoch: InvalidationReadinessEpoch,
        stage_order: InvalidationStageOrder,
    ) -> LoweredInvalidationBatch {
        LoweredInvalidationBatch::lower(resolved, readiness_epoch, stage_order)
    }

    pub(crate) fn binding_axes(lowered: &LoweredInvalidationBatch) -> InvalidationWorkBindingAxes {
        lowered.binding().axes().clone()
    }

    pub(crate) fn admit_ready(
        lowered: LoweredInvalidationBatch,
        current: InvalidationWorkBindingAxes,
    ) -> TransitionOutcome<
        ReadyInvalidationBatch,
        InvalidationProgressionDenial,
        InvalidationProgressionDenial,
        InvalidationProgressionDenial,
        InvalidationProgressionDenial,
        crate::data::error::SignalError,
    > {
        ReadyInvalidationBatch::admit(lowered, Binding::new(current))
    }

    pub(crate) fn ready_binding(ready: &ReadyInvalidationBatch) -> &InvalidationWorkBindingAxes {
        ready.binding().axes()
    }

    pub(crate) fn execute<Outcome>(
        ready: ReadyInvalidationBatch,
        effect: impl FnOnce(&InvalidationWorkBatch) -> Result<Outcome, crate::data::error::SignalError>,
    ) -> TransitionOutcome<
        ExecutedInvalidationBatch<Outcome>,
        std::convert::Infallible,
        std::convert::Infallible,
        std::convert::Infallible,
        std::convert::Infallible,
        crate::data::error::SignalError,
    > {
        ExecutedInvalidationBatch::execute(ready, effect)
    }

    pub(crate) fn into_executed_outcome<Outcome>(
        executed: ExecutedInvalidationBatch<Outcome>,
    ) -> Outcome {
        executed.into_outcome()
    }
}

fn map_source(
    outcome: TransitionOutcome<AdmittedSourceRecompute, SourceAdmissionDenial>,
) -> InvalidationOriginAdmissionOutcome {
    match outcome {
        TransitionOutcome::Success(admitted) => {
            TransitionOutcome::success(ResolvedInvalidationWork::from_source(admitted))
        }
        TransitionOutcome::Denied(SourceAdmissionDenial::RebindRequired) => {
            TransitionOutcome::rebind_required(InvalidationProgressionDenial::RebindRequired)
        }
        TransitionOutcome::Denied(SourceAdmissionDenial::StaleRevision) => {
            TransitionOutcome::stale(InvalidationProgressionDenial::StaleDependencyRevision)
        }
        TransitionOutcome::Denied(SourceAdmissionDenial::StaleGeneration) => {
            TransitionOutcome::stale(InvalidationProgressionDenial::StaleOriginGeneration)
        }
        TransitionOutcome::Denied(SourceAdmissionDenial::WrongOrigin) => {
            TransitionOutcome::denied(InvalidationProgressionDenial::ContractRejected)
        }
        TransitionOutcome::Deferred(never)
        | TransitionOutcome::Stale(never)
        | TransitionOutcome::RebindRequired(never)
        | TransitionOutcome::Failed(never) => match never {},
    }
}

fn map_dependency(
    outcome: TransitionOutcome<AdmittedDependencyRecompute, DependencyAdmissionDenial>,
) -> InvalidationOriginAdmissionOutcome {
    match outcome {
        TransitionOutcome::Success(admitted) => {
            TransitionOutcome::success(ResolvedInvalidationWork::from_dependency(admitted))
        }
        TransitionOutcome::Denied(DependencyAdmissionDenial::RebindRequired) => {
            TransitionOutcome::rebind_required(InvalidationProgressionDenial::RebindRequired)
        }
        TransitionOutcome::Denied(DependencyAdmissionDenial::StaleRevision) => {
            TransitionOutcome::stale(InvalidationProgressionDenial::StaleDependencyRevision)
        }
        TransitionOutcome::Denied(DependencyAdmissionDenial::StaleGraphInstance) => {
            TransitionOutcome::stale(InvalidationProgressionDenial::StaleGraphInstance)
        }
        TransitionOutcome::Denied(DependencyAdmissionDenial::StaleCommitOrdinals) => {
            TransitionOutcome::stale(InvalidationProgressionDenial::StaleOriginGeneration)
        }
        TransitionOutcome::Denied(DependencyAdmissionDenial::WrongOrigin) => {
            TransitionOutcome::denied(InvalidationProgressionDenial::ContractRejected)
        }
        TransitionOutcome::Deferred(never)
        | TransitionOutcome::Stale(never)
        | TransitionOutcome::RebindRequired(never)
        | TransitionOutcome::Failed(never) => match never {},
    }
}

fn map_structural(
    outcome: TransitionOutcome<AdmittedStructuralRecompute, StructuralAdmissionDenial>,
) -> InvalidationOriginAdmissionOutcome {
    match outcome {
        TransitionOutcome::Success(admitted) => {
            TransitionOutcome::success(ResolvedInvalidationWork::from_structural(admitted))
        }
        TransitionOutcome::Denied(StructuralAdmissionDenial::RebindRequired) => {
            TransitionOutcome::rebind_required(InvalidationProgressionDenial::RebindRequired)
        }
        TransitionOutcome::Denied(StructuralAdmissionDenial::StaleRevision) => {
            TransitionOutcome::stale(InvalidationProgressionDenial::StaleDependencyRevision)
        }
        TransitionOutcome::Denied(StructuralAdmissionDenial::StaleMutation) => {
            TransitionOutcome::stale(InvalidationProgressionDenial::StaleOriginGeneration)
        }
        TransitionOutcome::Denied(StructuralAdmissionDenial::WrongOrigin) => {
            TransitionOutcome::denied(InvalidationProgressionDenial::ContractRejected)
        }
        TransitionOutcome::Deferred(never)
        | TransitionOutcome::Stale(never)
        | TransitionOutcome::RebindRequired(never)
        | TransitionOutcome::Failed(never) => match never {},
    }
}
