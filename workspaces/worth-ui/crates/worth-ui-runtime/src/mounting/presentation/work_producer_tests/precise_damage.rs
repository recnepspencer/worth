use worth_ui_host_contract::{
    UiMountedFrameIdentity, UiMountedPaintCommandChange, UiMountedPresentationWorkView,
};

use super::world::{rect_spec, MountedPresentationWorld};
use crate::mounting::presentation::work_producer::{
    SuccessorIssueRequest, UiMountedPresentationState,
};

#[test]
fn precise_replacement_carries_vacated_and_successor_bounds() {
    let world = MountedPresentationWorld::new();
    let predecessor = world.projection(
        UiMountedFrameIdentity::mint_unbound().unwrap(),
        [rect_spec(world.first_instance, 0.0)],
    );
    let successor = world.projection(
        UiMountedFrameIdentity::mint_unbound().unwrap(),
        [rect_spec(world.first_instance, 96.0)],
    );
    let predecessor_state =
        UiMountedPresentationState::from_projection(&predecessor, world.requirement, None);
    let successor_state = UiMountedPresentationState::from_projection(
        &successor,
        world.requirement,
        Some(predecessor.frame()),
    );
    let replacement = successor.retained_paint_commands()[0].clone();
    let lease = super::super::UiMountedPresentationLeaseGate::default()
        .claim()
        .unwrap();

    let work = predecessor_state
        .issue_successor(SuccessorIssueRequest::new(
            &successor_state,
            &[world.first_instance],
            &[UiMountedPaintCommandChange::replacement(
                predecessor.retained_paint_commands()[0].identity(),
                replacement,
            )],
            &lease,
        ))
        .unwrap();
    let UiMountedPresentationWorkView::Delta(delta) = work.view() else {
        panic!("precise replacement must issue delta work");
    };
    let mut xs = delta
        .damage()
        .iter()
        .map(|damage| crate::whole_number::whole_u32(f64::from(damage.bounds().x())).unwrap())
        .collect::<Vec<_>>();
    xs.sort_unstable();
    assert_eq!(xs, [0, 96], "old and new pixels both require replay");
}
