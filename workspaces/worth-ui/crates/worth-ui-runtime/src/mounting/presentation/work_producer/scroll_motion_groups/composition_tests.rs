//! Producer-unit proof of physical composition, not authored topology admission.
use super::group_offset::{
    UiBoundGroupStanding, UiGroupStanding, UiPublishedGroupOffset, UiScrollGroupBind,
};
use super::*;
use crate::mounting::presentation::presented_surface_witness_for_certification;
use crate::mounting::presentation::work_producer_tests::world::{
    rect_spec, MountedPresentationWorld,
};
use crate::mounting::presentation::{
    motion_sampling::UiMountedMotionSampler, UiMountedPresentationLeaseGate,
};
use crate::runtime::motion::{UiMotionCommitReceipt, UiMotionDeclaration};
use worth_ui_host_contract::*;

#[test]
fn nested_scroll_samples_compose_once_and_keep_a_settled_ancestor() {
    let world = MountedPresentationWorld::new();
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let projection = world.projection(frame, [rect_spec(world.first_instance, 0.0)]);
    let command = projection.authored_paint_commands()[0].identity();
    let mut state =
        UiMountedPresentationState::from_projection(&projection, world.requirement, None);
    let outer = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let inner = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let child = world.first_instance;
    let target = |owner, key| {
        UiMotionTargetIdentity::from_scroll_region_owner(
            world.requirement.semantic_surface(),
            owner,
            key,
        )
    };
    let outer_target = target(outer, 81);
    let inner_target = target(inner, 82);
    let outer_clip = rect(0.0, 100.0);
    let inner_clip = rect(20.0, 40.0);
    let clips: Arc<[_]> = Arc::from([
        UiMountedScrollMotionClip {
            bounds: outer_clip,
            owner: Some(outer),
        },
        UiMountedScrollMotionClip {
            bounds: inner_clip,
            owner: Some(inner),
        },
    ]);
    let group = |target, owner, content, viewport, mut members: Vec<_>| {
        members.sort();
        UiMountedScrollMotionGroup {
            input: UiMountedScrollMotionGroupInput {
                target,
                owner,
                content,
                rest: content,
                viewport,
                offset: UiScrollOffset::origin(),
                scale: UiScrollPresentationDeviceScale::admit(1000).unwrap(),
                chrome: None,
                members: members
                    .into_iter()
                    .map(|instance| UiMountedScrollMotionMember {
                        instance,
                        clips: clips.clone(),
                    })
                    .collect::<Vec<_>>()
                    .into(),
            },
            commands: Arc::from([UiMountedScrollMotionCommand {
                identity: command,
                clips: clips.clone(),
                base_translation: None,
            }]),
            thumbs: Arc::from([]),
            bound_standing: UiBoundGroupStanding::new(
                UiGroupStanding::Published(UiPublishedGroupOffset::of(UiScrollOffset::origin())),
                UiScrollGroupBind::default(),
            ),
            displayed_sample: Default::default(),
        }
    };
    state.scroll_motion_groups.groups = std::rc::Rc::new(BTreeMap::from([
        (
            outer_target,
            group(
                outer_target,
                outer,
                rect(0.0, 200.0),
                outer_clip,
                vec![inner, child],
            ),
        ),
        (
            inner_target,
            group(
                inner_target,
                inner,
                rect(20.0, 80.0),
                inner_clip,
                vec![child],
            ),
        ),
    ]));
    state.scroll_motion_groups.memberships = Arc::new(HashMap::from([(
        command,
        Arc::from([outer_target, inner_target]),
    )]));
    let presentation = UiHostObservationPresentationBasis::new(
        world.requirement.host_surface(),
        frame,
        world.requirement.binding(),
        UiHostPresentationEpoch::issued_by_host(1),
    );
    let mut sampler = UiMountedMotionSampler::default();
    let install = |identity, target, from, to, height| {
        UiMotionCommitReceipt::for_sampling_test_transition(
            identity,
            target,
            presentation,
            Some([0.0, from, 100.0, height]),
            true,
            Some([0.0, to, 100.0, height]),
            true,
            UiMotionDeclaration::scroll_settle(120),
            None,
        )
    };
    sampler
        .install(install(81, outer_target, 0.0, -10.0, 200.0))
        .unwrap();
    sampler
        .install(install(82, inner_target, 20.0, 15.0, 80.0))
        .unwrap();
    let lease = UiMountedPresentationLeaseGate::default().claim().unwrap();
    let rest = sampler.prepare_tick(1, presentation).unwrap();
    let (_, acceptance) = state
        .prepare_motion_sample(rest.receipt(), presentation, &lease)
        .unwrap();
    acceptance
        .accept(
            &state,
            &presented_surface_witness_for_certification(presentation),
        )
        .unwrap();
    sampler.commit_prepared(rest.presented_for_certification());
    let displayed_at_rest = state
        .scroll_motion_groups
        .groups
        .values()
        .map(|group| group.displayed_sample.get())
        .collect::<Vec<_>>();
    assert!(
        displayed_at_rest.iter().all(Option::is_some),
        "the witness displayed every group's rest sample"
    );
    let moving = sampler.prepare_tick(121, presentation).unwrap();
    let (work, rejected) = state
        .prepare_motion_sample(moving.receipt(), presentation, &lease)
        .unwrap();
    let UiMountedPresentationWorkView::Sample(work) = work.view() else {
        panic!("physical sample");
    };
    assert_eq!(
        work.changes().len(),
        1,
        "nested groups issue one atomic command change"
    );
    assert_translation(work.changes()[0], -15.0);
    assert_eq!(
        work.changes()[0].clip(),
        Some(rect(10.0, 40.0)),
        "inner clip moves with outer, never with its own content"
    );
    let before = state.accepted_motion_change(command);
    drop(rejected);
    assert_eq!(state.accepted_motion_change(command), before);
    assert_eq!(
        state
            .scroll_motion_groups
            .groups
            .values()
            .map(|group| group.displayed_sample.get())
            .collect::<Vec<_>>(),
        displayed_at_rest,
        "rejection moves no group evidence"
    );
    let (_, accepted) = state
        .prepare_motion_sample(moving.receipt(), presentation, &lease)
        .unwrap();
    accepted
        .accept(
            &state,
            &presented_surface_witness_for_certification(presentation),
        )
        .unwrap();
    sampler.commit_prepared(moving.presented_for_certification());

    sampler
        .install(install(83, inner_target, 15.0, 12.0, 80.0))
        .unwrap();
    let rest = sampler.prepare_tick(200, presentation).unwrap();
    let (_, accepted) = state
        .prepare_motion_sample(rest.receipt(), presentation, &lease)
        .unwrap();
    accepted
        .accept(
            &state,
            &presented_surface_witness_for_certification(presentation),
        )
        .unwrap();
    sampler.commit_prepared(rest.presented_for_certification());
    let next = sampler.prepare_tick(320, presentation).unwrap();
    assert_eq!(
        next.receipt().samples().len(),
        1,
        "outer is settled, not resampled"
    );
    let (work, _) = state
        .prepare_motion_sample(next.receipt(), presentation, &lease)
        .unwrap();
    let UiMountedPresentationWorkView::Sample(work) = work.view() else {
        panic!("physical sample");
    };
    assert_translation(work.changes()[0], -18.0);
    assert_eq!(work.changes()[0].clip(), Some(rect(10.0, 40.0)));
}

#[test]
fn a_group_rebuilt_before_its_accepted_sample_settles_moves_from_where_the_host_shows_it() {
    let world = MountedPresentationWorld::new();
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let projection = world.projection(frame, [rect_spec(world.first_instance, 0.0)]);
    let command = projection.authored_paint_commands()[0].identity();
    let mut state =
        UiMountedPresentationState::from_projection(&projection, world.requirement, None);
    let owner = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let target = UiMotionTargetIdentity::from_scroll_region_owner(
        world.requirement.semantic_surface(),
        owner,
        81,
    );
    let clips: Arc<[_]> = Arc::from([UiMountedScrollMotionClip {
        bounds: rect(0.0, 100.0),
        owner: Some(owner),
    }]);
    // The publication still names the origin: the accepted sample has not
    // settled into the semantic offset when the group is bound again.
    let bind = |state: &UiMountedPresentationState| UiMountedScrollMotionGroup {
        input: UiMountedScrollMotionGroupInput {
            target,
            owner,
            content: rect(0.0, 200.0),
            rest: rect(0.0, 200.0),
            viewport: rect(0.0, 100.0),
            offset: UiScrollOffset::origin(),
            scale: UiScrollPresentationDeviceScale::admit(1000).unwrap(),
            chrome: None,
            members: Arc::from([UiMountedScrollMotionMember {
                instance: world.first_instance,
                clips: clips.clone(),
            }]),
        },
        commands: Arc::from([UiMountedScrollMotionCommand {
            identity: command,
            clips: clips.clone(),
            base_translation: state
                .displayed_base_translation(command, UiScrollGroupBind::default()),
        }]),
        thumbs: Arc::from([]),
        bound_standing: UiBoundGroupStanding::new(
            state.group_standing(target, UiScrollOffset::origin()),
            UiScrollGroupBind::default(),
        ),
        displayed_sample: Default::default(),
    };
    state.scroll_motion_groups.groups = std::rc::Rc::new(BTreeMap::from([(target, bind(&state))]));
    state.scroll_motion_groups.memberships =
        Arc::new(HashMap::from([(command, Arc::from([target]))]));
    let presentation = UiHostObservationPresentationBasis::new(
        world.requirement.host_surface(),
        frame,
        world.requirement.binding(),
        UiHostPresentationEpoch::issued_by_host(1),
    );
    let mut sampler = UiMountedMotionSampler::default();
    let install = |identity, from, to| {
        UiMotionCommitReceipt::for_sampling_test_transition(
            identity,
            target,
            presentation,
            Some([0.0, from, 100.0, 200.0]),
            true,
            Some([0.0, to, 100.0, 200.0]),
            true,
            UiMotionDeclaration::scroll_settle(120),
            None,
        )
    };
    let lease = UiMountedPresentationLeaseGate::default().claim().unwrap();
    let present =
        |state: &UiMountedPresentationState, sampler: &mut UiMountedMotionSampler, tick| {
            let prepared = sampler.prepare_tick(tick, presentation).unwrap();
            let (work, acceptance) = state
                .prepare_motion_sample(prepared.receipt(), presentation, &lease)
                .unwrap();
            let UiMountedPresentationWorkView::Sample(sample) = work.view() else {
                panic!("physical sample");
            };
            let change = sample.changes()[0];
            acceptance
                .accept(
                    state,
                    &presented_surface_witness_for_certification(presentation),
                )
                .unwrap();
            sampler.commit_prepared(prepared.presented_for_certification());
            change
        };
    sampler.install(install(81, 0.0, -40.0)).unwrap();
    present(&state, &mut sampler, 1);
    assert_translation(present(&state, &mut sampler, 121), -40.0);

    let rebound = bind(&state);
    assert_eq!(
        rebound.commands[0]
            .base_translation
            .map(|base| base.components()),
        Some([0.0, -40.0])
    );
    state.scroll_motion_groups.groups = std::rc::Rc::new(BTreeMap::from([(target, rebound)]));
    sampler.install(install(82, -40.0, -60.0)).unwrap();
    // The host shows the content at -40; the next settle departs from there.
    assert_translation(present(&state, &mut sampler, 200), -40.0);
    assert_translation(present(&state, &mut sampler, 320), -60.0);
    let standing = state.scroll_motion_groups.groups[&target]
        .bound_standing
        .standing();
    assert!(matches!(standing, UiGroupStanding::Displayed(..)));
    assert_eq!(standing.points(), [0.0, 40.0]);
}

pub(super) fn rect(y: f32, height: f32) -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: 0.0,
        y,
        width: 100.0,
        height,
        coordinate_space: UiMountedCoordinateSpace::Viewport,
    })
    .unwrap()
}

pub(super) fn assert_translation(change: UiMountedPresentationSampleChange, dy: f32) {
    let transform = change.transform().unwrap();
    assert_eq!(transform.sampled().y() - transform.source().y(), dy);
}
