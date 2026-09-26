//! Lowering a Scroll transition target into the Motion transition request that
//! will interpolate it.
//!
//! Scroll never samples and never interpolates. It states where the content is
//! now, where it is going, and how long the settle horizon has left; Motion
//! owns the curve, the retarget and the scheduling. The target is bound to an
//! exact Scroll region occurrence through `UiMotionTargetIdentity::from_scroll_region_owner`,
//! keeping it structurally distinct from that occurrence's ordinary and
//! Portal-content targets.

/// The mounted evidence a Scroll transition needs before it can address Motion.
/// The mounted instance, the Motion owner key and the revisions come from the
/// preparation that resolved this occurrence, so this file never infers
/// identity from a proxy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiScrollMotionBinding {
    mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    motion_owner_key: u64,
    predecessor_revision: u64,
    successor_revision: u64,
    presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    content: crate::mounting::UiLaidOut<worth_ui_host_contract::UiMountedCanonicalBox>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiScrollMotionRequestDenial {
    /// The translated content box left the finite, non-negative-extent range
    /// Motion admits for semantic geometry.
    ContentGeometryInadmissible,
    /// The settle horizon has already ended at the preparation tick, so there
    /// is no remaining duration for Motion to interpolate over.
    SettleHorizonExhausted,
    Motion(crate::runtime::motion::UiMotionTransitionRequestDenial),
}

impl UiScrollMotionBinding {
    pub(crate) const fn new(
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        motion_owner_key: u64,
        predecessor_revision: u64,
        successor_revision: u64,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        content: crate::mounting::UiLaidOut<worth_ui_host_contract::UiMountedCanonicalBox>,
    ) -> Self {
        Self {
            mounted_instance,
            motion_owner_key,
            predecessor_revision,
            successor_revision,
            presentation,
            content,
        }
    }
}

/// The Motion request that settles `target` from the accepted displayed offset.
/// The remaining settle horizon at `tick`, not a fixed duration, is the
/// declared settle length, so a notch arriving mid-settle extends one
/// transition instead of restarting another.
pub(crate) fn scroll_settle_motion_request(
    target: super::UiScrollTransitionTarget,
    accepted_offset: super::super::UiScrollOffset,
    tick: u64,
    binding: UiScrollMotionBinding,
) -> Result<crate::runtime::motion::UiMotionTransitionRequest, UiScrollMotionRequestDenial> {
    let remaining = target.horizon().remaining_ticks(tick);
    if remaining == 0 {
        return Err(UiScrollMotionRequestDenial::SettleHorizonExhausted);
    }
    let settle_ticks = u32::try_from(remaining)
        .map_err(|_| UiScrollMotionRequestDenial::SettleHorizonExhausted)?;
    motion_request(target, accepted_offset, settle_ticks, binding)
}

/// An extent accepted after the input horizon still owes its final lawful
/// target. It settles on the next eligible sample instead of discarding that
/// target or starting a fresh wheel duration.
pub(crate) fn scroll_extent_motion_request(
    target: super::UiScrollTransitionTarget,
    accepted_offset: super::super::UiScrollOffset,
    tick: u64,
    binding: UiScrollMotionBinding,
) -> Result<crate::runtime::motion::UiMotionTransitionRequest, UiScrollMotionRequestDenial> {
    let remaining = target.horizon().remaining_ticks(tick).max(1);
    let settle_ticks = u32::try_from(remaining)
        .map_err(|_| UiScrollMotionRequestDenial::SettleHorizonExhausted)?;
    motion_request(target, accepted_offset, settle_ticks, binding)
}

fn motion_request(
    target: super::UiScrollTransitionTarget,
    accepted_offset: super::super::UiScrollOffset,
    settle_ticks: u32,
    binding: UiScrollMotionBinding,
) -> Result<crate::runtime::motion::UiMotionTransitionRequest, UiScrollMotionRequestDenial> {
    let identity = crate::runtime::motion::UiMotionTargetIdentity::from_scroll_region_owner(
        target.owner().semantic_surface(),
        binding.mounted_instance,
        binding.motion_owner_key,
    );
    let predecessor = scroll_content_geometry(binding.content, accepted_offset)?;
    let successor = scroll_content_geometry(binding.content, target.target_offset())?;
    crate::runtime::motion::UiMotionTransitionRequest::from_family_transition(
        identity,
        crate::runtime::motion::UiMotionTransitionEndpoint::new(
            binding.predecessor_revision,
            binding.presentation,
            Some(predecessor),
            true,
        ),
        crate::runtime::motion::UiMotionTransitionEndpoint::new(
            binding.successor_revision,
            binding.presentation,
            Some(successor),
            true,
        ),
        crate::runtime::motion::UiMotionDeclaration::scroll_settle(settle_ticks),
    )
    .map_err(UiScrollMotionRequestDenial::Motion)
}

/// The content box as the scrolled offset places it. A positive scroll offset
/// moves content toward the viewport origin, so the translation is negative.
///
/// A Scroll track moves content where it is laid out, so every sample it
/// yields is measured from a laid-out rest; a frame presenting the region
/// through a Portal moves the track's translation with the region.
pub(crate) fn scroll_content_geometry(
    content: crate::mounting::UiLaidOut<worth_ui_host_contract::UiMountedCanonicalBox>,
    offset: super::super::UiScrollOffset,
) -> Result<crate::mounting::presentation::UiPublishedRect, UiScrollMotionRequestDenial> {
    let content = content.into_layout_space();
    let points = |subpixels: i64| {
        subpixels as f32
            / worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f32
    };
    crate::mounting::presentation::UiPublishedRect::from_committed_components(
        [
            content.x() - points(offset.inline_subpixels()),
            content.y() - points(offset.block_subpixels()),
            content.width(),
            content.height(),
        ],
        content.coordinate_space(),
    )
    .map_err(|_| UiScrollMotionRequestDenial::ContentGeometryInadmissible)
}
