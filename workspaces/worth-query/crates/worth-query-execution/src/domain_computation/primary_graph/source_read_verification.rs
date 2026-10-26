//! Completed exact-Arc comparisons live only in one selected-pin verification extent.
use super::{
    aggregate_observation::AggregateObservation,
    structural_walk::{memory, StructuralWalkObservation},
};
use crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure as Failure;
use std::collections::BTreeMap;
use std::sync::Arc;
use worth_execution::ExecutionMemoryReservation;
use worth_relational::facade::runtime::{RelationalPinnedRead, RelationalRuntime};
use worth_relational::facade::{
    snapshots::SnapshotHandle, transactions::RelationalExecutionDenialCause as Cause,
};

pub(in crate::domain_computation::primary_graph) struct SourceReadVerification<'runtime> {
    runtime: &'runtime RelationalRuntime,
    pin: RelationalPinnedRead,
    completed: BTreeMap<usize, (Arc<StructuralWalkObservation>, bool)>,
    aggregates: BTreeMap<usize, (Arc<AggregateObservation>, bool)>,
    bytes: u64,
    _memory: Option<ExecutionMemoryReservation>,
}
impl<'runtime> SourceReadVerification<'runtime> {
    pub(in crate::domain_computation::primary_graph) fn open(
        runtime: &'runtime RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
    ) -> Result<Self, Failure> {
        let result = execution.run_native_with_checkpoint(|context| {
            StructuralWalkObservation::checkpoint(context).map_err(Failure::Execution)?;
            runtime.read_truth().pin_read_capability(snapshot, &[], execution)
                .map_err(|stop| match stop {
                    worth_relational::facade::runtime::RelationalPinnedReadAdmissionDenial::Request(stop) => Failure::Execution(stop.into()),
                    _ => Failure::Unavailable,
                })
        });
        let pin = match result {
            Ok((value, _)) => value?,
            Err(stop) => return Err(Failure::Execution(stop.into())),
        };
        Ok(Self {
            runtime,
            pin,
            completed: BTreeMap::new(),
            aggregates: BTreeMap::new(),
            bytes: 0,
            _memory: None,
        })
    }

    pub(in crate::domain_computation::primary_graph) fn answers_image(
        &self,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
    ) -> bool {
        std::ptr::eq(self.runtime, runtime) && self.pin.snapshot() == snapshot
    }

    pub(in crate::domain_computation::primary_graph) fn check_walk(
        &mut self,
        body: &Arc<StructuralWalkObservation>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        control: Option<&dyn Fn() -> Result<(), Cause>>,
    ) -> Result<bool, Failure> {
        let result = execution.run_native_with_checkpoint(|context| {
            StructuralWalkObservation::checkpoint(context).map_err(Failure::Execution)?;
            if let Some(control) = control {
                control().map_err(Failure::Execution)?;
            }
            let address = Arc::as_ptr(body) as usize;
            let equal = if let Some((original, equal)) = self.completed.get(&address) {
                if !Arc::ptr_eq(original, body) {
                    return Err(Failure::Unavailable);
                }
                *equal
            } else {
                let equal = body.remains_equal_at(&self.pin, execution, context, control)?;
                memory::grow_tree::<(usize, (Arc<StructuralWalkObservation>, bool))>(
                    &mut self._memory,
                    &mut self.bytes,
                    self.completed.len(),
                    execution,
                    context.has_accounting(),
                )
                .map_err(Failure::Execution)?;
                self.completed.insert(address, (Arc::clone(body), equal));
                equal
            };
            StructuralWalkObservation::checkpoint(context).map_err(Failure::Execution)?;
            if let Some(control) = control {
                control().map_err(Failure::Execution)?;
            }
            Ok(equal)
        });
        match result {
            Ok((value, _)) => value,
            Err(stop) => Err(Failure::Execution(stop.into())),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn check_aggregate(
        &mut self,
        body: &Arc<AggregateObservation>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        control: Option<&dyn Fn() -> Result<(), Cause>>,
    ) -> Result<bool, Failure> {
        let result = execution.run_native_with_checkpoint(|context| {
            AggregateObservation::checkpoint(context).map_err(Failure::Execution)?;
            if let Some(control) = control {
                control().map_err(Failure::Execution)?;
            }
            let address = Arc::as_ptr(body) as usize;
            let equal = if let Some((original, equal)) = self.aggregates.get(&address) {
                if !Arc::ptr_eq(original, body) {
                    return Err(Failure::Unavailable);
                }
                *equal
            } else {
                let equal = body.remains_equal_at(&self.pin, execution, context, control)?;

                self.aggregates.insert(address, (Arc::clone(body), equal));
                equal
            };
            AggregateObservation::checkpoint(context).map_err(Failure::Execution)?;
            if let Some(control) = control {
                control().map_err(Failure::Execution)?;
            }
            Ok(equal)
        });
        match result {
            Ok((value, _)) => value,
            Err(stop) => Err(Failure::Execution(stop.into())),
        }
    }
}
