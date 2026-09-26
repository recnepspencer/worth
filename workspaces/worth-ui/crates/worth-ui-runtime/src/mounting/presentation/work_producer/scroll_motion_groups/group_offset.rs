//! Scroll group offsets and translations by truth status.
//!
//! A group stands where its retained commands show it: at its published
//! offset until a witness displays it, and at the displayed offset after. Each
//! sample tick the prepared candidate puts the group at an accepted offset.
//! Every translation this lane issues runs from a standing to the candidate,
//! so each one names its source status, and its destination is always
//! accepted. The components leave only at the GPU-upload edge, as a command
//! transform or clip.
//!
//! Two crossings take evidence. An accepted offset carries the presentation
//! basis its sample was accepted on, and a group the tick does not move is
//! held where it stands on the basis of the tick's sample receipt;
//! translations compose only on one basis. A command a witness displayed shows
//! each of its groups where the group stood when the command was bound, so
//! that witness displayed a published standing too; the displayed base and the
//! standing carry the bind that produced them, and must name the same one.

use super::super::motion_sample::UiMountedMotionSampleWorkDenial as Denial;
use crate::mounting::presentation::motion_sampling::UiPresentationMotionSampleReceipt;
use crate::mounting::presentation::{UiAcceptedRect, UiDisplayedRect};
use crate::runtime::scroll::{UiScrollOffset, UiScrollPresentationDeviceScale};
use worth_ui_host_contract::{UiHostObservationPresentationBasis, UiMountedCanonicalBox};

/// One binding of a surface's Scroll groups. Every displayed base and bound
/// standing a bind produces names it, so a base is only ever read against the
/// standings it was bound with.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::mounting::presentation::work_producer) struct UiScrollGroupBind(u64);

/// A group's committed offset, in logical points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct UiPublishedGroupOffset([f64; 2]);

/// The offset a witness displayed a group at, in logical points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct UiDisplayedGroupOffset([f64; 2]);

/// The offset the prepared candidate puts a group at, in logical points, and
/// the presentation basis that candidate was accepted on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct UiAcceptedGroupOffset {
    points: [f64; 2],
    accepted_on: UiHostObservationPresentationBasis,
}

/// Where a group's retained commands show it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum UiGroupStanding {
    /// No witness has displayed the group; it stands where it was published.
    Published(UiPublishedGroupOffset),
    /// A witness displayed the group here, showing `sample`.
    Displayed(UiDisplayedGroupOffset, UiPresentationMotionSampleReceipt),
}

/// Where a group stood when `bound_in` bound it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct UiBoundGroupStanding {
    standing: UiGroupStanding,
    bound_in: UiScrollGroupBind,
}

/// How far a candidate moves content standing at its published offset.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct UiPublishedToAcceptedTranslation {
    delta: [f32; 2],
    accepted_on: UiHostObservationPresentationBasis,
}

/// How far a candidate moves content standing where a witness displayed it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct UiDisplayedToAcceptedTranslation {
    delta: [f32; 2],
    accepted_on: UiHostObservationPresentationBasis,
}

/// The translation a witness displayed on one command's retained transform,
/// kept by the bind that read it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::mounting::presentation::work_producer) struct UiDisplayedCommandTranslation {
    delta: [f32; 2],
    bound_in: UiScrollGroupBind,
}

/// The translation the prepared candidate gives one command.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct UiAcceptedCommandTranslation([f32; 2]);

impl UiScrollGroupBind {
    /// The bind after this one.
    pub(super) const fn next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
}

impl UiPublishedGroupOffset {
    pub(super) fn of(offset: UiScrollOffset) -> Self {
        let unit = worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f64;
        Self([
            offset.inline_subpixels() as f64 / unit,
            offset.block_subpixels() as f64 / unit,
        ])
    }

    pub(super) fn move_to(
        self,
        candidate: UiAcceptedGroupOffset,
        scale: UiScrollPresentationDeviceScale,
    ) -> UiPublishedToAcceptedTranslation {
        UiPublishedToAcceptedTranslation {
            delta: snapped_move(self.0, candidate.points, scale),
            accepted_on: candidate.accepted_on,
        }
    }
}

impl UiDisplayedGroupOffset {
    /// The offset `sample` stands at from the group's rest content box.
    pub(super) fn of_sample(content: UiMountedCanonicalBox, sample: UiDisplayedRect) -> Self {
        Self(offset_from_rest(content, sample.components()))
    }

    pub(super) fn move_to(
        self,
        candidate: UiAcceptedGroupOffset,
        scale: UiScrollPresentationDeviceScale,
    ) -> UiDisplayedToAcceptedTranslation {
        UiDisplayedToAcceptedTranslation {
            delta: snapped_move(self.0, candidate.points, scale),
            accepted_on: candidate.accepted_on,
        }
    }
}

impl UiAcceptedGroupOffset {
    /// Where the candidate `tick` accepts holds a group the tick does not
    /// move: where the host's retained commands already show it, accepted on
    /// the tick's presentation basis.
    pub(super) fn held_by(
        tick: &UiPresentationMotionSampleReceipt,
        standing: UiGroupStanding,
    ) -> Self {
        Self {
            points: standing.points(),
            accepted_on: tick.presentation_basis(),
        }
    }

    /// The offset `sample` stands at from the group's rest content box.
    pub(super) fn of_sample(content: UiMountedCanonicalBox, sample: UiAcceptedRect) -> Self {
        Self {
            points: offset_from_rest(content, sample.components()),
            accepted_on: sample.presentation_basis(),
        }
    }

    /// The offset chrome facts are derived at for this candidate. Derivation
    /// is geometry only: the thumb it yields is issued in the same candidate,
    /// so it is accepted too, never published.
    pub(super) fn chrome_derivation_offset(self) -> Option<UiScrollOffset> {
        let unit = worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f64;
        UiScrollOffset::new(
            (self.points[0].max(0.0) * unit).round() as i64,
            (self.points[1].max(0.0) * unit).round() as i64,
        )
    }
}

impl UiBoundGroupStanding {
    pub(super) const fn new(standing: UiGroupStanding, bound_in: UiScrollGroupBind) -> Self {
        Self { standing, bound_in }
    }

    pub(super) const fn standing(self) -> UiGroupStanding {
        self.standing
    }

    /// Where `base`'s witness displayed this group. The witness showed the
    /// whole command as it was bound, so it displayed a published standing as
    /// well; a base some other bind read shows the group somewhere else and is
    /// refused.
    pub(super) fn shown_by(
        self,
        base: UiDisplayedCommandTranslation,
    ) -> Result<UiDisplayedGroupOffset, Denial> {
        if base.bound_in != self.bound_in {
            return Err(Denial::DisplayedBaseFromAnotherBind);
        }
        Ok(UiDisplayedGroupOffset(self.standing.points()))
    }
}

impl UiGroupStanding {
    /// The sample a witness displayed the group at, if one did.
    pub(super) const fn displayed_by(self) -> Option<UiPresentationMotionSampleReceipt> {
        match self {
            Self::Published(_) => None,
            Self::Displayed(_, sample) => Some(sample),
        }
    }

    /// The offset the group stands at, in logical points.
    pub(super) const fn points(self) -> [f64; 2] {
        match self {
            Self::Published(UiPublishedGroupOffset(points))
            | Self::Displayed(UiDisplayedGroupOffset(points), _) => points,
        }
    }
}

impl UiPublishedToAcceptedTranslation {
    /// Content no group moves on `presentation`.
    pub(super) const fn none(presentation: UiHostObservationPresentationBasis) -> Self {
        Self {
            delta: [0.0, 0.0],
            accepted_on: presentation,
        }
    }

    /// Two groups' moves of the same published content compose, when both
    /// were accepted on one presentation basis.
    pub(super) fn then(self, other: Self) -> Result<Self, Denial> {
        if self.accepted_on != other.accepted_on {
            return Err(Denial::PresentationBasisMismatch);
        }
        Ok(Self {
            delta: add(self.delta, other.delta),
            accepted_on: self.accepted_on,
        })
    }

    /// The components, for the GPU-upload edge: a published box this candidate
    /// moves.
    pub(super) const fn components(self) -> [f32; 2] {
        self.delta
    }
}

impl UiDisplayedCommandTranslation {
    /// The translation a displayed command transform carries, as `bound_in`
    /// read it.
    pub(in crate::mounting::presentation::work_producer) fn of_displayed_transform(
        source: UiMountedCanonicalBox,
        sampled: UiMountedCanonicalBox,
        bound_in: UiScrollGroupBind,
    ) -> Self {
        Self {
            delta: [sampled.x() - source.x(), sampled.y() - source.y()],
            bound_in,
        }
    }

    #[cfg(test)]
    pub(super) const fn components(self) -> [f32; 2] {
        self.delta
    }
}

impl UiAcceptedCommandTranslation {
    /// A command still at its published layout moves by its groups' moves on
    /// `presentation`.
    pub(super) fn from_published(
        presentation: UiHostObservationPresentationBasis,
        moves: impl IntoIterator<Item = UiPublishedToAcceptedTranslation>,
    ) -> Result<Self, Denial> {
        moves
            .into_iter()
            .try_fold(
                UiPublishedToAcceptedTranslation::none(presentation),
                |sum, next| sum.then(next),
            )
            .map(|sum| Self(sum.delta))
    }

    /// A command a witness displayed moves from its displayed transform by
    /// each group's move on `presentation` from where that transform shows
    /// the group.
    pub(super) fn from_displayed(
        presentation: UiHostObservationPresentationBasis,
        base: UiDisplayedCommandTranslation,
        moves: impl IntoIterator<Item = UiDisplayedToAcceptedTranslation>,
    ) -> Result<Self, Denial> {
        moves
            .into_iter()
            .try_fold(base.delta, |sum, step| {
                if step.accepted_on != presentation {
                    return Err(Denial::PresentationBasisMismatch);
                }
                Ok(add(sum, step.delta))
            })
            .map(Self)
    }

    /// The components, for the GPU-upload edge.
    pub(super) const fn components(self) -> [f32; 2] {
        self.0
    }
}

fn offset_from_rest(content: UiMountedCanonicalBox, sampled: [f32; 4]) -> [f64; 2] {
    [
        f64::from(content.x() - sampled[0]),
        f64::from(content.y() - sampled[1]),
    ]
}

/// Both ends snap to the device grid, so a move never lands content between
/// device pixels.
fn snapped_move(from: [f64; 2], to: [f64; 2], scale: UiScrollPresentationDeviceScale) -> [f32; 2] {
    let snap = |value| value - scale.grid_residue(value);
    [
        (snap(from[0]) - snap(to[0])) as f32,
        (snap(from[1]) - snap(to[1])) as f32,
    ]
}

fn add(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] + b[0], a[1] + b[1]]
}
