//! Reacquire through the same Native owner at a genuinely admitted selected pin.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::source_currentness::WorthQuerySourceCurrentnessFailure as Failure;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    runtime::{
        DerivedRelationReadDenial as Denial, DerivedRelationReadShape, PinnedDerivedRelationScope,
        RelationalPinnedRead, RelationalPinnedReadAdmissionDenial, RelationalRuntime,
        StructuralReadError, StructuralWalkDenialKind,
    },
    schema::DerivedRelationDenial as Definition,
};
impl DerivedSourceObservation {
    pub(in crate::domain_computation::primary_graph) fn remains_equal(
        &self,
        runtime: &RelationalRuntime,
        execution: &ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
    ) -> Result<bool, Failure> {
        let pin = runtime
            .read_truth()
            .pin_read_capability(snapshot, &[], execution)
            .map_err(|stop| match stop {
                RelationalPinnedReadAdmissionDenial::Request(cause) => Failure::Execution(cause),
                _ => Failure::Unavailable,
            })?;
        let result = execution.run_native_with_checkpoint(|context| {
            self.remains_equal_at(&pin, execution, context, None)
        });
        match result {
            Ok((value, _)) => value,
            Err(stop) => Err(Failure::Execution(stop.into())),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn remains_equal_at(
        &self,
        pin: &RelationalPinnedRead,
        execution: &ExecutionRequest<'_, '_>,
        context: &mut MapKernelContext<'_, '_>,
        control: Option<&dyn Fn() -> Result<(), Cause>>,
    ) -> Result<bool, Failure> {
        Self::checkpoint(context).map_err(Failure::Execution)?;
        if let Some(control) = control {
            control().map_err(Failure::Execution)?;
        }
        if self.source().runtime_instance_id() != pin.snapshot().runtime_instance_id() {
            return Err(Failure::Unavailable);
        }
        let mut scope = PinnedDerivedRelationScope::new(
            &self.definition,
            self.key.root(),
            DerivedRelationReadShape::Adjacency {
                reverse: self.key.direction()
                    == worth_foundational::facade::RelationDirection::Reverse,
            },
        );
        if let Some(control) = control {
            scope = scope.with_control(control);
        }
        let current = match pin.read_derived_source(execution, scope) {
            Ok(body) => body,
            Err(stop) => return match stop.kind {
                Denial::Request(cause) => Err(Failure::Execution(cause)),
                Denial::Definition(Definition::WorkBudgetExceeded)
                    | Denial::Structural(StructuralWalkDenialKind::Read(StructuralReadError::WorkBudgetExceeded)) => Err(Failure::WorkBudgetExceeded),
                Denial::Structural(StructuralWalkDenialKind::Read(_)) | Denial::Definition(_) => Ok(false),
                Denial::Structural(StructuralWalkDenialKind::Interrupted(stop)) => Err(Failure::Execution(match stop { worth_relational::facade::runtime::CustomInvariantInterruption::Cancelled => Cause::Cancelled, worth_relational::facade::runtime::CustomInvariantInterruption::DeadlineElapsed => Cause::DeadlineElapsed })),
                Denial::Structural(StructuralWalkDenialKind::MemoryBudgetExceeded) => Err(Failure::Unavailable),
                Denial::Structural(StructuralWalkDenialKind::MemoryAdmission(stop)) => Err(Failure::Execution(Cause::from(worth_execution::WorkCeilingDenial::Admission(worth_execution::LeaseDenial::MemoryExhausted(stop))))),
            },
        };
        same_data(self.native.data(), current.data(), || {
            Self::checkpoint(context).map_err(Failure::Execution)?;
            if let Some(control) = control {
                control().map_err(Failure::Execution)?;
            }
            Ok::<_, Failure>(())
        })
    }
}
