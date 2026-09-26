//! A command bound with no displayed base is shown where its group stands.
use super::super::super::command_motion_slot::UiDisplayedCommandMotion;
use super::super::composition_tests::{assert_translation, rect};
use super::super::group_offset::{UiBoundGroupStanding, UiScrollGroupBind};
use super::super::*;
use crate::mounting::presentation::presented_surface_witness_for_certification;
use crate::mounting::presentation::work_producer_tests::world::{
    rect_spec, MountedPresentationWorld,
};
use crate::mounting::presentation::{
    motion_sampling::UiMountedMotionSampler, UiMountedPresentationLeaseGate,
};
use crate::mounting::projection::UiMountedAppearanceSurfaceSampleGeometry;
use crate::runtime::motion::{UiMotionCommitReceipt, UiMotionDeclaration};
use crate::runtime::scroll::chrome::{
    UiScrollAdmittedChrome, UiScrollChromeAxis, UiScrollChromeAxisSupport, UiScrollChromeFacts,
    UiScrollChromeMetrics,
};
use crate::runtime::scroll::UiScrollBounds;
use worth_ui_host_contract::*;

/// A frame projecting one rect, and the Scroll group a settle moves from
/// the origin 40 points down its content.
struct Settling {
    world: MountedPresentationWorld,
    presentation: UiHostObservationPresentationBasis,
    state: UiMountedPresentationState,
    rect: UiMountedPaintCommandIdentity,
    owner: UiMountedInstanceIdentity,
    target: UiMotionTargetIdentity,
    sampler: UiMountedMotionSampler,
}

impl Settling {
    fn new() -> Self {
        let world = MountedPresentationWorld::new();
        let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
        let projection = world.projection(frame, [rect_spec(world.first_instance, 0.0)]);
        let rect = projection.authored_paint_commands()[0].identity();
        let state =
            UiMountedPresentationState::from_projection(&projection, world.requirement, None);
        let owner = UiMountedInstanceIdentity::mint_unbound().unwrap();
        let target = UiMotionTargetIdentity::from_scroll_region_owner(
            world.requirement.semantic_surface(),
            owner,
            81,
        );
        let presentation = UiHostObservationPresentationBasis::new(
            world.requirement.host_surface(),
            frame,
            world.requirement.binding(),
            UiHostPresentationEpoch::issued_by_host(1),
        );
        let mut sampler = UiMountedMotionSampler::default();
        sampler
            .install(UiMotionCommitReceipt::for_sampling_test_transition(
                81,
                target,
                presentation,
                Some([0.0, 0.0, 100.0, 200.0]),
                true,
                Some([0.0, -40.0, 100.0, 200.0]),
                true,
                UiMotionDeclaration::scroll_settle(120),
                None,
            ))
            .unwrap();
        Self {
            world,
            presentation,
            state,
            rect,
            owner,
            target,
            sampler,
        }
    }

    /// Bind the group over `commands` as a frame that replaced each of them
    /// binds it: with no displayed base.
    fn rebind(&mut self, commands: &[UiMountedPaintCommandIdentity]) {
        let clips: Arc<[_]> = Arc::from([UiMountedScrollMotionClip {
            bounds: rect(0.0, 100.0),
            owner: Some(self.owner),
        }]);
        let thumbs = commands
            .iter()
            .filter_map(|command| command.scroll_chrome_identity())
            .collect::<Vec<_>>();
        let group = UiMountedScrollMotionGroup {
            input: UiMountedScrollMotionGroupInput {
                target: self.target,
                owner: self.owner,
                content: rect(0.0, 200.0),
                rest: rect(0.0, 200.0),
                viewport: rect(0.0, 100.0),
                offset: UiScrollOffset::origin(),
                scale: UiScrollPresentationDeviceScale::admit(1000).unwrap(),
                chrome: (!thumbs.is_empty()).then(block_chrome),
                members: Arc::from([UiMountedScrollMotionMember {
                    instance: self.world.first_instance,
                    clips: clips.clone(),
                }]),
            },
            commands: commands
                .iter()
                .map(|identity| UiMountedScrollMotionCommand {
                    identity: *identity,
                    clips: clips.clone(),
                    base_translation: None,
                })
                .collect(),
            thumbs: Arc::from(thumbs),
            bound_standing: UiBoundGroupStanding::new(
                self.state
                    .group_standing(self.target, UiScrollOffset::origin()),
                UiScrollGroupBind::default(),
            ),
            displayed_sample: Default::default(),
        };
        let groups = &mut self.state.scroll_motion_groups;
        groups.groups = std::rc::Rc::new(BTreeMap::from([(self.target, group)]));
        groups.memberships = Arc::new(
            commands
                .iter()
                .map(|command| (*command, Arc::from([self.target])))
                .collect(),
        );
        groups.owners = Arc::new(BTreeMap::from([(self.owner, self.target)]));
    }

    /// Display the settle's samples at each of `ticks`.
    fn display(&mut self, ticks: &[u64]) {
        let lease = UiMountedPresentationLeaseGate::default().claim().unwrap();
        for &tick in ticks {
            let prepared = self.sampler.prepare_tick(tick, self.presentation).unwrap();
            let (_, acceptance) = self
                .state
                .prepare_motion_sample(prepared.receipt(), self.presentation, &lease)
                .unwrap();
            acceptance
                .accept(
                    &self.state,
                    &presented_surface_witness_for_certification(self.presentation),
                )
                .unwrap();
            self.sampler
                .commit_prepared(prepared.presented_for_certification());
        }
    }

    /// Replace `command` with one whose new slot shows nothing yet, and
    /// show each unbased command where its group stands.
    fn replace_and_show(
        &mut self,
        command: UiMountedPaintCommandIdentity,
    ) -> UiDisplayedCommandMotion {
        self.state.own_motion_slot(command).unwrap();
        self.state
            .show_unbased_commands_where_groups_stand()
            .unwrap();
        assert_eq!(self.state.carried_motion, vec![command]);
        self.state
            .motion_slot(command)
            .and_then(UiCommandMotionAcceptance::displayed)
            .expect("the host is told where the group stands")
    }
}

fn block_chrome() -> UiScrollAdmittedChrome {
    let role = |name| worth_ui_dsl::UiAppearanceRoleIdentity::new(name).unwrap();
    UiScrollAdmittedChrome::admit_declared_chrome(
        UiScrollChromeAxisSupport::Block,
        role("platform.pulse.appearance.scroll_track"),
        role("platform.pulse.appearance.scroll_thumb"),
        UiScrollChromeMetrics::declared(),
    )
    .unwrap()
}

/// Where the group's block thumb stands `points` down its content.
fn thumb_at(points: i64) -> UiMountedCanonicalBox {
    UiScrollChromeFacts::derive(
        rect(0.0, 100.0),
        UiScrollBounds::from_mounted_region(rect(0.0, 200.0), rect(0.0, 100.0)).unwrap(),
        UiScrollOffset::new(0, points * 1_000).unwrap(),
        &block_chrome(),
    )
    .unwrap()
    .axis(UiScrollChromeAxis::Block)
    .unwrap()
    .thumb()
}

#[test]
fn a_replaced_command_is_shown_where_its_displayed_group_stands() {
    let mut settling = Settling::new();
    let command = settling.rect;
    // No witness has displayed the group: the frame's layout is where it
    // stands, and nothing is carried.
    settling.rebind(&[command]);
    settling
        .state
        .show_unbased_commands_where_groups_stand()
        .unwrap();
    assert!(settling.state.carried_motion.is_empty());

    settling.display(&[1, 121]);
    settling.rebind(&[command]);
    let shown = settling.replace_and_show(command);
    assert_translation(shown.change, -40.0);
    assert_eq!(shown.change.clip(), Some(rect(0.0, 100.0)));

    // A command already shown there is not told again.
    settling.rebind(&[command]);
    settling
        .state
        .show_unbased_commands_where_groups_stand()
        .unwrap();
    assert_eq!(settling.state.carried_motion, vec![command]);
}

#[test]
fn a_group_displayed_where_the_frame_lays_it_out_carries_nothing() {
    let mut settling = Settling::new();
    let command = settling.rect;
    settling.rebind(&[command]);
    // The settle's first sample holds the group at the origin it was
    // published at: a layer there would move nothing.
    settling.display(&[1]);
    settling.rebind(&[command]);
    settling.state.own_motion_slot(command).unwrap();
    settling
        .state
        .show_unbased_commands_where_groups_stand()
        .unwrap();
    assert!(settling.state.carried_motion.is_empty());
    assert!(settling
        .state
        .motion_slot(command)
        .and_then(UiCommandMotionAcceptance::displayed)
        .is_none());
}

#[test]
fn a_replaced_appearance_surface_is_shown_where_its_group_stands() {
    let mut settling = Settling::new();
    let instance = settling.world.second_instance;
    let surface = UiMountedPaintCommandIdentity::appearance_surface(instance);
    let geometry = UiMountedAppearanceSurfaceSampleGeometry::for_sampling_test(
        rect(10.0, 20.0),
        rect(0.0, 100.0),
    );
    assert!(settling
        .state
        .bind_appearance_surface_target(instance, Some(geometry)));
    settling.rebind(&[surface]);
    settling.display(&[1, 121]);

    settling.rebind(&[surface]);
    let shown = settling.replace_and_show(surface);
    assert_translation(shown.change, -40.0);
    assert_eq!(shown.change.clip(), Some(rect(0.0, 100.0)));
}

#[test]
fn a_rebound_thumb_is_shown_where_its_group_stands() {
    let mut settling = Settling::new();
    let identity = UiMountedScrollChromeIdentity::from_runtime_mounting(
        settling.owner,
        UiMountedScrollChromeAxis::Block,
        UiMountedScrollChromePart::Thumb,
    );
    let thumb = UiMountedPaintCommandIdentity::scroll_chrome(identity);
    let at_rest = thumb_at(0);
    std::rc::Rc::make_mut(&mut settling.state.scroll_motion_groups.chrome).insert(
        identity,
        UiMountedScrollChromeSampleTarget {
            bounds: at_rest,
            clip: rect(0.0, 100.0),
            opacity: UiMountedAppearanceOpacity::ONE,
            portal: None,
            motion: UiCommandMotionAcceptance::default(),
        },
    );
    settling.rebind(&[thumb]);
    settling.display(&[1, 121]);

    // A frame that binds the bars anew gives the thumb a new slot.
    settling.rebind(&[thumb]);
    let shown = settling.replace_and_show(thumb);
    let transform = shown.change.transform().unwrap();
    assert_eq!(transform.source(), at_rest);
    // Thumbs snap to the device grid, so the one shown lands within half a
    // device pixel of the thumb 40 points down.
    let settled = thumb_at(40);
    assert!(settled.y() > at_rest.y());
    assert!((transform.sampled().y() - settled.y()).abs() <= 0.5);
    assert_eq!(transform.sampled().height(), at_rest.height());
}

#[test]
fn a_command_shown_through_a_portal_keeps_the_portal_sample_it_shows() {
    let mut settling = Settling::new();
    let command = settling.rect;
    settling.rebind(&[command]);
    settling.display(&[1, 121]);

    // The frame replaces the command, and a Portal tick shows its new slot
    // through the Portal alone.
    settling.rebind(&[command]);
    settling.state.own_motion_slot(command).unwrap();
    let mut portal = UiMountedMotionSampler::default();
    portal
        .install(UiMotionCommitReceipt::for_sampling_test_transition(
            840,
            UiMotionTargetIdentity::from_portal_owner(
                settling.world.requirement.semantic_surface(),
                settling.world.first_instance,
                settling.world.first_instance.diagnostic_value(),
            ),
            settling.presentation,
            None,
            true,
            None,
            false,
            UiMotionDeclaration::portal_exit(),
            None,
        ))
        .unwrap();
    let lease = UiMountedPresentationLeaseGate::default().claim().unwrap();
    let tick = portal.prepare_tick(122, settling.presentation).unwrap();
    let (_, acceptance) = settling
        .state
        .prepare_motion_sample(tick.receipt(), settling.presentation, &lease)
        .unwrap();
    acceptance
        .accept(
            &settling.state,
            &presented_surface_witness_for_certification(settling.presentation),
        )
        .unwrap();
    let held = settling
        .state
        .motion_slot(command)
        .and_then(UiCommandMotionAcceptance::displayed)
        .expect("the Portal tick shows the new slot");
    assert!(held.layers.portal_only().is_some());
    assert!(held.layers.scroll_layer().is_none());

    // The Scroll layer joins the Portal one under the Portal's sample.
    settling
        .state
        .show_unbased_commands_where_groups_stand()
        .unwrap();
    assert_eq!(settling.state.carried_motion, vec![command]);
    let shown = settling
        .state
        .motion_slot(command)
        .and_then(UiCommandMotionAcceptance::displayed)
        .unwrap();
    assert_eq!(shown.sample, held.sample);
    assert_eq!(shown.layers.portal_only(), held.layers.portal_only());
    let scroll = shown.layers.scroll_transform().unwrap();
    assert_eq!(scroll.sampled().y() - scroll.source().y(), -40.0);
}
