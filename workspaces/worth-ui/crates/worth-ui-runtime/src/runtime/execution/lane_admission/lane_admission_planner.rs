use std::collections::BTreeMap;

use crate::runtime::planning::WorthUiExecutionPlanLoweringFacts;
use crate::runtime::{
    WorthUiExecutionLane, WorthUiExecutionLaneDescriptor, WorthUiExecutionLaneSupport,
    WorthUiLaneAdmission, WorthUiLaneAdmissionCounters, WorthUiLaneAdmissionDenial,
    WorthUiLaneAdmissionDenialReason, WorthUiLaneSupportDiagnostic, WorthUiLaneSupportStatus,
    WorthUiQueryLaneFactLink, WorthUiRuntimeHandleAllocationBasis,
};

pub(crate) struct WorthUiLaneAdmissionPlanner;

struct WorthUiPlanLaneAdmissionEvidence {
    descriptors: BTreeMap<WorthUiExecutionLane, WorthUiExecutionLaneDescriptor>,
    query_fact_links: Vec<WorthUiQueryLaneFactLink>,
    missing_query_owned_support_link: bool,
}

impl WorthUiLaneAdmissionPlanner {
    pub(crate) fn admit_regional_successor(
        authority: &WorthUiExecutionPlanLoweringFacts,
        support: &WorthUiExecutionLaneSupport,
        store: &crate::runtime::planning::plan_topology::WorthUiPlanRegionStore,
    ) -> Result<WorthUiLaneAdmission, WorthUiLaneAdmissionDenial> {
        let mut counters = WorthUiLaneAdmissionCounters::default();
        counters.record_admission();
        counters.record_plan_nodes(store.region_count());
        let mut lane_evidence = WorthUiPlanLaneAdmissionEvidence {
            descriptors: BTreeMap::new(),
            query_fact_links: Vec::new(),
            missing_query_owned_support_link: false,
        };
        for family in regional_families() {
            if store.family_count(family) == 0 {
                continue;
            }
            let lane = super::lane_for_family(family);
            lane_evidence.descriptors.entry(lane).or_insert_with(|| {
                WorthUiExecutionLaneDescriptor::for_lane(
                    lane,
                    family == crate::runtime::WorthUiPlanNodeInputFamily::QueryViewBinding,
                )
            });
        }
        counters.record_distinct_lanes(lane_evidence.descriptors.len());
        verify_supported_plan_lanes(&lane_evidence, support, &mut counters)?;
        Ok(WorthUiLaneAdmission::new(
            support.rows().cloned().collect(),
            Vec::new(),
            WorthUiRuntimeHandleAllocationBasis::from_lowering_authority(authority).digest(),
            counters,
        ))
    }

    pub(crate) fn admit(
        authority: &WorthUiExecutionPlanLoweringFacts,
        support: &WorthUiExecutionLaneSupport,
    ) -> Result<WorthUiLaneAdmission, WorthUiLaneAdmissionDenial> {
        let mut counters = WorthUiLaneAdmissionCounters::default();
        counters.record_admission();
        let node_inputs = authority.node_inputs();

        let lane_evidence = collect_plan_lane_admission_evidence(node_inputs, &mut counters);
        counters.record_distinct_lanes(lane_evidence.descriptors.len());

        verify_supported_plan_lanes(&lane_evidence, support, &mut counters)?;
        verify_query_lane_support_evidence(&lane_evidence, counters)?;

        Ok(WorthUiLaneAdmission::new(
            support.rows().cloned().collect(),
            lane_evidence.query_fact_links,
            WorthUiRuntimeHandleAllocationBasis::from_lowering_authority(authority).digest(),
            counters,
        ))
    }
}

fn regional_families() -> [crate::runtime::WorthUiPlanNodeInputFamily; 13] {
    use crate::runtime::WorthUiPlanNodeInputFamily as Family;
    [
        Family::ComponentInvocation,
        Family::LayoutRegion,
        Family::Command,
        Family::TokenStyle,
        Family::ChildRange,
        Family::StateSlot,
        Family::QueryViewBinding,
        Family::Accessibility,
        Family::DiagnosticsRef,
        Family::LanePartitionRef,
        Family::RenderResourceRef,
        Family::CanvasSpatial,
        Family::RealtimeOverlay,
    ]
}

fn collect_plan_lane_admission_evidence(
    node_inputs: &[crate::runtime::WorthUiPlanNodeInput],
    counters: &mut WorthUiLaneAdmissionCounters,
) -> WorthUiPlanLaneAdmissionEvidence {
    let mut evidence = WorthUiPlanLaneAdmissionEvidence {
        descriptors: BTreeMap::new(),
        query_fact_links: Vec::new(),
        missing_query_owned_support_link: false,
    };

    for (position, node_input) in node_inputs.iter().enumerate() {
        counters.record_plan_node();
        let descriptor = WorthUiExecutionLaneDescriptor::from_node_input(node_input);
        record_query_lane_support_evidence(position, node_input, &mut evidence, counters);
        evidence
            .descriptors
            .entry(descriptor.lane())
            .or_insert(descriptor);
    }

    evidence
}

fn record_query_lane_support_evidence(
    position: usize,
    node_input: &crate::runtime::WorthUiPlanNodeInput,
    evidence: &mut WorthUiPlanLaneAdmissionEvidence,
    counters: &mut WorthUiLaneAdmissionCounters,
) {
    let descriptor = WorthUiExecutionLaneDescriptor::from_node_input(node_input);
    if descriptor.lane() != WorthUiExecutionLane::QueryBound {
        return;
    }

    if let Ok(plan_index) = u32::try_from(position)
        && let Some(links) = WorthUiQueryLaneFactLink::from_plan_node_input(plan_index, node_input)
    {
        counters.record_query_support_link();
        evidence.query_fact_links.push(links);
    }

    if node_input.query_binding_identity().is_none() && node_input.transition().is_none() {
        evidence.missing_query_owned_support_link = true;
    }
}

fn verify_supported_plan_lanes(
    lane_evidence: &WorthUiPlanLaneAdmissionEvidence,
    support: &WorthUiExecutionLaneSupport,
    counters: &mut WorthUiLaneAdmissionCounters,
) -> Result<(), WorthUiLaneAdmissionDenial> {
    for lane in lane_evidence.descriptors.keys().copied() {
        counters.record_support_row_lookup();
        let Some(row) = support.row_for_lane(lane) else {
            counters.record_unsupported_lane_denial();
            return Err(denial(
                WorthUiLaneAdmissionDenialReason::UnsupportedLaneReference,
                Some(lane),
                None,
                *counters,
            ));
        };
        if row.status() != WorthUiLaneSupportStatus::Supported {
            counters.record_unsupported_lane_denial();
            return Err(denial(
                WorthUiLaneAdmissionDenialReason::UnsupportedLaneReference,
                Some(lane),
                Some(WorthUiLaneSupportDiagnostic::unsupported(row)),
                *counters,
            ));
        }
    }
    Ok(())
}

fn verify_query_lane_support_evidence(
    lane_evidence: &WorthUiPlanLaneAdmissionEvidence,
    counters: WorthUiLaneAdmissionCounters,
) -> Result<(), WorthUiLaneAdmissionDenial> {
    if !lane_evidence
        .descriptors
        .contains_key(&WorthUiExecutionLane::QueryBound)
    {
        return Ok(());
    }

    if lane_evidence.missing_query_owned_support_link || lane_evidence.query_fact_links.is_empty() {
        return Err(denial(
            WorthUiLaneAdmissionDenialReason::MissingQuerySupportLinks,
            Some(WorthUiExecutionLane::QueryBound),
            None,
            counters,
        ));
    }

    Ok(())
}

fn denial(
    reason: WorthUiLaneAdmissionDenialReason,
    lane: Option<WorthUiExecutionLane>,
    diagnostic: Option<WorthUiLaneSupportDiagnostic>,
    counters: WorthUiLaneAdmissionCounters,
) -> WorthUiLaneAdmissionDenial {
    WorthUiLaneAdmissionDenial::new(reason, lane, diagnostic, counters)
}
