use std::borrow::{Borrow, Cow};

use super::WorthUiAllocationReplanDenial;
use super::{
    construct_verified_planning_input_handoff,
    construct_verified_planning_input_handoff_from_projection,
};
use crate::evidence::{UiConstraintPropagationDenial, UiMeasurementBasis};
use crate::runtime::planning::allocation_planning::WorthUiAllocationPlanner;
use crate::runtime::planning::UiAllocationCandidateMintAuthority;
use crate::runtime::{
    UiAllocationCandidate, WorthUiAllocationPlanning, WorthUiAllocationPlanningBasis,
    WorthUiAllocationPlanningCounters, WorthUiAllocationPlanningDenial,
    WorthUiAllocationPlanningDenialReason, WorthUiPendingActivation,
};

pub(crate) enum ConstraintSetAdmissionDecision {
    Admitted(crate::graph::UiAdmittedAllocationConstraintBasis),
    Denied(UiConstraintPropagationDenial),
}

pub(crate) fn classify_constraint_set_admission(
    measurement_basis: &UiMeasurementBasis,
    allocation_neighborhood: &crate::evidence::UiAllocationNeighborhood,
) -> ConstraintSetAdmissionDecision {
    match measurement_basis.admit_allocation_constraint_basis(allocation_neighborhood) {
        Ok(constraint_set) => ConstraintSetAdmissionDecision::Admitted(constraint_set),
        Err(denial) => ConstraintSetAdmissionDecision::Denied(denial),
    }
}

pub(crate) fn build_constraint_set_denial_planning(
    measurement_basis: &UiMeasurementBasis,
    allocation_neighborhood: &crate::evidence::UiAllocationNeighborhood,
    constraint_set_denial: UiConstraintPropagationDenial,
) -> UiAllocationCandidate {
    let mut counters = WorthUiAllocationPlanningCounters::default();
    counters.record_planning_attempt();
    counters.record_measurement_basis_read();
    let basis = WorthUiAllocationPlanningBasis::denied(
        measurement_basis.clone(),
        allocation_neighborhood.clone(),
    );
    let denial_posture = Some(WorthUiAllocationPlanningDenial::new(
        WorthUiAllocationPlanningDenialReason::ConstraintSetDenied,
        None,
        Some(constraint_set_denial),
        counters,
    ));
    UiAllocationCandidate::from_planning(
        WorthUiAllocationPlanning::new(basis, None, denial_posture, counters),
        UiAllocationCandidateMintAuthority::new(),
    )
}

pub(crate) fn plan_allocation_for_pending_activation<P: Borrow<WorthUiPendingActivation>>(
    pending_activation: P,
    measurement_basis: &UiMeasurementBasis,
    allocation_neighborhood: &crate::evidence::UiAllocationNeighborhood,
) -> UiAllocationCandidate {
    let pending_activation = pending_activation.borrow();
    match classify_constraint_set_admission(measurement_basis, allocation_neighborhood) {
        ConstraintSetAdmissionDecision::Denied(constraint_set_denial) => {
            build_constraint_set_denial_planning(
                measurement_basis,
                allocation_neighborhood,
                constraint_set_denial,
            )
        }
        ConstraintSetAdmissionDecision::Admitted(constraint_basis) => {
            let handoff =
                construct_verified_planning_input_handoff(pending_activation, constraint_basis)
                    .expect("constraint admission must preserve graph-planning alignment");
            UiAllocationCandidate::from_planning(
                WorthUiAllocationPlanner::plan(handoff.into_admission()),
                UiAllocationCandidateMintAuthority::new(),
            )
        }
    }
}

pub(crate) fn plan_allocation_for_projection(
    projection: &crate::runtime::planning::allocation_planning::WorthUiAllocationPlanningProjection,
    measurement_basis: &UiMeasurementBasis,
    allocation_neighborhood: &crate::evidence::UiAllocationNeighborhood,
) -> UiAllocationCandidate {
    match classify_constraint_set_admission(measurement_basis, allocation_neighborhood) {
        ConstraintSetAdmissionDecision::Denied(constraint_set_denial) => {
            build_constraint_set_denial_planning(
                measurement_basis,
                allocation_neighborhood,
                constraint_set_denial,
            )
        }
        ConstraintSetAdmissionDecision::Admitted(constraint_basis) => {
            let handoff = construct_verified_planning_input_handoff_from_projection(
                projection,
                constraint_basis,
            )
            .expect("constraint admission must preserve graph-planning alignment");
            UiAllocationCandidate::from_planning(
                WorthUiAllocationPlanner::plan(handoff.into_admission()),
                UiAllocationCandidateMintAuthority::new(),
            )
        }
    }
}

pub(crate) fn replan_admitted_candidate(
    selected: &crate::graph::UiAdmittedReplanNeighborhood,
) -> Result<UiAllocationCandidate, WorthUiAllocationReplanDenial> {
    replan_admitted_candidate_with_sources(selected, None, None, None)
}

fn replan_admitted_candidate_with_sources(
    selected: &crate::graph::UiAdmittedReplanNeighborhood,
    portal: Option<crate::runtime::UiPortalAllocationPlanningBasis>,
    query: Option<&crate::graph::UiQueryMeasurementReplanConsequence>,
    host: Option<&crate::graph::UiHostMeasurementReplanConsequence>,
) -> Result<UiAllocationCandidate, WorthUiAllocationReplanDenial> {
    let previous = selected.allocation_candidate();
    let mut measurement_basis = Cow::Borrowed(previous.measurement_basis());
    if let Some(query) = query {
        if query.predecessor_basis_identity_digest()
            != previous.measurement_basis().identity_digest()
        {
            return Err(WorthUiAllocationReplanDenial::QueryMeasurementBasisMismatch);
        }
        measurement_basis = Cow::Owned(
            previous
                .measurement_basis()
                .succeed_settled_query_receipt(query.receipt())
                .map_err(|_| WorthUiAllocationReplanDenial::QueryMeasurementBasisMismatch)?,
        );
    }
    if let Some(host) = host {
        if host.predecessor_basis_identity_digest()
            != previous.measurement_basis().identity_digest()
        {
            return Err(WorthUiAllocationReplanDenial::HostMeasurementBasisMismatch);
        }
        measurement_basis = Cow::Owned(
            measurement_basis
                .succeed_host_measurement_result(host.measurement())
                .map_err(|_| WorthUiAllocationReplanDenial::HostMeasurementBasisMismatch)?,
        );
    }
    if let Some(portal) = portal.as_ref() {
        measurement_basis = Cow::Owned(
            measurement_basis
                .succeed_portal_measurement_result(portal.measurement_result())
                .map_err(|_| WorthUiAllocationReplanDenial::PortalMeasurementBasisMismatch)?,
        );
    }
    let measurement_basis = measurement_basis.as_ref();
    let neighborhood = previous.allocation_neighborhood();
    let constraint_basis = portal
        .as_ref()
        .map_or_else(
            || measurement_basis.admit_allocation_constraint_basis(neighborhood),
            |portal| {
                measurement_basis
                    .admit_allocation_constraint_basis_with_portal(neighborhood, portal)
            },
        )
        .map_err(|_| WorthUiAllocationReplanDenial::ConstraintSetDenied)?;
    let projection = previous
        .planning()
        .projection()
        .cloned()
        .ok_or(WorthUiAllocationReplanDenial::CandidateProjectionUnavailable)?;
    let admission =
        crate::runtime::planning::allocation_planning::WorthUiAllocationPlanningAdmission::from_projection(
            projection,
            constraint_basis,
            portal,
        );
    let mut candidate = UiAllocationCandidate::from_planning(
        WorthUiAllocationPlanner::plan(admission),
        UiAllocationCandidateMintAuthority::new(),
    );
    if let Some((impact, narrowing)) = selected.replacement_lineage() {
        candidate.seal_replan_successor(impact, narrowing);
    } else {
        candidate.seal_catalog_successor();
    }
    Ok(candidate)
}

pub(crate) fn replan_selected_candidates(
    selection: &crate::graph::UiAdmittedReplanNeighborhoodSet,
) -> Result<Vec<UiAllocationCandidate>, u16> {
    selection
        .ordinal_neighborhoods()
        .map(|(ordinal, selected)| replan_admitted_candidate(selected).map_err(|_| ordinal))
        .collect()
}

pub(crate) fn replan_selected_candidates_with_portal(
    selection: &crate::graph::UiAdmittedReplanNeighborhoodSet,
) -> Result<Vec<UiAllocationCandidate>, u16> {
    let consequences = selection.transaction_basis().consequences();
    selection
        .ordinal_neighborhoods()
        .map(|(ordinal, selected)| {
            let mut matching = consequences.portal_anchors().iter().filter(|consequence| {
                consequence
                    .movement()
                    .target()
                    .primary()
                    .neighborhood_identity()
                    == selected.identity()
            });
            let first = matching.next();
            if matching.next().is_some() {
                return Err(ordinal);
            }
            let portal = first
                .map(|consequence| {
                    crate::runtime::UiPortalAllocationPlanningBasis::seal(
                        consequence.movement(),
                        selected.identity(),
                    )
                    .ok_or(ordinal)
                })
                .transpose()?;
            let mut matching_query =
                consequences
                    .query_measurements()
                    .iter()
                    .filter(|consequence| {
                        consequence.neighborhood_identity_digest()
                            == selected.identity().identity_digest()
                    });
            let query = matching_query.next();
            if query.is_some() && matching_query.next().is_some() {
                return Err(ordinal);
            }
            let mut matching_host = consequences
                .host_measurements()
                .iter()
                .filter(|consequence| {
                    consequence.neighborhood_identity_digest()
                        == selected.identity().identity_digest()
                });
            let host = matching_host.next();
            if host.is_some() && matching_host.next().is_some() {
                return Err(ordinal);
            }
            replan_admitted_candidate_with_sources(selected, portal, query, host)
                .map_err(|_| ordinal)
        })
        .collect()
}

pub(crate) fn replan_selected_candidates_with_resize(
    selection: &crate::graph::UiAdmittedReplanNeighborhoodSet,
    basis: &crate::runtime::UiResizeAllocationPlanningBasis,
) -> Result<Vec<UiAllocationCandidate>, u16> {
    let mut candidates = replan_selected_candidates(selection)?;
    for (candidate, selected) in candidates.iter_mut().zip(selection.ordered_neighborhoods()) {
        candidate.seal_resize_basis(basis.clone());
        if let Some((impact, narrowing)) = selected.replacement_lineage() {
            candidate.seal_replan_successor(impact, narrowing);
        } else {
            candidate.seal_catalog_successor();
        }
    }
    Ok(candidates)
}
