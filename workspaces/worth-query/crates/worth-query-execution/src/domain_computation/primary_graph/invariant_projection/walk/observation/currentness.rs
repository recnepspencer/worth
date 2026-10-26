//! One complete Native comparison at the actual selected current snapshot.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure as Failure;
use worth_relational::facade::runtime::{
    PinnedStructuralWalkDenialKind as Denial, PinnedStructuralWalkScope,
    RelationalPinnedReadAdmissionDenial,
};

impl StructuralWalkObservation {
    pub(in crate::domain_computation::primary_graph) fn remains_equal(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
    ) -> Result<bool, Failure> {
        let result = execution.run_native_with_checkpoint(|context| {
            let pin = runtime
                .read_truth()
                .pin_read_capability(snapshot, &[], execution)
                .map_err(|stop| match stop {
                    RelationalPinnedReadAdmissionDenial::Request(stop) => {
                        Failure::Execution(stop.into())
                    }
                    _ => Failure::Unavailable,
                })?;
            let current = match pin.complete_relation_walk(
                execution,
                PinnedStructuralWalkScope::new(
                    self.query.roots(),
                    self.query.entity_kind(),
                    self.query.relation_kind(),
                    self.query.direction(),
                ),
            ) {
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
                return Ok(false);
            }
            for (before, after) in self.native.visits().iter().zip(current.visits()) {
                Self::checkpoint(context).map_err(Failure::Execution)?;
                if before != after {
                    return Ok(false);
                }
            }
            for (before, after) in self.native.relations().iter().zip(current.relations()) {
                Self::checkpoint(context).map_err(Failure::Execution)?;
                if before != after {
                    return Ok(false);
                }
            }
            Ok(true)
        });
        match result {
            Ok((value, _)) => value,
            Err(stop) => Err(Failure::Execution(stop.into())),
        }
    }
}
