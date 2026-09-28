use super::plan_topology_test_support::*;
use crate::runtime::{
    WorthUiChildRangeHandle, WorthUiExecutionPlanInput, WorthUiPlanNodeInputFamily,
    WorthUiPlanTopologyDenialReason,
};

#[test]
fn equivalent_plan_inputs_preserve_semantics_without_sharing_handle_authority() {
    let (left_runtime, left_input, left_planning, left_handles) = topology_fixture();
    let (_, right_input, right_planning, right_handles) = topology_fixture();
    let left = assemble(&left_planning, &left_input, &left_handles);
    let right = assemble(&right_planning, &right_input, &right_handles);

    assert_eq!(
        left.handle_receipt().basis_digest(),
        right.handle_receipt().basis_digest()
    );
    assert_ne!(
        left.handle_receipt().arena_identity(),
        right.handle_receipt().arena_identity()
    );
    assert_eq!(
        left_runtime.digest_execution_plan(&left),
        left_runtime.digest_execution_plan(&right)
    );
    for (left_node, right_node) in left
        .topology()
        .traversal_order()
        .iter()
        .zip(right.topology().traversal_order())
    {
        assert_eq!(left_node.family(), right_node.family());
        assert_eq!(
            left_node.runtime_handle().plan_index(),
            right_node.runtime_handle().plan_index()
        );
        assert_ne!(left_node.runtime_handle(), right_node.runtime_handle());
    }
    assert_eq!(left.lane_partitions(), right.lane_partitions());
    assert_eq!(left.lookup_index(), right.lookup_index());
    assert_eq!(left.counters(), right.counters());
}

#[test]
fn initial_construction_receipt_includes_every_full_plan_pass() {
    let (_, plan_input, planning, allocation) = topology_fixture();
    let plan = assemble(&planning, &plan_input, &allocation);
    let construction = plan.construction_counters();

    assert_eq!(
        construction.handle_allocation(),
        allocation.counters(),
        "handle construction must not disappear from the plan cost receipt"
    );
    assert_eq!(construction.topology(), plan.counters());
    assert_eq!(
        construction.regional_storage(),
        plan.region_storage_counters()
    );
    assert_eq!(
        construction.full_candidate_node_visit_count(),
        plan_input.node_inputs().len() * 4,
        "initial lowering, admission, handle materialization, and flat reconstruction remain explicit"
    );
}

#[test]
fn plan_topology_assembly_rejects_missing_child_or_lane_links() {
    let (_, plan_input, planning, allocation) = topology_fixture();
    let mut runtime_handles = allocation.runtime_handles().to_vec();
    runtime_handles.pop();
    let broken = allocation_with_runtime_handles(&allocation, runtime_handles);

    let denial = assemble_err(&planning, &plan_input, &broken);

    assert_eq!(
        denial.reason(),
        WorthUiPlanTopologyDenialReason::MissingRuntimeHandle
    );
    assert_eq!(denial.counters().denial_count(), 1);
    assert!(denial.counters().topology_node_count() == 0);
}

#[test]
fn frame_traversal_uses_plan_topology_without_artifact_tree_scan() {
    let (_, plan_input, planning, allocation) = topology_fixture();
    let plan = assemble(&planning, &plan_input, &allocation);
    let counters = plan.counters();

    assert_eq!(
        plan.topology().traversal_order().len(),
        plan_input.node_inputs().len()
    );
    assert!(plan
        .lane_partitions()
        .iter()
        .all(|lane| lane.node_count() > 0));
    assert_eq!(counters.artifact_tree_scan_count(), 0);
    assert_eq!(counters.registry_string_lookup_count(), 0);
    assert_eq!(counters.broad_registry_scan_count(), 0);
    assert_eq!(counters.topology_validation_count(), 4);

    let region_nodes = plan
        .topology()
        .traversal_order()
        .iter()
        .filter_map(|node| node.region_structure())
        .collect::<Vec<_>>();
    assert!(!region_nodes.is_empty());
    assert!(region_nodes
        .iter()
        .all(|structure| structure.structure_declared()));
    assert_eq!(
        plan.topology().child_ranges().len(),
        counters.child_range_count()
    );
    assert!(plan
        .topology()
        .child_ranges()
        .iter()
        .all(|range| !range.is_empty()));
    for (node_input, topology_node) in plan_input
        .node_inputs()
        .iter()
        .zip(plan.topology().traversal_order())
    {
        if let Some(region_structure) = topology_node.region_structure() {
            assert_eq!(
                region_structure.root_region_count(),
                node_input.topology_input().root_region_count()
            );
            assert_eq!(
                region_structure.region_count(),
                node_input.topology_input().region_count()
            );
            assert_eq!(
                region_structure.mount_count(),
                node_input.topology_input().mount_count()
            );
            assert_eq!(
                region_structure.max_region_depth(),
                node_input.topology_input().max_region_depth()
            );
            assert_eq!(
                topology_node.child_range().map(|range| range.len()),
                (region_structure.root_region_count() > 0)
                    .then_some(u32::try_from(region_structure.root_region_count()).unwrap())
            );
        }
    }
}

#[test]
fn plan_topology_rejects_missing_child_range_handles() {
    let (_, plan_input, _, _) = topology_fixture();
    let plan_input = plan_input_with_first_child_range_family(plan_input);
    let allocation_planning = topology_planning("plan-topology.missing-child-range");
    let allocation = allocate_handles(&allocation_planning, &plan_input);
    assert!(allocation.child_range_handles().next().is_some());
    let broken = allocation_with_child_ranges(&allocation, Vec::new());

    let denial = assemble_err(&allocation_planning, &plan_input, &broken);

    assert_eq!(
        denial.reason(),
        WorthUiPlanTopologyDenialReason::MissingRuntimeHandle
    );
    assert_eq!(denial.counters().denial_count(), 1);
}

#[test]
fn plan_topology_rejects_orphaned_child_range_handles() {
    let (_, plan_input, _, _) = topology_fixture();
    let plan_input = plan_input_with_first_child_range_family(plan_input);
    let allocation_planning = topology_planning("plan-topology.orphaned-child-range");
    let allocation = allocate_handles(&allocation_planning, &plan_input);
    let mut child_ranges = allocation.child_range_handles().collect::<Vec<_>>();
    let exemplar = child_ranges[0];
    child_ranges[0] = WorthUiChildRangeHandle::new(
        u32::MAX,
        exemplar.slot_generation(),
        exemplar.arena_identity(),
    );
    let broken = allocation_with_child_ranges(&allocation, child_ranges);

    let denial = assemble_err(&allocation_planning, &plan_input, &broken);

    assert_eq!(
        denial.reason(),
        WorthUiPlanTopologyDenialReason::RuntimeHandleOutOfBounds
    );
    assert_eq!(denial.counters().denial_count(), 1);
}

fn plan_input_with_first_child_range_family(
    plan_input: WorthUiExecutionPlanInput,
) -> WorthUiExecutionPlanInput {
    let mut node_inputs = plan_input.node_inputs().to_vec();
    let index = node_inputs
        .iter()
        .position(|input| input.family() != WorthUiPlanNodeInputFamily::ChildRange)
        .expect("fixture includes a non-child-range plan input");
    node_inputs[index] = node_inputs[index]
        .clone()
        .with_family_for_test(WorthUiPlanNodeInputFamily::ChildRange);
    WorthUiExecutionPlanInput::new(
        plan_input.basis().clone(),
        plan_input.context().clone(),
        node_inputs,
        plan_input.counters(),
    )
}

#[test]
fn plan_topology_rejects_missing_declared_region_structure() {
    let (_, plan_input, _, _) = topology_fixture();
    let broken_input = plan_input_without_first_region_structure(plan_input);
    let broken_planning = topology_planning("plan-topology.missing-region-structure");
    let broken_allocation = allocate_handles(&broken_planning, &broken_input);

    let denial = assemble_err(&broken_planning, &broken_input, &broken_allocation);

    assert_eq!(
        denial.reason(),
        WorthUiPlanTopologyDenialReason::MissingRegionStructure
    );
    assert_eq!(denial.counters().denial_count(), 1);
}
