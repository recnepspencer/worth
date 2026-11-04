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
            Self::checkpoint(context).map_err(Failure::Execution)?;
            if let Some(control) = control {
                control().map_err(Failure::Execution)?;
            }
            let pin = runtime
                .read_truth()
                .pin_read_capability(snapshot, &[], execution)
                .map_err(|stop| match stop {
                    RelationalPinnedReadAdmissionDenial::Request(stop) => Failure::Execution(stop),
                    _ => Failure::Unavailable,
                })?;
            self.remains_equal_at(&pin, execution, context, control)
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
        self.remains_equal_on_pin(pin, pin.snapshot().into(), execution, context, control)
    }
    pub(in crate::domain_computation::primary_graph) fn remains_equal_on_pin<Phase>(
        &self,
        pin: &RelationalPinnedRead<Phase>,
        origin: crate::domain_computation::primary_graph::projection_read_origin::ProjectionReadOriginRef<'_>,
        execution: &ExecutionRequest<'_, '_>,
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
        if self.source() == &origin {
            poll()?;
            return Ok(true);
        }
        let mut scope = self.key.native_scope();
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
        Self::same_native(&self.native, &current, poll)
    }
    fn same_native(
        original: &PinnedAggregateRead,
        current: &PinnedAggregateRead,
        mut poll: impl FnMut() -> Result<(), Failure>,
    ) -> Result<bool, Failure> {
        if current.definition() != original.definition()
            || current.identity() != original.identity()
            || current.len() != original.len()
        {
            return Ok(false);
        }
        for (old, now) in original.members().zip(current.members()) {
            poll()?;
            if old != now {
                return Ok(false);
            }
        }
        poll()?;
        Ok(true)
    }
    pub(in crate::domain_computation::primary_graph) fn support_equal_between<A, B>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        left: &RelationalPinnedRead<A>,
        right: &RelationalPinnedRead<B>,
    ) -> Result<bool, Failure> {
        let answer = execution
            .run_native_with_checkpoint(|context| {
                let left = left
                    .read_aggregate(execution, self.key.native_scope())
                    .map_err(support_stop)?;
                let right = right
                    .read_aggregate(execution, self.key.native_scope())
                    .map_err(support_stop)?;
                Self::same_native(&left, &right, || {
                    Self::checkpoint(context).map_err(Failure::Execution)
                })
            })
            .map_err(|cause| Failure::Execution(cause.into()))?;
        answer.0
    }
}

pub(super) fn support_stop(stop: PinnedAggregateReadDenial) -> Failure {
    match stop.kind {
        Denial::Read(cause) => Failure::Execution(cause),
        _ => Failure::Unavailable,
    }
}
