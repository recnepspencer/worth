use std::collections::BTreeSet;

use crate::runtime::{
    WorthUiPlanNodeInput, WorthUiPlanNodeInputFamily, WorthUiRuntimeHandleAllocationCounters,
    WorthUiRuntimeHandleAllocationDenial, WorthUiRuntimeHandleAllocationDenialReason,
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct SpecializedHandleClaim {
    family: WorthUiPlanNodeInputFamily,
    identity_basis: String,
}

pub(super) fn reject_invalid_specialized_handle_claims(
    node_inputs: &[WorthUiPlanNodeInput],
) -> Result<WorthUiRuntimeHandleAllocationCounters, WorthUiRuntimeHandleAllocationDenial> {
    let mut counters = WorthUiRuntimeHandleAllocationCounters::default();
    let mut specialized_claims = BTreeSet::new();
    for node_input in node_inputs {
        reject_missing_query_binding_evidence(node_input, counters)?;

        let Some(claim) = specialized_handle_claim_for(node_input) else {
            continue;
        };
        counters.record_collision_check();
        if specialized_claims.insert(claim) {
            continue;
        }
        counters.record_collision_denial();
        return Err(denial(
            WorthUiRuntimeHandleAllocationDenialReason::DuplicatePlanLocalHandleClaim,
            counters,
        ));
    }
    Ok(counters)
}

fn reject_missing_query_binding_evidence(
    node_input: &WorthUiPlanNodeInput,
    counters: WorthUiRuntimeHandleAllocationCounters,
) -> Result<(), WorthUiRuntimeHandleAllocationDenial> {
    if node_input.query_binding_identity().is_none()
        || node_input.query_settled_fact_link().is_some()
    {
        return Ok(());
    }
    Err(denial(
        WorthUiRuntimeHandleAllocationDenialReason::MissingQueryBindingEvidence,
        counters,
    ))
}

fn specialized_handle_claim_for(
    node_input: &WorthUiPlanNodeInput,
) -> Option<SpecializedHandleClaim> {
    if !claims_specialized_handle(node_input) {
        return None;
    }
    Some(SpecializedHandleClaim {
        family: node_input.family(),
        identity_basis: node_input.identity_basis().to_owned(),
    })
}

fn claims_specialized_handle(node_input: &WorthUiPlanNodeInput) -> bool {
    match node_input.family() {
        WorthUiPlanNodeInputFamily::ComponentInvocation
        | WorthUiPlanNodeInputFamily::Command
        | WorthUiPlanNodeInputFamily::TokenStyle
        | WorthUiPlanNodeInputFamily::ChildRange
        | WorthUiPlanNodeInputFamily::LanePartitionRef => true,
        WorthUiPlanNodeInputFamily::QueryViewBinding => {
            node_input.query_binding_identity().is_some()
        }
        WorthUiPlanNodeInputFamily::StateSlot
        | WorthUiPlanNodeInputFamily::LayoutRegion
        | WorthUiPlanNodeInputFamily::Accessibility
        | WorthUiPlanNodeInputFamily::DiagnosticsRef
        | WorthUiPlanNodeInputFamily::RenderResourceRef
        | WorthUiPlanNodeInputFamily::CanvasSpatial
        | WorthUiPlanNodeInputFamily::RealtimeOverlay => node_input.transition().is_some(),
    }
}

fn denial(
    reason: WorthUiRuntimeHandleAllocationDenialReason,
    counters: WorthUiRuntimeHandleAllocationCounters,
) -> WorthUiRuntimeHandleAllocationDenial {
    WorthUiRuntimeHandleAllocationDenial::new(reason, counters)
}
