use crate::schema::data::AspectContractPlanCatalog;
use crate::visibility::materialization::read_records::projection::VisibilityProjectionView;

use super::admission::{self, ObservationResult};
use super::{
    RelationalAuthorizationBudgetedObservationStop as Stop,
    RelationalAuthorizationObservationAdmission, RelationalAuthorizationObservationCounters,
    RelationalAuthorizationObservationDenial, RelationalAuthorizationObservationPlan,
    RelationalAuthorizationPathObservation,
};

pub(super) fn evaluate<A: RelationalAuthorizationObservationAdmission>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    aspect_plans: &AspectContractPlanCatalog,
    view: &VisibilityProjectionView,
    plan: &RelationalAuthorizationObservationPlan,
    admission: &mut A,
) -> ObservationResult<RelationalAuthorizationEvaluation, A> {
    admission::prepare(admission, 1, 0)?;
    let principal = view
        .read_exact_entity_state(execution, plan.principal(), |metadata, _| {
            metadata.kind_id == plan.principal_kind()
        })
        .map_err(admission::borrowed_denial)?
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
            .read_exact_entity_state(execution, plan.scope(), |metadata, _| {
                metadata.kind_id == plan.scope_kind()
            })
            .map_err(admission::borrowed_denial)?
            .unwrap_or(false);
        counters.entity_records_inspected += 1;
        if !scope {
            return Err(Stop::Native(
                RelationalAuthorizationObservationDenial::ScopeUnavailableOrWrongKind,
            ));
        }
    }
    admission::array::<A, RelationalAuthorizationPathObservation>(admission, plan.paths().len())?;
    let mut paths = Vec::with_capacity(plan.paths().len());
    for path in plan.paths() {
        paths.push(super::path_evaluation::admitted::evaluate_path(
            execution,
            aspect_plans,
            view,
            plan,
            path,
            &mut counters,
            admission,
        )?);
    }
    Ok(RelationalAuthorizationEvaluation { paths, counters })
}

pub(super) struct RelationalAuthorizationEvaluation {
    pub(super) paths: Vec<RelationalAuthorizationPathObservation>,
    pub(super) counters: RelationalAuthorizationObservationCounters,
}
