//! Fresh Native member selection at the genuinely admitted comparison image.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure as Failure;
use worth_relational::facade::runtime::{
    PinnedAggregateReadDenialKind as Denial, RelationalPinnedReadAdmissionDenial, RelationalRuntime,
};
impl AggregateObservation {
    pub(in crate::domain_computation::primary_graph) fn remains_equal(
        &self,
        runtime: &RelationalRuntime,
        execution: &ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
        control: Option<&dyn Fn() -> Result<(), Cause>>,
    ) -> Result<bool, Failure> {
        let result = execution.run_native_with_checkpoint(|context| {
            let mut poll = || {
                Self::checkpoint(context).map_err(Failure::Execution)?;
                if let Some(control) = control {
                    control().map_err(Failure::Execution)?;
                }
                Ok::<(), Failure>(())
            };
            poll()?;
            let pin = runtime
                .read_truth()
                .pin_read_capability(snapshot, &[], execution)
                .map_err(|stop| match stop {
                    RelationalPinnedReadAdmissionDenial::Request(stop) => {
                        Failure::Execution(WorkCeilingDenial::Stopped(stop).into())
                    }
                    _ => Failure::Unavailable,
                })?;
            if self.source() == pin.snapshot() {
                poll()?;
                return Ok(true);
            }
            let mut scope =
                PinnedAggregateReadScope::new(self.key.relation_kind(), self.key.member());
            if let Some(control) = control {
                scope = scope.with_control(control);
            }
            let current = match pin.read_aggregate(execution, scope) {
                Ok(value) => value,
                Err(stop) => {
                    return match stop.kind {
                        Denial::Read(cause) => Err(Failure::Execution(cause)),
                        Denial::MemberUnavailable
                        | Denial::WrongMemberKind
                        | Denial::NotAggregateRelation
                        | Denial::DefinitionUnavailable => Ok(false),
                        _ => Err(Failure::Unavailable),
                    }
                }
            };
            poll()?;
            if current.definition() != self.native.definition()
                || current.identity() != self.native.identity()
                || current.len() != self.native.len()
            {
                return Ok(false);
            }
            for (old, now) in self.native.members().zip(current.members()) {
                poll()?;
                if old != now {
                    return Ok(false);
                }
            }
            poll()?;
            Ok(true)
        });
        match result {
            Ok((value, _)) => value,
            Err(stop) => Err(Failure::Execution(stop.into())),
        }
    }
}
