//! `UiScrollTransitionTarget`: the semantic offset a Scroll owner is settling
//! toward, bound to the exact owner occurrence and incarnation that produced it.
//!
//! A target is not displayed truth. It may lead the accepted displayed offset,
//! and it is legal only while its bounds still admit it, which is why every
//! bounds reconciliation re-clamps it in the same preparation.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiScrollTransitionTarget {
    owner: super::super::UiScrollOwnerIdentity,
    incarnation: super::super::UiScrollOwnerIncarnation,
    target_offset: super::super::UiScrollOffset,
    horizon: super::UiScrollSettleHorizon,
}

impl UiScrollTransitionTarget {
    pub(crate) const fn new(
        owner: super::super::UiScrollOwnerIdentity,
        incarnation: super::super::UiScrollOwnerIncarnation,
        target_offset: super::super::UiScrollOffset,
        horizon: super::UiScrollSettleHorizon,
    ) -> Self {
        Self {
            owner,
            incarnation,
            target_offset,
            horizon,
        }
    }

    pub(crate) const fn owner(self) -> super::super::UiScrollOwnerIdentity {
        self.owner
    }

    pub(crate) const fn incarnation(self) -> super::super::UiScrollOwnerIncarnation {
        self.incarnation
    }

    pub(crate) const fn target_offset(self) -> super::super::UiScrollOffset {
        self.target_offset
    }

    pub(crate) const fn horizon(self) -> super::UiScrollSettleHorizon {
        self.horizon
    }

    #[cfg(test)]
    pub(crate) const fn settle_deadline_tick(self) -> u64 {
        self.horizon.deadline_tick()
    }

    pub(crate) fn binds(
        self,
        owner: super::super::UiScrollOwnerIdentity,
        incarnation: super::super::UiScrollOwnerIncarnation,
    ) -> bool {
        self.owner == owner && self.incarnation == incarnation
    }

    /// The same transition aimed at a new offset under an extended horizon.
    pub(crate) const fn retargeted(
        self,
        target_offset: super::super::UiScrollOffset,
        horizon: super::UiScrollSettleHorizon,
    ) -> Self {
        Self {
            owner: self.owner,
            incarnation: self.incarnation,
            target_offset,
            horizon,
        }
    }

    /// The same transition with its target pulled back inside reconciled
    /// bounds. A transition may never continue toward an illegal target.
    pub(crate) fn reclamped(self, bounds: super::super::UiScrollBounds) -> Self {
        Self {
            target_offset: bounds.clamp(self.target_offset),
            ..self
        }
    }
}
