//! The Scroll transition succession: the sole writer of transition targets.
//!
//! One target per Scroll owner, replaced in place, so a wheel burst cannot grow
//! storage with the number of events it delivered. Accumulation runs against
//! the owner's *current* target rather than an obsolete animation endpoint, so
//! a second notch in the same direction adds a full notch of travel. Input in
//! the opposite direction takes control instead: that axis restarts from the
//! accepted displayed sample, so a reversal is felt on the next eligible frame
//! rather than being absorbed by a target far ahead of the content.

use std::collections::BTreeMap;

/// The accepted owner geometry a transition is computed against: the displayed
/// offset Scroll has accepted, the bounds currently reconciled for the owner,
/// and the owner's axis policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiScrollTransitionBasis {
    accepted_offset: super::super::UiScrollOffset,
    bounds: super::super::UiScrollBounds,
    axes: super::super::UiScrollAxes,
}

/// What reconciling bounds did to an owner's pending target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiScrollTransitionReclampOutcome {
    NoTarget,
    Retained(super::UiScrollTransitionTarget),
    Reclamped(super::UiScrollTransitionTarget),
    /// The owner was reincarnated. A target from a retired incarnation is
    /// retired outright; it is never clamped into its replacement.
    RetiredStaleIncarnation,
    /// The reconciled extent leaves nowhere to scroll to. Clamping the target
    /// to the origin would leave a settle standing that has no distance to
    /// travel, so it is retired instead and the motion behind it can end.
    RetiredEmptyExtent,
}

/// Which offset one axis accumulates against for an arriving delta.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UiScrollAccumulationBasis {
    CurrentTarget,
    AcceptedSample,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct UiScrollOwnerTransition {
    target: super::UiScrollTransitionTarget,
    window: super::UiScrollWheelWindow,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct UiScrollTransitionSuccession {
    owners: BTreeMap<super::super::UiScrollOwnerIdentity, UiScrollOwnerTransition>,
}

impl UiScrollTransitionBasis {
    pub(crate) const fn new(
        accepted_offset: super::super::UiScrollOffset,
        bounds: super::super::UiScrollBounds,
        axes: super::super::UiScrollAxes,
    ) -> Self {
        Self {
            accepted_offset,
            bounds,
            axes,
        }
    }
}

impl UiScrollTransitionSuccession {
    pub(crate) const fn new() -> Self {
        Self {
            owners: BTreeMap::new(),
        }
    }

    /// Stage or advance the owner's transition for one admitted wheel event.
    pub(crate) fn accumulate_wheel(
        &mut self,
        owner: super::super::UiScrollOwnerIdentity,
        incarnation: super::super::UiScrollOwnerIncarnation,
        input: super::UiScrollWheelInput,
        basis: UiScrollTransitionBasis,
    ) -> Result<super::UiScrollTransitionTarget, super::UiScrollTransitionDenial> {
        let delta = input.points_subpixels()?;
        let live = self.live_transition(owner, incarnation, input.input_tick());
        let window = live
            .map_or_else(super::UiScrollWheelWindow::closed, |live| live.window)
            .observe(input.phase(), input.input_tick());
        let start = accumulation_start(live.map(|live| live.target), basis, delta);
        let (target_offset, _) = super::super::routing::consume_delta(
            basis.bounds.clamp(start),
            basis.bounds,
            basis.axes,
            delta,
        );
        let horizon = match live {
            Some(live) => live
                .target
                .horizon()
                .extended_to_latest_input(input.input_tick(), input.settle_ticks())?,
            None => super::UiScrollSettleHorizon::from_latest_input(
                input.input_tick(),
                input.settle_ticks(),
            )?,
        };
        let target = live.map_or_else(
            || super::UiScrollTransitionTarget::new(owner, incarnation, target_offset, horizon),
            |live| live.target.retargeted(target_offset, horizon),
        );
        self.owners
            .insert(owner, UiScrollOwnerTransition { target, window });
        Ok(target)
    }

    /// Re-clamp the owner's pending target against freshly reconciled bounds,
    /// or retire it when the owner has been reincarnated or its extent has
    /// collapsed to nothing.
    pub(crate) fn reconcile_bounds(
        &mut self,
        owner: super::super::UiScrollOwnerIdentity,
        incarnation: super::super::UiScrollOwnerIncarnation,
        bounds: super::super::UiScrollBounds,
    ) -> UiScrollTransitionReclampOutcome {
        let Some(stored) = self.owners.get(&owner).copied() else {
            return UiScrollTransitionReclampOutcome::NoTarget;
        };
        if !stored.target.binds(owner, incarnation) {
            self.owners.remove(&owner);
            return UiScrollTransitionReclampOutcome::RetiredStaleIncarnation;
        }
        if bounds.admits_no_travel() {
            self.owners.remove(&owner);
            return UiScrollTransitionReclampOutcome::RetiredEmptyExtent;
        }
        let reclamped = stored.target.reclamped(bounds);
        self.owners.insert(
            owner,
            UiScrollOwnerTransition {
                target: reclamped,
                window: stored.window,
            },
        );
        if reclamped.target_offset() == stored.target.target_offset() {
            UiScrollTransitionReclampOutcome::Retained(reclamped)
        } else {
            UiScrollTransitionReclampOutcome::Reclamped(reclamped)
        }
    }

    /// Retire the owner's transition outright: thumb capture taking direct
    /// control, owner removal, modality loss, or shutdown.
    pub(crate) fn retire(&mut self, owner: super::super::UiScrollOwnerIdentity) -> bool {
        self.owners.remove(&owner).is_some()
    }

    /// Retire every transition whose settle horizon has ended at `tick`.
    pub(crate) fn advance(&mut self, tick: u64) -> usize {
        let retired = self
            .owners
            .iter()
            .filter(|(_, stored)| stored.target.horizon().is_exhausted_at(tick))
            .map(|(owner, _)| *owner)
            .collect::<Vec<_>>();
        for owner in &retired {
            self.owners.remove(owner);
        }
        retired.len()
    }

    #[cfg(test)]
    pub(crate) fn target(
        &self,
        owner: super::super::UiScrollOwnerIdentity,
        incarnation: super::super::UiScrollOwnerIncarnation,
    ) -> Option<super::UiScrollTransitionTarget> {
        self.owners
            .get(&owner)
            .map(|stored| stored.target)
            .filter(|target| target.binds(owner, incarnation))
    }

    /// Whether the owner's coarse-wheel accumulation window is still open, and
    /// the tick of the latest input that reached it.
    #[cfg(test)]
    pub(crate) fn accumulation_window(
        &self,
        owner: super::super::UiScrollOwnerIdentity,
    ) -> Option<super::UiScrollWheelWindow> {
        self.owners.get(&owner).map(|stored| stored.window)
    }

    /// Every owner that currently holds a pending transition, in owner order,
    /// with the target it is settling toward. Succession is the only place
    /// that knows which owners are mid-settle, so anything that has to visit
    /// them -- retiring the ones that have arrived, reporting what is in
    /// flight -- reads them from here rather than keeping a second list.
    pub(crate) fn pending_owners(
        &self,
    ) -> impl Iterator<
        Item = (
            super::super::UiScrollOwnerIdentity,
            super::UiScrollTransitionTarget,
        ),
    > + '_ {
        self.owners
            .iter()
            .map(|(owner, stored)| (*owner, stored.target))
    }

    #[cfg(test)]
    pub(crate) fn pending_count(&self) -> usize {
        self.owners.len()
    }

    pub(crate) fn clear(&mut self) -> usize {
        let released = self.owners.len();
        self.owners.clear();
        released
    }

    fn live_transition(
        &self,
        owner: super::super::UiScrollOwnerIdentity,
        incarnation: super::super::UiScrollOwnerIncarnation,
        tick: u64,
    ) -> Option<UiScrollOwnerTransition> {
        self.owners
            .get(&owner)
            .copied()
            .filter(|stored| stored.target.binds(owner, incarnation))
            .filter(|stored| !stored.target.horizon().is_exhausted_at(tick))
    }
}

/// The offset the arriving delta is added to, chosen per axis.
fn accumulation_start(
    live: Option<super::UiScrollTransitionTarget>,
    basis: UiScrollTransitionBasis,
    delta: super::super::UiScrollDelta,
) -> super::super::UiScrollOffset {
    let Some(target) = live.map(super::UiScrollTransitionTarget::target_offset) else {
        return basis.accepted_offset;
    };
    let axis = |target_axis: i64, accepted_axis: i64, delta_axis: i64| match axis_accumulation_basis(
        target_axis - accepted_axis,
        delta_axis,
    ) {
        UiScrollAccumulationBasis::CurrentTarget => target_axis,
        UiScrollAccumulationBasis::AcceptedSample => accepted_axis,
    };
    super::super::UiScrollOffset::new(
        axis(
            target.inline_subpixels(),
            basis.accepted_offset.inline_subpixels(),
            delta.inline_subpixels(),
        ),
        axis(
            target.block_subpixels(),
            basis.accepted_offset.block_subpixels(),
            delta.block_subpixels(),
        ),
    )
    .expect("an accumulation start chosen from two non-negative offsets stays non-negative")
}

/// Opposite-direction input takes control: it accumulates from the accepted
/// sample rather than from a target the content has not reached yet.
const fn axis_accumulation_basis(pending_travel: i64, delta: i64) -> UiScrollAccumulationBasis {
    if (pending_travel > 0 && delta < 0) || (pending_travel < 0 && delta > 0) {
        UiScrollAccumulationBasis::AcceptedSample
    } else {
        UiScrollAccumulationBasis::CurrentTarget
    }
}
