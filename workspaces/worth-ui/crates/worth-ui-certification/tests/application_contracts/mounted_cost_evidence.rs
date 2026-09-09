use worth_ui_runtime::facade::mounted::{
    UiHostSurfacePresentationMode, UiMountWorkClass, UiMountedFrameOutcome, UiPresentationDeadline,
};
use worth_ui_test_support::WorthUiMountedIdentityCertificationExt;
use worth_ui_test_support::WorthUiMountedPublicationCertificationExt;

use super::mounted_application_lifecycle::in_flight_presentation_world::{
    mounted_session, prepared,
};
use super::mounted_application_lifecycle::known_empty_surface_world::profile;
use super::mounted_host_protocol::scripted_host::ScriptedPresentationHost;

#[test]
fn one_instance_delta_names_its_order_batch_without_rescanning_semantic_truth() {
    const INITIAL_INSTANCES: usize = 512;
    let host = ScriptedPresentationHost::default();
    let (mut session, _) = mounted_session(host.clone(), "mounted-delta-cost", 1);
    let identity = session.inspect_mounted_identity();
    let first = identity.mounted_instances()[0].clone();
    let surface = first.basis().semantic_surface_identity();
    let node = session
        .mounted_graph_node(first.graph_node_identity())
        .unwrap();
    for _ in 1..INITIAL_INSTANCES {
        session.mount_instance(node, surface).unwrap();
    }

    let initial = prepared(&mut session);
    let initial_cost = initial.cost_report();
    assert_eq!(initial_cost.work_class(), UiMountWorkClass::InitialMount);
    assert_eq!(
        initial_cost.initial_mounted_instances(),
        INITIAL_INSTANCES as u64
    );
    host.push_presented();
    assert!(matches!(
        session.present_prepared_mounted_frame(initial, UiPresentationDeadline::at_tick(10), 0),
        UiMountedFrameOutcome::Published(_)
    ));

    let comparison = prepared(&mut session);
    let comparison_cost = comparison.cost_report();
    assert_eq!(
        comparison_cost.work_class(),
        UiMountWorkClass::UnchangedReuse
    );
    assert_eq!(comparison_cost.changed_mounted_instances(), 0);
    assert_eq!(comparison_cost.named().considered(), 0);
    drop(comparison);

    session.mount_instance(node, surface).unwrap();
    let delta = prepared(&mut session);
    let delta_cost = delta.cost_report();
    assert_eq!(delta_cost.work_class(), UiMountWorkClass::SemanticDelta);
    assert_eq!(delta_cost.initial_mounted_instances(), 0);
    assert_eq!(delta_cost.changed_mounted_instances(), 1);
    assert_eq!(delta_cost.surface_instance_pairs(), 1);
    assert_eq!(delta_cost.changed_binding_generations(), 0);
    assert_eq!(
        delta_cost.named().considered(),
        2,
        "the new occurrence and its count-only paint owner are both considered"
    );
    assert_logarithmic_index_work(delta_cost.index_entries_touched(), INITIAL_INSTANCES);
    assert_eq!(
        delta_cost.replaced_batch_rows(),
        2,
        "one changed instance replaces only its layer and count-only paint rows"
    );
    assert_eq!(
        delta_cost.replaced_batch_bytes(),
        specialized_replacement_bytes(),
        "the replacement granule carries no retained semantic-order batch"
    );
}

#[test]
fn overlapping_semantic_and_binding_delta_counts_distinct_surface_pairs_once() {
    const WARM_INSTANCES: usize = 64;
    let host = ScriptedPresentationHost::default();
    let (mut session, bindings) = mounted_session(host.clone(), "mounted-mixed-delta-cost", 1);
    let binding = bindings[0];
    let identity = session.inspect_mounted_identity();
    let first = identity.mounted_instances()[0].clone();
    let surface = first.basis().semantic_surface_identity();
    let node = session
        .mounted_graph_node(first.graph_node_identity())
        .unwrap();
    for _ in 1..WARM_INSTANCES {
        session.mount_instance(node, surface).unwrap();
    }
    host.push_presented();
    let initial = prepared(&mut session);
    assert!(matches!(
        session.present_prepared_mounted_frame(initial, UiPresentationDeadline::at_tick(10), 0),
        UiMountedFrameOutcome::Published(_)
    ));

    host.push_presentation(
        worth_ui_runtime::facade::mounted::UiHostSurfacePresentationOutcome::PresentationIndeterminate,
    );
    let uncertain = prepared(&mut session);
    assert!(matches!(
        session.present_prepared_mounted_frame(uncertain, UiPresentationDeadline::at_tick(20), 1),
        UiMountedFrameOutcome::PresentationIndeterminate(_)
    ));
    session.mount_instance(node, surface).unwrap();
    session
        .rebind_host_surface(
            binding,
            UiHostSurfacePresentationMode::RecordOnly,
            profile(2),
        )
        .unwrap();
    let delta = prepared(&mut session);
    let cost = delta.cost_report();
    assert_eq!(cost.work_class(), UiMountWorkClass::SemanticDelta);
    assert_eq!(cost.changed_mounted_instances(), 1);
    assert_eq!(cost.changed_binding_generations(), 1);
    assert_eq!(
        cost.surface_instance_pairs(),
        (WARM_INSTANCES + 1) as u64,
        "u is the distinct union, not changed-surface pairs plus an overlapping changed instance"
    );
    assert_eq!(
        cost.named().considered(),
        3,
        "the new occurrence, count-only paint owner, and changed binding are considered"
    );
    assert_eq!(
        cost.replaced_batch_rows(),
        2,
        "binding overlap does not rematerialize retained semantic-order rows"
    );
    assert_eq!(cost.replaced_batch_bytes(), specialized_replacement_bytes());
    assert_eq!(
        delta.receipt().delta().surface_instance_pairs(),
        cost.surface_instance_pairs()
    );
}

fn assert_logarithmic_index_work(index_entries: u64, graph_instances: usize) {
    let binary_search_levels =
        usize::BITS as usize - graph_instances.max(1).leading_zeros() as usize;
    let independent_avl_ceiling = 2 + 16 * binary_search_levels;
    assert!(
        (2..=u64::try_from(independent_avl_ceiling).unwrap()).contains(&index_entries),
        "one changed instance may touch the retained semantic, order, and appearance AVL \
         paths, not a linear fraction of {graph_instances} instances; observed \
         {index_entries}, ceiling {independent_avl_ceiling}"
    );
}

fn specialized_replacement_bytes() -> u64 {
    let bytes = std::mem::size_of::<worth_ui_runtime::facade::mounted::UiMountedLayerRow>()
        .checked_add(std::mem::size_of::<
            worth_ui_runtime::facade::mounted::UiMountedPaintBatchRow,
        >())
        .expect("the two-row replacement granule fits usize");
    u64::try_from(bytes).expect("the replacement granule fits u64")
}
