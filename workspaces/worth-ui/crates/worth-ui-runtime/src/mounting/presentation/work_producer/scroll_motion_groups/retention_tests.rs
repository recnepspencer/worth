use super::*;
use crate::mounting::presentation::work_producer_tests::world::{
    rect_spec, MountedPresentationWorld,
};

#[test]
fn independently_projected_command_clips_are_not_mistaken_for_shared_input_clips() {
    // This fixture isolates storage ownership, not semantic Scroll geometry.
    // Projection allocates its own clip list even when values equal the input.
    let world = MountedPresentationWorld::new();
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let projection = world.projection(frame, [rect_spec(world.first_instance, 0.0)]);
    let mut state =
        UiMountedPresentationState::from_projection(&projection, world.requirement, None);
    let command = &projection.authored_paint_commands()[0];
    let bounds = command.clip_bounds();
    let target = UiMotionTargetIdentity::from_scroll_region_owner(
        world.requirement.semantic_surface(),
        world.first_instance,
        17,
    );
    let clip = UiMountedScrollMotionClip {
        bounds,
        owner: None,
    };
    let group = UiMountedScrollMotionGroup {
        input: UiMountedScrollMotionGroupInput {
            target,
            owner: world.first_instance,
            content: bounds,
            viewport: bounds,
            offset: UiScrollOffset::default(),
            scale: UiScrollPresentationDeviceScale::admit(1000).unwrap(),
            chrome: None,
            members: Arc::from([UiMountedScrollMotionMember {
                instance: world.first_instance,
                clips: Arc::from([clip.clone()]),
            }]),
        },
        commands: Arc::from([UiMountedScrollMotionCommand {
            identity: command.identity(),
            clips: Arc::from([clip.clone()]),
            base_translation: None,
        }]),
        thumbs: Arc::from([]),
        bound_standing: super::group_offset::UiBoundGroupStanding::new(
            super::group_offset::UiGroupStanding::Published(
                super::group_offset::UiPublishedGroupOffset::of(
                    crate::runtime::scroll::UiScrollOffset::origin(),
                ),
            ),
            super::group_offset::UiScrollGroupBind::default(),
        ),
        displayed_sample: Default::default(),
    };
    assert!(!Arc::ptr_eq(
        &group.input.members[0].clips,
        &group.commands[0].clips
    ));
    state.scroll_motion_groups.groups = std::rc::Rc::new(BTreeMap::from([(target, group.clone())]));
    state.scroll_motion_groups.owners = Arc::new(BTreeMap::from([(world.first_instance, target)]));
    state.scroll_motion_groups.memberships = Arc::new(std::collections::HashMap::from([(
        command.identity(),
        Arc::from([target]),
    )]));
    let initial = state.indexed_motion_reserved_bytes().unwrap();
    state.scroll_motion_groups.geometry_index_reserved_bytes = 101;
    assert_eq!(
        state.indexed_motion_reserved_bytes(),
        Some(initial + 101),
        "the shared layout index is charged once per surface, not per overlapping group"
    );
    state.scroll_motion_groups.geometry_index_reserved_bytes = 0;
    let mut enlarged = group.clone();
    let mut projected = enlarged.commands[0].clone();
    projected.clips = Arc::from([clip.clone(), clip.clone()]);
    enlarged.commands = Arc::from([projected]);
    state.scroll_motion_groups.groups =
        std::rc::Rc::new(BTreeMap::from([(target, enlarged.clone())]));
    assert_eq!(
        state.indexed_motion_reserved_bytes().unwrap() - initial,
        size_of::<UiMountedScrollMotionClip>(),
        "the projected allocation has its own charge"
    );
    let mut member = enlarged.input.members[0].clone();
    member.clips = Arc::from([clip.clone(), clip]);
    enlarged.input.members = Arc::from([member]);
    state.scroll_motion_groups.groups = std::rc::Rc::new(BTreeMap::from([(target, enlarged)]));
    assert_eq!(
        state.indexed_motion_reserved_bytes().unwrap() - initial,
        2 * size_of::<UiMountedScrollMotionClip>(),
        "input provenance remains separately retained"
    );

    let basis = crate::mounting::UiMountedVisualRegionBasis::new(Box::default(), Box::default());
    let direct = 4 * size_of::<crate::runtime::scroll::UiPreparedScrollDirectSuccession>()
        + 2 * (size_of::<crate::runtime::scroll::UiScrollOwnerIdentity>() + 6 * size_of::<usize>());
    assert_eq!(
        basis
            .clone()
            .with_direct_scroll_reservation(2)
            .motion_acceptance_reserved_bytes(),
        Some(2 * direct),
        "both pending owners and exact frame/publication snapshots are charged"
    );
    assert_eq!(
        basis
            .clone()
            .with_direct_scroll_reservation(usize::MAX)
            .motion_acceptance_reserved_bytes(),
        None
    );
    assert_eq!(
        basis
            .clone()
            .with_indexed_motion_reservation(Some(initial))
            .motion_acceptance_reserved_bytes(),
        Some(initial)
    );
    assert_eq!(
        basis
            .with_indexed_motion_reservation(None)
            .motion_acceptance_reserved_bytes(),
        None,
        "overflow reaches the existing retention admission instead of wrapping"
    );
}
