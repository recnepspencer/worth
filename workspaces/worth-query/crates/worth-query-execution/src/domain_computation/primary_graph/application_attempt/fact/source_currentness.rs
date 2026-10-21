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
        match self.comparison(runtime, execution, snapshot) {
            Ok(equal) => Ok(Ok(FactMovement::from_equal(equal))),
            Err(WorthQuerySourceCurrentnessFailure::Execution(cause)) => {
                Err(CompanionPreflightStop::Execution(cause))
            }
            Err(stop) => Ok(Err(stop)),
        }
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
