//! Admit the selected snapshot and bind its authorization evaluation operands.
use crate::runtime::RelationalRuntime;

use super::admission::{self, ObservationResult, UnrestrictedObservation};
use super::observation_identity::mint_observation_identity;
use super::{
    RelationalAuthorizationBudgetedObservationStop as Stop,
    RelationalAuthorizationObservationAdmission, RelationalAuthorizationObservationDenial,
    RelationalAuthorizationObservationEvidence, RelationalAuthorizationObservationPlan,
};

use super::evaluation::RelationalAuthorizationEvaluation;
use crate::capabilities::AspectPlanSource;

impl RelationalRuntime {
    pub fn observe_authorization_budgeted<A: RelationalAuthorizationObservationAdmission>(
        &self,
        plan: RelationalAuthorizationObservationPlan,
        admission: &mut A,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<RelationalAuthorizationObservationEvidence, Stop<A::Stop>> {
        execution.check_live().map_err(|stop| {
            Stop::Native(RelationalAuthorizationObservationDenial::Execution(
                stop.into(),
            ))
        })?;
        let evaluation = self.evaluate_authorization_admitted(&plan, admission, execution)?;
        admission::prepare(admission, 1, 0)?;
        let identity = mint_observation_identity(&plan).ok_or(Stop::Native(
            RelationalAuthorizationObservationDenial::ObservationIdentityExhausted,
        ))?;
        Ok(RelationalAuthorizationObservationEvidence::mint(
            plan,
            identity,
            evaluation.paths,
            evaluation.counters,
        ))
    }

    pub fn observe_authorization(
        &self,
        plan: RelationalAuthorizationObservationPlan,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<RelationalAuthorizationObservationEvidence, RelationalAuthorizationObservationDenial>
    {
        self.observe_authorization_budgeted(plan, &mut UnrestrictedObservation, execution)
            .map_err(admission::legacy_stop)
    }

    pub(super) fn evaluate_authorization_admitted<
        A: RelationalAuthorizationObservationAdmission,
    >(
        &self,
        plan: &RelationalAuthorizationObservationPlan,
        admission: &mut A,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> ObservationResult<RelationalAuthorizationEvaluation, A> {
        admission::prepare(admission, 1, 0)?;
        if plan.snapshot().runtime_instance_id != self.runtime_instance_id() {
            return Err(Stop::Native(
                RelationalAuthorizationObservationDenial::ForeignRuntime {
                    expected_runtime_instance_id: self.runtime_instance_id(),
                    actual_runtime_instance_id: plan.snapshot().runtime_instance_id,
                },
            ));
        }
        let view = self
            .read_truth()
            .project_snapshot(plan.snapshot())
            .ok_or(Stop::Native(
                RelationalAuthorizationObservationDenial::SnapshotUnavailable,
            ))?;
        super::evaluation::evaluate(
            execution,
            self.aspect_plan_catalog(),
            &view,
            plan,
            admission,
        )
    }
}
