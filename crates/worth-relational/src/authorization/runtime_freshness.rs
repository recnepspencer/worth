use crate::runtime::RelationalRuntime;
use crate::snapshots::data::SnapshotHandle;

use super::{
    admission::RequestObservation, RelationalAuthorizationBudgetedObservationStop as Stop,
    RelationalAuthorizationObservationAdmission, RelationalAuthorizationObservationEvidence,
    RelationalAuthorizationObservationFreshness, RelationalAuthorizationPlanAdmissionStop,
};

impl RelationalRuntime {
    pub fn compare_authorization_observation_budgeted<
        A: RelationalAuthorizationObservationAdmission,
    >(
        &self,
        expected: &RelationalAuthorizationObservationEvidence,
        snapshot: &SnapshotHandle,
        admission: &mut A,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<RelationalAuthorizationObservationFreshness, Stop<A::Stop>> {
        let plan = match expected
            .comparison_plan_admitted(snapshot, |work, bytes| admission.prepare(work, bytes))
        {
            Ok(plan) => plan,
            Err(RelationalAuthorizationPlanAdmissionStop::Plan(_)) => {
                return Ok(RelationalAuthorizationObservationFreshness::Stale);
            }
            Err(RelationalAuthorizationPlanAdmissionStop::Admission(stop)) => {
                return Err(Stop::Admission(stop));
            }
            Err(RelationalAuthorizationPlanAdmissionStop::AccountingOverflow) => {
                return Err(Stop::AccountingOverflow);
            }
        };
        let current = match self.evaluate_authorization_admitted(&plan, admission, execution) {
            Ok(current) => current,
            Err(Stop::Native(super::RelationalAuthorizationObservationDenial::Execution(
                cause,
            ))) => {
                return Err(Stop::Native(
                    super::RelationalAuthorizationObservationDenial::Execution(cause),
                ));
            }
            Err(Stop::Native(_) | Stop::ExactBasisRequired) => {
                return Ok(RelationalAuthorizationObservationFreshness::Stale);
            }
            Err(Stop::Admission(stop)) => return Err(Stop::Admission(stop)),
            Err(Stop::Execution(cause)) => return Err(Stop::Execution(cause)),
            Err(Stop::AccountingOverflow) => return Err(Stop::AccountingOverflow),
        };
        super::freshness::compare(expected, current, admission)
    }

    pub fn compare_authorization_observation(
        &self,
        expected: &RelationalAuthorizationObservationEvidence,
        snapshot: SnapshotHandle,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> RelationalAuthorizationObservationFreshness {
        match self.compare_authorization_observation_budgeted(
            expected,
            &snapshot,
            &mut RequestObservation(execution),
            execution,
        ) {
            Ok(freshness) => freshness,
            Err(Stop::Execution(cause) | Stop::Admission(cause)) => {
                RelationalAuthorizationObservationFreshness::ExecutionStopped(cause)
            }
            Err(Stop::AccountingOverflow) => {
                RelationalAuthorizationObservationFreshness::ExecutionStopped(
                    crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow,
                )
            }
            Err(Stop::Native(super::RelationalAuthorizationObservationDenial::Execution(
                cause,
            ))) => RelationalAuthorizationObservationFreshness::ExecutionStopped(cause),
            Err(Stop::Native(_) | Stop::ExactBasisRequired) => {
                RelationalAuthorizationObservationFreshness::Stale
            }
        }
    }
}
