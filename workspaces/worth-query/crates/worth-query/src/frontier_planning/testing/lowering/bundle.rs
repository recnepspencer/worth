use super::super::{
    frontier_input_kind, FrontierBundlePlan, FrontierPlanningCounters, FrontierPlanningError,
    FrontierPlanningInput,
};
use super::{lower_live_plan_to_frontier_plan, lower_preflight_to_frontier_plan};

pub(crate) fn lower_frontier_bundle(
    inputs: &[FrontierPlanningInput],
) -> Result<FrontierBundlePlan, FrontierPlanningError> {
    if inputs.is_empty() {
        return Err(FrontierPlanningError::UnsupportedBundleComposition);
    }

    let first_input_kind = frontier_input_kind(&inputs[0]);
    if inputs
        .iter()
        .skip(1)
        .any(|input| frontier_input_kind(input) != first_input_kind)
    {
        return Err(FrontierPlanningError::UnsupportedBundleComposition);
    }

    let mut route_plans = Vec::with_capacity(inputs.len());
    for input in inputs {
        let plan = match input {
            FrontierPlanningInput::ExecutionPreflight(preflight) => {
                lower_preflight_to_frontier_plan(preflight)
            }
            FrontierPlanningInput::LivePlan(live) => lower_live_plan_to_frontier_plan(live),
        }
        .map_err(|err| match err {
            FrontierPlanningError::UnsupportedFrontierFamily => {
                FrontierPlanningError::UnsupportedBundleComposition
            }
            other => other,
        })?;
        route_plans.push(plan);
    }

    let expected_basis = route_plans[0].bundle_basis_digest().clone();
    for route_plan in route_plans.iter().skip(1) {
        if route_plan.bundle_basis_digest() != &expected_basis {
            return Err(FrontierPlanningError::MixedBasisBundle {
                expected_basis_digest: expected_basis.clone(),
                found_basis_digest: route_plan.bundle_basis_digest().clone(),
            });
        }
    }

    Ok(FrontierBundlePlan {
        bundle_basis_digest: route_plans[0].bundle_basis_digest().clone(),
        counters: FrontierPlanningCounters {
            frontier_planning_invocation_count: 1,
            planned_bundle_route_count: route_plans.len(),
            predicted_breadth: route_plans
                .iter()
                .map(|route| route.predicted_breadth().value())
                .sum(),
            planned_packet_merge_boundary_count: route_plans
                .iter()
                .map(|route| route.packet_set().packets().len())
                .sum(),
        },
        route_plans,
    })
}
