//! What a frame's work records as displaced once it lands: the displayed
//! command a delta replaces or removes, and never one it only inserts, since
//! no tick shows a sample of a command the displayed frame does not draw.

use worth_ui_host_contract::{
    UiMountedFrameIdentity, UiMountedInstanceIdentity, UiMountedPaintCommandChange,
    UiMountedPaintCommandIdentity, UiMountedPresentationWorkView, UiMountedProjectionView,
};

use super::world::RectSpec;
use super::{rect_spec, MountedPresentationWorld};
use crate::mounting::presentation::work_producer::{
    SuccessorIssueRequest, UiMountedPresentationState,
};

/// The successor state of a frame moving `from` to `to`, with what its work
/// displaces recorded, and the delta's changes.
fn displacing<const P: usize, const S: usize>(
    world: &MountedPresentationWorld,
    from: [RectSpec; P],
    to: [RectSpec; S],
    changed: &[UiMountedInstanceIdentity],
) -> (
    UiMountedProjectionView,
    UiMountedPresentationState,
    Vec<UiMountedPaintCommandChange>,
) {
    let predecessor = world.projection(UiMountedFrameIdentity::mint_unbound().unwrap(), from);
    let successor = world.projection(UiMountedFrameIdentity::mint_unbound().unwrap(), to);
    let predecessor_state =
        UiMountedPresentationState::from_projection(&predecessor, world.requirement, None);
    let mut successor_state = UiMountedPresentationState::from_projection(
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
            changed,
            &[],
            &lease,
        ))
        .unwrap();
    let UiMountedPresentationWorkView::Delta(delta) = work.view() else {
        panic!("a membership or geometry change produces delta work");
    };
    let changes = delta.changes().to_vec();
    successor_state.displace_samples(&work, &[]);
    (predecessor, successor_state, changes)
}

fn command(projection: &UiMountedProjectionView, row: usize) -> UiMountedPaintCommandIdentity {
    UiMountedPaintCommandIdentity::portal_overlay(&projection.portal_overlays().rows()[row])
}

#[test]
fn an_insert_only_delta_displaces_no_sample() {
    let world = MountedPresentationWorld::new();
    let third = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let (predecessor, state, changes) = displacing(
        &world,
        [rect_spec(world.first_instance, 0.0)],
        [rect_spec(world.first_instance, 0.0), rect_spec(third, 80.0)],
        &[third],
    );
    let [UiMountedPaintCommandChange::Insert(inserted)] = changes.as_slice() else {
        panic!("the frame only inserts a command: {changes:?}");
    };
    assert!(!state.displaces_sample_of(inserted.identity()));
    assert!(!state.displaces_sample_of(command(&predecessor, 0)));
    assert!(state.records_no_displacement(), "nothing is allocated");
}

#[test]
fn a_replaced_or_removed_command_displaces_its_sample() {
    let world = MountedPresentationWorld::new();
    let (predecessor, state, changes) = displacing(
        &world,
        [
            rect_spec(world.first_instance, 0.0),
            rect_spec(world.second_instance, 40.0),
        ],
        [rect_spec(world.first_instance, 12.0)],
        &[world.first_instance, world.second_instance],
    );
    let (moved, removed) = (command(&predecessor, 0), command(&predecessor, 1));
    assert!(changes.iter().any(|change| matches!(
        change,
        UiMountedPaintCommandChange::Replace { predecessor, .. } if *predecessor == moved
    )));
    assert!(changes.contains(&UiMountedPaintCommandChange::Remove(removed)));
    assert!(state.displaces_sample_of(moved));
    assert!(state.displaces_sample_of(removed));
}
