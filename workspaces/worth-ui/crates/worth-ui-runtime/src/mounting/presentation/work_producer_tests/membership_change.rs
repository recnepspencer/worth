use worth_ui_host_contract::{
    UiMountedFrameIdentity, UiMountedInstanceIdentity, UiMountedPaintCommandIdentity,
    UiMountedPresentationWorkView,
};

use super::{rect_spec, MountedPresentationWorld};
use crate::mounting::presentation::work_producer::{
    SuccessorIssueRequest, UiMountedPresentationState,
};

#[test]
fn removal_and_insert_carry_exact_identities_vacated_damage_and_total_order() {
    let world = MountedPresentationWorld::new();
    let predecessor = world.projection(
        UiMountedFrameIdentity::mint_unbound().unwrap(),
        [
            rect_spec(world.first_instance, 0.0),
            rect_spec(world.second_instance, 40.0),
        ],
    );
    let third = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let successor = world.projection(
        UiMountedFrameIdentity::mint_unbound().unwrap(),
        [
            rect_spec(world.second_instance, 40.0),
            rect_spec(third, 80.0),
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
            &[world.first_instance, third],
            &[],
            &lease,
        ))
        .unwrap();
    let UiMountedPresentationWorkView::Delta(delta) = work.view() else {
        panic!("membership change must produce delta work");
    };
    assert_eq!(delta.changes().len(), 2);
    let removed =
        UiMountedPaintCommandIdentity::portal_overlay(&predecessor.portal_overlays().rows()[0]);
    let retained =
        UiMountedPaintCommandIdentity::portal_overlay(&successor.portal_overlays().rows()[0]);
    let inserted =
        UiMountedPaintCommandIdentity::portal_overlay(&successor.portal_overlays().rows()[1]);
    assert!(delta.changes().iter().any(|change| matches!(
        change,
        worth_ui_host_contract::UiMountedPaintCommandChange::Remove(identity) if *identity == removed
    )));
    assert!(delta.changes().iter().any(|change| matches!(
        change,
        worth_ui_host_contract::UiMountedPaintCommandChange::Insert(command) if command.identity() == inserted
    )));
    assert!(delta
        .damage()
        .iter()
        .any(|damage| damage.bounds().x() == 0.0));
    assert!(delta
        .damage()
        .iter()
        .any(|damage| damage.bounds().x() == 80.0));
    let removed_order = worth_ui_host_contract::UiMountedPaintOrderIdentity::for_command(removed);
    let retained_order = worth_ui_host_contract::UiMountedPaintOrderIdentity::for_command(retained);
    let inserted_order = worth_ui_host_contract::UiMountedPaintOrderIdentity::for_command(inserted);
    assert_eq!(
        delta.order(),
        &[
            worth_ui_host_contract::UiMountedPaintOrderEdit::remove(removed_order),
            worth_ui_host_contract::UiMountedPaintOrderEdit::place_after(
                inserted_order,
                Some(retained_order),
            ),
        ]
    );
}
