use crate::runtime::RelationalRuntime;

use super::admission::{self, ObservationResult, UnrestrictedObservation};
use super::observation_identity::mint_observation_identity;
use super::{
    RelationalAuthorizationBudgetedObservationStop as Stop,
    RelationalAuthorizationObservationAdmission, RelationalAuthorizationObservationCounters,
    RelationalAuthorizationObservationDenial, RelationalAuthorizationObservationEvidence,
    RelationalAuthorizationObservationPlan, RelationalAuthorizationPathObservation,
};

impl RelationalRuntime {
    pub fn observe_authorization_budgeted<A: RelationalAuthorizationObservationAdmission>(
        &self,
        plan: RelationalAuthorizationObservationPlan,
        admission: &mut A,
    ) -> Result<RelationalAuthorizationObservationEvidence, Stop<A::Stop>> {
        let evaluation = self.evaluate_authorization_admitted(&plan, admission)?;
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
    ) -> Result<RelationalAuthorizationObservationEvidence, RelationalAuthorizationObservationDenial>
    {
        self.observe_authorization_budgeted(plan, &mut UnrestrictedObservation)
            .map_err(admission::legacy_stop)
    }

    pub(super) fn evaluate_authorization_admitted<
        A: RelationalAuthorizationObservationAdmission,
    >(
        &self,
        plan: &RelationalAuthorizationObservationPlan,
        admission: &mut A,
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
        admission::prepare(admission, 1, 0)?;
        let principal = view
            .with_exact_entity_state(plan.principal(), |metadata, _| {
                metadata.kind_id == plan.principal_kind()
            })
            .map_err(|_| Stop::ExactBasisRequired)?
            .unwrap_or(false);
        if !principal {
            return Err(Stop::Native(
                RelationalAuthorizationObservationDenial::PrincipalUnavailableOrWrongKind,
            ));
        }
        let mut counters = RelationalAuthorizationObservationCounters {
            entity_records_inspected: 1,
            ..Default::default()
        };
        if plan.scope() != plan.principal() {
            admission::prepare(admission, 1, 0)?;
            let scope = view
                .with_exact_entity_state(plan.scope(), |metadata, _| {
                    metadata.kind_id == plan.scope_kind()
                })
                .map_err(|_| Stop::ExactBasisRequired)?
                .unwrap_or(false);
            counters.entity_records_inspected += 1;
            if !scope {
                return Err(Stop::Native(
                    RelationalAuthorizationObservationDenial::ScopeUnavailableOrWrongKind,
                ));
            }
        }
        admission::array::<A, RelationalAuthorizationPathObservation>(
            admission,
            plan.paths().len(),
        )?;
        let mut paths = Vec::with_capacity(plan.paths().len());
        for path in plan.paths() {
            paths.push(super::path_evaluation::admitted::evaluate_path(
                self,
                &view,
                plan,
                path,
                &mut counters,
                admission,
            )?);
        }
        Ok(RelationalAuthorizationEvaluation { paths, counters })
    }
}

pub(super) struct RelationalAuthorizationEvaluation {
    pub(super) paths: Vec<RelationalAuthorizationPathObservation>,
    pub(super) counters: RelationalAuthorizationObservationCounters,
}
