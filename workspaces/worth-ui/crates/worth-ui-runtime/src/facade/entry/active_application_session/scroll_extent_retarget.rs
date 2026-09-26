//! Accept prepared layout offsets and reconcile Motion from that exact surface publication.
use crate::mounting::{UiMountedFrameOutcome, WorthUiMountedSessionState};
use crate::runtime::motion::{UiMotionRuntimeState, UiMotionTargetIdentity, UiMotionTerminalCause};
use crate::runtime::scroll::transition::{
    scroll_content_geometry, scroll_extent_motion_request, UiScrollMotionBinding,
};
use crate::runtime::scroll::UiScrollRuntimeState;

/// Settle what a publication committed for Scroll, lead the rows it
/// published on each surface whose settle is owed, then assert in debug
/// builds that displayed geometry stands on the latest witness. Every
/// publication path calls this once its transition is finished.
pub(in crate::facade::entry) fn settle_presented_scroll_extent(
    mut scroll: Option<&mut UiScrollRuntimeState>,
    motion: Option<&mut UiMotionRuntimeState>,
    mounted: &mut WorthUiMountedSessionState,
    interaction: &mut crate::runtime::interaction::UiInteractionRuntimeState,
    settles: &super::scroll_settle_retry::UiOwedScrollSettles,
    outcome: &UiMountedFrameOutcome,
    now: u64,
) {
    retarget_presented_scroll_extent(scroll.as_deref_mut(), motion, mounted, outcome, now);
    super::scroll_settle_hit_lead::UiScrollHitLeadParts {
        mounted,
        scroll: scroll.as_deref(),
        interaction,
    }
    .lead_published(settles, outcome);
    super::scroll_witness_invariant::debug_assert_scroll_geometry_witnessed(
        super::UiScrollSettlementReading {
            mounted,
            scroll: scroll.as_deref(),
        },
        interaction,
        settles,
    );
}

fn retarget_presented_scroll_extent(
    scroll: Option<&mut UiScrollRuntimeState>,
    mut motion: Option<&mut UiMotionRuntimeState>,
    mounted: &mut WorthUiMountedSessionState,
    outcome: &UiMountedFrameOutcome,
    now: u64,
) {
    let (UiMountedFrameOutcome::Published(publication)
    | UiMountedFrameOutcome::Reconciled(publication)) = outcome
    else {
        return;
    };
    let Some(scroll) = scroll else {
        return;
    };
    // Direct input carries exact immutable evidence through the accepted frame,
    // independently of whether Motion is installed.
    for prepared in publication.direct_scroll() {
        if publication
            .presentation_for_surface(prepared.surface())
            .is_some()
            && scroll.commit_presented_direct(*prepared)
        {
            let target = super::scroll_direct_control::scroll_content_motion_target(
                prepared.owner(),
                prepared.occurrence(),
            );
            mounted.retire_scroll_motion_sample(target);
            if let Some(motion) = motion.as_deref_mut() {
                motion.terminalize_target(target, UiMotionTerminalCause::DisplacedByDirectControl);
            }
        }
    }
    mounted.settle_published_direct_scroll(publication);
    let mut changed = std::collections::BTreeSet::new();
    publication.with_surface_presentations(|surfaces| {
        for surface in surfaces {
            if scroll.commit_presented_layout(surface.semantic_surface()) {
                changed.insert(surface.semantic_surface());
            }
        }
    });
    let live = scroll
        .pending_settle_occurrences()
        .into_iter()
        .map(|(owner, occurrence)| {
            let target = UiMotionTargetIdentity::from_scroll_region_owner(
                owner.semantic_surface(),
                occurrence,
                super::scroll_transition_preparation::scroll_motion_owner_key(owner),
            );
            (target, owner)
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    // A sample no committed track stands behind has nothing to rebase: its
    // Motion has arrived. Where the frame placed its group, the host shows the
    // group where the frame publishes it. That is where the arrival was
    // carried into the staged layout, and the settle it was owed ends there,
    // as a page ends one. A frame prepared before the arrival, with no layout
    // left to carry it into, pulls the content back to where it was lowered,
    // so the settle is placed where the reader saw it arrive.
    for (target, _) in mounted.accepted_scroll_group_samples() {
        if motion
            .as_deref()
            .is_none_or(|motion| motion.committed_track(target).is_none())
            && mounted.scroll_group_placed_on_screen(target)
        {
            mounted.retire_scroll_motion_sample(target);
            if let Some(owner) = live.get(&target) {
                if !super::scroll_arrival_placement::place_pulled_back_arrival(
                    scroll, mounted, *owner, target,
                ) {
                    scroll.retire_transition(*owner);
                }
            }
        }
    }
    if changed.is_empty() {
        return;
    }
    let Some(motion) = motion else {
        return;
    };
    for target in motion.committed_scroll_targets() {
        if changed.contains(&target.semantic_surface()) && !live.contains_key(&target) {
            mounted.retire_scroll_motion_sample(target);
            motion.terminalize_target(target, UiMotionTerminalCause::NowhereLeftToSettle);
        }
    }
    for (owner, occurrence) in scroll.pending_settle_occurrences() {
        if !changed.contains(&owner.semantic_surface()) {
            continue;
        }
        let target = UiMotionTargetIdentity::from_scroll_region_owner(
            owner.semantic_surface(),
            occurrence,
            super::scroll_transition_preparation::scroll_motion_owner_key(owner),
        );
        let Some(track) = motion.committed_track(target) else {
            continue;
        };
        let Some(presentation) = publication
            .presentation_for_surface(owner.semantic_surface())
            .map(|displayed| displayed.basis())
        else {
            continue;
        };
        if track.successor_presentation().binding() != presentation.binding() {
            scroll.retire_transition(owner);
            mounted.retire_scroll_motion_sample(target);
            motion.terminalize_target(target, UiMotionTerminalCause::ReboundAway);
            continue;
        }
        let Some(slot) = scroll.ownership_chain(occurrence).ok().and_then(|chain| {
            chain
                .owners()
                .iter()
                .position(|candidate| *candidate == owner)
        }) else {
            continue;
        };
        let (Some((owner_instance, _, _)), Some(content)) = (
            mounted.scroll_region_geometry(occurrence, slot),
            mounted.scroll_region_rest(occurrence, slot),
        ) else {
            continue;
        };
        let Some(basis) = mounted.current_mounted_identity_basis(owner_instance) else {
            continue;
        };
        let incarnation = crate::runtime::scroll::UiScrollOwnerIncarnation::from_mount_incarnation(
            basis.mount_incarnation(),
        );
        let Some(pending) = scroll.transition_target(owner, incarnation) else {
            continue;
        };
        let Ok(endpoint) = scroll_content_geometry(content, pending.target_offset()) else {
            continue;
        };
        if track.successor_geometry() == Some(endpoint) {
            continue;
        }
        let accepted = scroll
            .offset(owner, incarnation)
            .expect("resolved Scroll incarnation");
        if accepted == pending.target_offset() {
            scroll.retire_transition(owner);
            mounted.retire_scroll_motion_sample(target);
            motion.terminalize_target(target, UiMotionTerminalCause::NowhereLeftToSettle);
            continue;
        }
        let Some(revision) = track.successor_revision().checked_add(1) else {
            // Revision exhaustion cannot leave an obsolete endpoint active.
            scroll.retire_transition(owner);
            mounted.retire_scroll_motion_sample(target);
            motion.terminalize_target(target, UiMotionTerminalCause::ReboundAway);
            continue;
        };
        let request = scroll_extent_motion_request(
            pending,
            accepted,
            now,
            UiScrollMotionBinding::new(
                occurrence,
                target.owner_key(),
                track.successor_revision(),
                revision,
                presentation,
                content,
            ),
        )
        .expect("accepted finite extent lowers with the remaining settle horizon");
        let commit = motion
            .reconcile_presented_scroll_extent(request, publication)
            .expect("accepted extent names the current Scroll track and exact surface binding");
        mounted
            .install_motion_commit(commit)
            .expect("existing track preserves sampler capacity");
        mounted.rebase_presented_scroll_extent(target, now);
    }
}
