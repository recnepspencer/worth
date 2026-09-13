use super::*;
use crate::mounting::presentation::work_producer_tests::world::{
    rect_spec, MountedPresentationWorld,
};

#[test]
fn retained_authored_slots_survive_omission_and_later_bundle_replacement() {
    let world = MountedPresentationWorld::new();
    let first = world.first_instance;
    let later = world.second_instance;
    let foreign = worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
    let hidden = worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
    let mut authored = UiPersistentOrder::default();
    for instance in [first, foreign, hidden, later] {
        authored.append(instance).unwrap();
    }
    let projection = world.projection(
        worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap(),
        [rect_spec(first, 0.0), rect_spec(later, 64.0)],
    );
    let before = UiMountedPresentationState::from_projection_in_authored_order(
        &projection,
        world.requirement,
        None,
        authored.clone(),
    );
    assert!(!before.presented_instances.contains(&foreign));
    assert!(!before.presented_instances.contains(&hidden));
    let later_commands = before
        .commands_by_instance
        .get(&later)
        .unwrap()
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    let expected = [UiMountedPaintOrderIdentity::for_command(
        later_commands[0].identity(),
    )];
    let mut after = before.clone();
    after.frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    after.predecessor = Some(before.frame);
    after.remove_bundle(first);
    after.remove_bundle(later);
    authored.remove(first).unwrap();
    after.instance_order = authored;
    after.insert_bundle(3, later, later_commands.into());
    assert_eq!(after.order(), expected);
    assert!(after.order_integrity.admits(&expected));
    let lease = crate::mounting::presentation::UiMountedPresentationLeaseGate::default()
        .claim()
        .unwrap();
    let work = before
        .issue_successor(super::super::SuccessorIssueRequest::new(
            &after,
            &[first, later],
            &[],
            &lease,
        ))
        .unwrap();
    let worth_ui_host_contract::UiMountedPresentationWorkView::Delta(delta) = work.view() else {
        panic!("removing the earlier presented bundle must issue a delta");
    };
    let mut observed = projection.retained_paint_order().to_vec();
    for edit in delta.order() {
        observed.retain(|identity| *identity != edit.identity());
        if !edit.is_removal() {
            let position = edit.predecessor().map_or(0, |predecessor| {
                observed
                    .iter()
                    .position(|identity| *identity == predecessor)
                    .unwrap()
                    + 1
            });
            observed.insert(position, edit.identity());
        }
    }
    assert_eq!(observed, expected);
    assert!(delta.order_integrity().admits(&observed));
}
