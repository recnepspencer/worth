//! Compare complete truth at one genuinely admitted selected Native image.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure as Failure;
use worth_relational::facade::runtime::{
    PinnedStructuralWalkDenialKind as Denial, PinnedStructuralWalkScope, RelationalPinnedRead,
    RelationalPinnedReadAdmissionDenial, RelationalRuntime,
};

pub(super) fn pin_at(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &SnapshotHandle,
) -> Result<RelationalPinnedRead, Failure> {
    runtime
        .read_truth()
        .pin_read_capability(snapshot, &[], execution)
        .map_err(|stop| match stop {
            RelationalPinnedReadAdmissionDenial::Request(stop) => Failure::Execution(stop.into()),
            _ => Failure::Unavailable,
        })
}

impl StructuralWalkObservation {
    pub(in crate::domain_computation::primary_graph) fn remains_equal(
        &self,
        runtime: &RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
    ) -> Result<bool, Failure> {
        let result = execution.run_native_with_checkpoint(|context| {
            let pin = pin_at(runtime, execution, snapshot)?;
            self.remains_equal_at(&pin, execution, context, None)
        });
        match result {
            Ok((value, _)) => value,
            Err(stop) => Err(Failure::Execution(stop.into())),
        }
    }

    pub(super) fn remains_equal_at(
        &self,
        pin: &RelationalPinnedRead,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        context: &mut MapKernelContext<'_, '_>,
        control: Option<&dyn Fn() -> Result<(), Cause>>,
    ) -> Result<bool, Failure> {
        let mut poll = || {
            Self::checkpoint(context).map_err(Failure::Execution)?;
            if let Some(control) = control {
                control().map_err(Failure::Execution)?;
            }
            Ok::<(), Failure>(())
        };
        poll()?;
        // Both the original body and selected pin are owner-issued actual truth.
        // Admission above is mandatory even for this immutable-source shortcut.
        if self.source() == pin.snapshot() {
            poll()?;
            return Ok(true);
        }
        let mut scope = PinnedStructuralWalkScope::new(
            self.query.roots(),
            self.query.entity_kind(),
            self.query.relation_kind(),
            self.query.direction(),
        );
        if let Some(control) = control {
            scope = scope.with_control(control);
        }
        let current = match pin.complete_relation_walk(execution, scope) {
            Ok(current) => current,
            Err(stop) => {
                return match stop.kind {
                    Denial::Read(cause) => Err(Failure::Execution(cause)),
                    Denial::EndpointUnavailable
                    | Denial::WrongEntityKind
                    | Denial::KindUnavailable => Ok(false),
                    Denial::WorkLimitExceeded | Denial::DistanceOverflow => {
                        Err(Failure::WorkBudgetExceeded)
                    }
                }
            }
        };
        if current.visits().len() != self.native.visits().len()
            || current.relations().len() != self.native.relations().len()
        {
            poll()?;
            return Ok(false);
        }
        for (before, after) in self.native.visits().iter().zip(current.visits()) {
            poll()?;
            if before != after {
                return Ok(false);
            }
        }
        for (before, after) in self.native.relations().iter().zip(current.relations()) {
            poll()?;
            if before != after {
                return Ok(false);
            }
        }
        poll()?;
        Ok(true)
    }
}
