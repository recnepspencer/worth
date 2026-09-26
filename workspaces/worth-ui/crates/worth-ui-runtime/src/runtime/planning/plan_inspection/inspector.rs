use super::{WorthUiPlanInspectionDenialReason, WorthUiPlanProvenanceSource};
use crate::runtime::planning::plan_equivalence::WorthUiExecutionPlanDigestor;
use crate::runtime::planning::WorthUiExecutionPlanLoweringFacts;
use crate::runtime::{
    WorthUiArtifactToPlanProvenance, WorthUiExecutionPlan, WorthUiExecutionPlanInspection,
    WorthUiLaneInspection, WorthUiPlanInspectionCounters, WorthUiPlanInspectionDenial,
    WorthUiPlanNode, WorthUiPlanNodeInput, WorthUiPlanNodeInputFamily, WorthUiPlanNodeInspection,
    WorthUiQueryInspectionLinks, WorthUiRuntimeHandleAllocationBasis,
};
use std::collections::BTreeMap;

pub(crate) struct WorthUiExecutionPlanInspector;

impl WorthUiExecutionPlanInspector {
    pub(crate) fn inspect(
        plan: &WorthUiExecutionPlan,
        authority: &WorthUiExecutionPlanLoweringFacts,
    ) -> Result<WorthUiExecutionPlanInspection, WorthUiPlanInspectionDenial> {
        let lowering_basis = authority.plan_input().basis();
        let mut counters = WorthUiPlanInspectionCounters::default();
        counters.record_inspection();
        validate_plan_input_alignment(plan, authority, counters)?;
        let rows = reconstructive_rows(plan, authority);

        let mut provenance = Vec::with_capacity(rows.len());
        let mut nodes = Vec::with_capacity(rows.len());
        for (node, node_input, _) in &rows {
            let node_provenance = provenance_for_node(node, node_input, &mut counters);
            counters.record_provenance_link();
            counters.record_node_inspection();
            nodes.push(WorthUiPlanNodeInspection::new(
                super::WorthUiPlanNodeInspectionInput {
                    plan_index: node.runtime_handle().plan_index(),
                    runtime_handle: node.runtime_handle(),
                    family: node.family(),
                    child_range: node.child_range(),
                    region_structure: node.region_structure(),
                    render_resource_ref: node.render_resource_ref(),
                    provenance: node_provenance.clone(),
                },
            ));
            provenance.push(node_provenance);
        }

        let lanes = reconstructive_lanes(plan, &rows, &mut counters);
        counters.record_plan_digest();
        let active_artifact_digest = lowering_basis
            .prior_artifact_digest()
            .unwrap_or_else(|| lowering_basis.candidate_artifact_digest());
        let handle_basis_digest = plan.handle_receipt().basis_digest();
        let plan_digest = WorthUiExecutionPlanDigestor::digest(plan).0;

        Ok(WorthUiExecutionPlanInspection::new(
            super::WorthUiExecutionPlanInspectionInput {
                active_artifact_digest,
                handle_basis_digest,
                handle_arena_identity: plan.handle_receipt().arena_identity(),
                lowering_identity: authority.identity().clone(),
                plan_digest,
                nodes,
                lanes,
                provenance,
                counters,
            },
        ))
    }
}

fn validate_plan_input_alignment(
    plan: &WorthUiExecutionPlan,
    authority: &WorthUiExecutionPlanLoweringFacts,
    counters: WorthUiPlanInspectionCounters,
) -> Result<(), WorthUiPlanInspectionDenial> {
    if !plan.shares_lowering_authority_with(authority) {
        return Err(denial(
            WorthUiPlanInspectionDenialReason::ForeignLoweringAuthority,
            counters,
        ));
    }
    let node_inputs = authority.node_inputs();
    let allocation_basis = WorthUiRuntimeHandleAllocationBasis::from_lowering_authority(authority);
    if !plan.handle_receipt().certifies_basis(&allocation_basis) {
        return Err(denial(
            WorthUiPlanInspectionDenialReason::PlanInputReceiptMismatch,
            counters,
        ));
    }
    if !plan.has_reconstructive_flat_projection() {
        if plan.region_count() != authority.plan_input().basis().candidate_node_input_count() {
            return Err(denial(
                WorthUiPlanInspectionDenialReason::PlanInputNodeCountMismatch,
                counters,
            ));
        }
        return Ok(());
    }
    if plan.topology().traversal_order().len() != node_inputs.len() {
        return Err(denial(
            WorthUiPlanInspectionDenialReason::PlanInputNodeCountMismatch,
            counters,
        ));
    }
    for (position, (node, node_input)) in plan
        .topology()
        .traversal_order()
        .iter()
        .zip(node_inputs)
        .enumerate()
    {
        let plan_index = u32::try_from(position).map_err(|_| {
            denial(
                WorthUiPlanInspectionDenialReason::RuntimeHandlePlanIndexMismatch,
                counters,
            )
        })?;
        if node.runtime_handle().plan_index() != plan_index {
            return Err(denial(
                WorthUiPlanInspectionDenialReason::RuntimeHandlePlanIndexMismatch,
                counters,
            ));
        }
        if node.family().input_family() != node_input.family() {
            return Err(denial(
                WorthUiPlanInspectionDenialReason::PlanNodeFamilyMismatch,
                counters,
            ));
        }
    }
    Ok(())
}

type ReconstructiveInspectionRow = (
    WorthUiPlanNode,
    WorthUiPlanNodeInput,
    Option<crate::runtime::WorthUiPlanExecutionLane>,
);

fn reconstructive_rows(
    plan: &WorthUiExecutionPlan,
    authority: &WorthUiExecutionPlanLoweringFacts,
) -> Vec<ReconstructiveInspectionRow> {
    if plan.has_reconstructive_flat_projection() {
        return plan
            .topology()
            .traversal_order()
            .iter()
            .cloned()
            .zip(authority.node_inputs().iter().cloned())
            .map(|(node, input)| (node, input, None))
            .collect();
    }
    plan.reconstructive_inspection_rows()
        .into_iter()
        .map(|(node, input, lane)| (node, input, Some(lane)))
        .collect()
}

fn reconstructive_lanes(
    plan: &WorthUiExecutionPlan,
    rows: &[ReconstructiveInspectionRow],
    counters: &mut WorthUiPlanInspectionCounters,
) -> Vec<WorthUiLaneInspection> {
    if plan.has_reconstructive_flat_projection() {
        return plan
            .lane_partitions()
            .iter()
            .map(|lane| {
                counters.record_lane_inspection();
                WorthUiLaneInspection::new(
                    lane.lane(),
                    lane.plan_indexes().to_vec(),
                    lane.node_count(),
                )
            })
            .collect();
    }
    let mut by_lane = BTreeMap::new();
    for (node, _, lane) in rows {
        by_lane
            .entry(lane.expect("regional inspection rows carry their lane"))
            .or_insert_with(Vec::new)
            .push(node.runtime_handle().plan_index());
    }
    by_lane
        .into_iter()
        .map(|(lane, plan_indexes)| {
            counters.record_lane_inspection();
            let node_count = plan_indexes.len();
            WorthUiLaneInspection::new(lane, plan_indexes, node_count)
        })
        .collect()
}

fn provenance_for_node(
    node: &WorthUiPlanNode,
    node_input: &WorthUiPlanNodeInput,
    counters: &mut WorthUiPlanInspectionCounters,
) -> WorthUiArtifactToPlanProvenance {
    let query_links = query_links_for_node_input(node_input, counters);
    WorthUiArtifactToPlanProvenance::new(
        node.runtime_handle().plan_index(),
        node_input.identity_basis().to_owned(),
        node_input.authored_provenance_digest(),
        node_input.family(),
        provenance_source_for_node_input(node_input),
        capability_reference_for_node_input(node_input),
        query_links,
    )
}

fn query_links_for_node_input(
    node_input: &WorthUiPlanNodeInput,
    counters: &mut WorthUiPlanInspectionCounters,
) -> Option<WorthUiQueryInspectionLinks> {
    let identity = node_input.query_binding_identity()?.clone();
    let settled_fact_link = node_input.query_settled_fact_link()?.clone();
    counters.record_query_link_preservation();
    counters.record_projection_consumption_link();
    Some(WorthUiQueryInspectionLinks::from_settled_fact_link(
        identity,
        settled_fact_link,
        node_input.query_preservation_receipt(),
    ))
}

fn capability_reference_for_node_input(node_input: &WorthUiPlanNodeInput) -> Option<String> {
    match node_input.family() {
        WorthUiPlanNodeInputFamily::QueryViewBinding => node_input
            .query_binding_identity()
            .map(|identity| identity.view_binding_id().to_owned())
            .or_else(|| Some(node_input.identity_basis().to_owned())),
        WorthUiPlanNodeInputFamily::ComponentInvocation
        | WorthUiPlanNodeInputFamily::Command
        | WorthUiPlanNodeInputFamily::TokenStyle
        | WorthUiPlanNodeInputFamily::LayoutRegion
        | WorthUiPlanNodeInputFamily::RenderResourceRef => {
            Some(node_input.identity_basis().to_owned())
        }
        WorthUiPlanNodeInputFamily::ChildRange
        | WorthUiPlanNodeInputFamily::StateSlot
        | WorthUiPlanNodeInputFamily::Accessibility
        | WorthUiPlanNodeInputFamily::DiagnosticsRef
        | WorthUiPlanNodeInputFamily::LanePartitionRef
        | WorthUiPlanNodeInputFamily::CanvasSpatial
        | WorthUiPlanNodeInputFamily::RealtimeOverlay => None,
    }
}

fn provenance_source_for_node_input(
    node_input: &WorthUiPlanNodeInput,
) -> WorthUiPlanProvenanceSource {
    if node_input.query_binding_identity().is_some()
        && node_input.query_settled_fact_link().is_some()
    {
        return WorthUiPlanProvenanceSource::QueryBinding;
    }
    provenance_source_for_family(node_input.family())
}

fn provenance_source_for_family(family: WorthUiPlanNodeInputFamily) -> WorthUiPlanProvenanceSource {
    match family {
        WorthUiPlanNodeInputFamily::ComponentInvocation => {
            WorthUiPlanProvenanceSource::ComponentLoweringHook
        }
        WorthUiPlanNodeInputFamily::LanePartitionRef => WorthUiPlanProvenanceSource::LaneBoundary,
        WorthUiPlanNodeInputFamily::DiagnosticsRef => WorthUiPlanProvenanceSource::Diagnostics,
        WorthUiPlanNodeInputFamily::RenderResourceRef => {
            WorthUiPlanProvenanceSource::RenderResource
        }
        WorthUiPlanNodeInputFamily::ChildRange
        | WorthUiPlanNodeInputFamily::Command
        | WorthUiPlanNodeInputFamily::TokenStyle
        | WorthUiPlanNodeInputFamily::StateSlot
        | WorthUiPlanNodeInputFamily::LayoutRegion
        | WorthUiPlanNodeInputFamily::QueryViewBinding
        | WorthUiPlanNodeInputFamily::Accessibility
        | WorthUiPlanNodeInputFamily::CanvasSpatial
        | WorthUiPlanNodeInputFamily::RealtimeOverlay => {
            WorthUiPlanProvenanceSource::ReplacementClassification
        }
    }
}

fn denial(
    reason: WorthUiPlanInspectionDenialReason,
    counters: WorthUiPlanInspectionCounters,
) -> WorthUiPlanInspectionDenial {
    WorthUiPlanInspectionDenial::new(reason, counters)
}
