use worth_ui_host_contract::{
    UiMountedFrameIdentity, UiMountedPaintCommandIdentity, UiMountedPresentationWorkView,
};

use super::{rect_spec, MountedPresentationWorld};
use crate::mounting::presentation::work_producer::{
    SuccessorIssueRequest, UiMountedPresentationState,
};

#[test]
fn equal_layer_total_order_follows_authored_node_order_not_command_identity() {
    let world = MountedPresentationWorld::new();
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let projection = world.projection(
        frame,
        [
            rect_spec(world.second_instance, 0.0),
            rect_spec(world.first_instance, 0.0),
        ],
    );
    let state = UiMountedPresentationState::from_projection(&projection, world.requirement, None);
    let lease = super::super::UiMountedPresentationLeaseGate::default()
        .claim()
        .unwrap();
    let work = state.issue_initial(&lease, &projection);
    let UiMountedPresentationWorkView::Initial(initial) = work.view() else {
        panic!("first presentation must issue initial work");
    };
    let second =
        UiMountedPaintCommandIdentity::portal_overlay(&projection.portal_overlays().rows()[0]);
    let first =
        UiMountedPaintCommandIdentity::portal_overlay(&projection.portal_overlays().rows()[1]);
    assert!(
        first.mounted_instance().diagnostic_value() < second.mounted_instance().diagnostic_value()
    );
    assert_eq!(
        initial.order(),
        &[
            worth_ui_host_contract::UiMountedPaintOrderIdentity::for_command(second),
            worth_ui_host_contract::UiMountedPaintOrderIdentity::for_command(first),
        ]
    );
}

#[test]
fn equal_layer_successor_reorder_remains_authored_when_identity_order_opposes_it() {
    let world = MountedPresentationWorld::new();
    let predecessor = world.projection(
        UiMountedFrameIdentity::mint_unbound().unwrap(),
        [
            rect_spec(world.first_instance, 0.0),
            rect_spec(world.second_instance, 0.0),
        ],
    );
    let successor = world.projection(
        UiMountedFrameIdentity::mint_unbound().unwrap(),
        [
            rect_spec(world.second_instance, 0.0),
            rect_spec(world.first_instance, 0.0),
        ],
    );
    let predecessor_state =
        UiMountedPresentationState::from_projection(&predecessor, world.requirement, None);
    let successor_state = UiMountedPresentationState::from_projection(
        &successor,
        world.requirement,
        Some(predecessor.frame()),
    );
    let lease = super::super::UiMountedPresentationLeaseGate::default()
        .claim()
        .unwrap();
    let work = predecessor_state
        .issue_successor(SuccessorIssueRequest::new(
            &successor_state,
            &[world.first_instance, world.second_instance],
            &[],
            &lease,
        ))
        .unwrap();
    let UiMountedPresentationWorkView::Delta(delta) = work.view() else {
        panic!("authored equal-layer reorder must issue delta work");
    };
    let expected = successor.retained_paint_order().to_vec();
    let mut observed = predecessor.retained_paint_order().to_vec();
    for edit in delta.order() {
        if let Some(position) = observed.iter().position(|entry| *entry == edit.identity()) {
            observed.remove(position);
        }
        if !edit.is_removal() {
            let position = edit
                .predecessor()
                .and_then(|predecessor| observed.iter().position(|entry| *entry == predecessor))
                .map_or(0, |position| position + 1);
            observed.insert(position, edit.identity());
        }
    }
    assert_eq!(observed, expected);
    assert!(
        expected[0].command().mounted_instance().diagnostic_value()
            > expected[1].command().mounted_instance().diagnostic_value()
    );
}
