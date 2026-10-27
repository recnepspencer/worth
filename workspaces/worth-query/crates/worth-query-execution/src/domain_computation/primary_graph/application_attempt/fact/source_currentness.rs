use worth_relational::facade::mvcc::CompanionPreflightStop;
mod comparison;
use super::{FactMovement, WorthQueryApplicationObservedFact};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorthQuerySourceCurrentnessFailure {
    /// The fact's own recorded bound refuses the comparison.
    WorkBudgetExceeded,
    Execution(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    Unavailable,
    OutputObservation(crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerificationStop),
}

impl WorthQueryApplicationObservedFact {
    /// The extent retains actual bodies and its one admitted pin. It cannot
    /// answer for another runtime/image or omit an original control poll.
    pub(in crate::domain_computation::primary_graph) fn source_currentness_with<'runtime>(
        &self,
        runtime: &'runtime worth_relational::facade::runtime::RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        reads: &mut Option<
            crate::domain_computation::primary_graph::source_read_verification::SourceReadVerification<
                'runtime,
            >,
        >,
        control: Option<
            &dyn Fn() -> Result<
                (),
                worth_relational::facade::transactions::RelationalExecutionDenialCause,
            >,
        >,
    ) -> Result<Result<FactMovement, WorthQuerySourceCurrentnessFailure>, CompanionPreflightStop>
    {
        if let Some(control) = control {
            control().map_err(CompanionPreflightStop::Execution)?;
        }
        if !matches!(self, Self::Aggregate { .. } | Self::StructuralWalk { .. }) {
            return self.source_currentness_in(runtime, execution, snapshot);
        }
        check_request(execution)?;
        super::super::super::fact_verification_observation::source_currentness();
        if reads.is_none() {
            match crate::domain_computation::primary_graph::source_read_verification::SourceReadVerification::open(runtime, execution, snapshot) {
                Ok(value) => *reads = Some(value),
                Err(stop) => return comparison_result(Err(stop)),
            }
        }
        let reads = reads
            .as_mut()
            .expect("the extent admitted its selected image");
        if !reads.answers_image(runtime, snapshot) {
            return Ok(Err(WorthQuerySourceCurrentnessFailure::Unavailable));
        }
        comparison_result(match self {
            Self::Aggregate { observation } => {
                reads.check_aggregate(observation, execution, control)
            }
            Self::StructuralWalk { observation } => {
                reads.check_walk(observation, execution, control)
            }
            _ => unreachable!("other facts were handled before the read extent"),
        })
    }

    /// Compare at the genuine selected snapshot under the original execution request.
    pub(in crate::domain_computation::primary_graph) fn source_currentness_in(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<Result<FactMovement, WorthQuerySourceCurrentnessFailure>, CompanionPreflightStop>
    {
        check_request(execution)?;
        super::super::super::fact_verification_observation::source_currentness();
        if matches!(
            self,
            Self::Field { .. } | Self::AbsentField { .. } | Self::OptionalAspectAbsent { .. }
        ) {
            return Ok(Err(WorthQuerySourceCurrentnessFailure::Unavailable));
        }
        if let Self::Relation {
            relation_kind,
            from,
            to,
            matching_relations,
        } = self
        {
            let Some(view) = runtime.read_truth().project_snapshot(snapshot) else {
                return Ok(Err(WorthQuerySourceCurrentnessFailure::Unavailable));
            };
            let read = view
                .bounded_outgoing_relations_of_kind(execution, *from, *relation_kind, None)
                .map_err(CompanionPreflightStop::Execution)?
                .expect("no separate traversal quota");
            let records = read.into_records();
            let actual = records
                .iter()
                .filter(|relation| relation.target == *to)
                .map(|relation| relation.relation_id);
            return Ok(Ok(FactMovement::from_equal(
                actual.eq(matching_relations.iter().copied()),
            )));
        }
        comparison_result(self.comparison(runtime, execution, snapshot))
    }
}
fn comparison_result(
    result: Result<bool, WorthQuerySourceCurrentnessFailure>,
) -> Result<Result<FactMovement, WorthQuerySourceCurrentnessFailure>, CompanionPreflightStop> {
    match result {
        Ok(equal) => Ok(Ok(FactMovement::from_equal(equal))),
        Err(WorthQuerySourceCurrentnessFailure::Execution(cause)) => {
            Err(CompanionPreflightStop::Execution(cause))
        }
        Err(stop) => Ok(Err(stop)),
    }
}
fn check_request(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), CompanionPreflightStop> {
    if let Err(stop) = execution.consult() {
        if let worth_execution::WorkCeilingDenial::Stopped(stopped) = stop
            && let Ok(event) = worth_relational::facade::mvcc::RelationalInterruptionEvent::from_request_stop(
                stopped, worth_relational::facade::mvcc::RelationalInterruptionBoundary::ObservationAdmission,
            ) {
                return Err(CompanionPreflightStop::Interrupted(event));
            }
        return Err(CompanionPreflightStop::Execution(stop.into()));
    }
    Ok(())
}
