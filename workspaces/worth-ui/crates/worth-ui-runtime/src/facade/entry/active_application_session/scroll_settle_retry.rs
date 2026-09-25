//! Scroll settles deferred past the frame that owed them.
//!
//! A deferral keeps the surface a presentation witness committed the settle
//! for, one per semantic surface, so a deferral on one surface never displaces
//! another's debt. A later frame pays each where the host still shows that
//! surface generation. Once it does not, the owed settle has nothing left to
//! land on: it is released, and a chrome capture waiting on it is finished,
//! exactly as when a tick is discarded. The next committed tick settles the
//! new generation against its own witness.

use std::collections::{BTreeMap, BTreeSet};

use super::UiScrollSettleDisposition;
use crate::mounting::presentation::motion_sampling::UiPresentationMotionPresentedSurface;
use worth_ui_host_contract::UiSemanticSurfaceIdentity;

/// The accepted Scroll samples still owed to the displayed pose, keyed by the
/// semantic surface whose witness committed them, and the surfaces whose last
/// settle was refused.
#[derive(Debug)]
pub(in crate::facade::entry) struct UiOwedScrollSettles {
    owed: BTreeMap<UiSemanticSurfaceIdentity, UiPresentationMotionPresentedSurface>,
    refused: BTreeSet<UiSemanticSurfaceIdentity>,
}

impl UiOwedScrollSettles {
    pub(super) const fn none() -> Self {
        Self {
            owed: BTreeMap::new(),
            refused: BTreeSet::new(),
        }
    }

    /// Owe `presented` a frame, replacing only its own surface's older debt.
    pub(super) fn owe(&mut self, presented: UiPresentationMotionPresentedSurface) {
        self.owed.insert(presented.semantic_surface(), presented);
    }

    pub(super) fn paid(&mut self, surface: UiSemanticSurfaceIdentity) {
        self.owed.remove(&surface);
        self.refused.remove(&surface);
    }

    /// The settle `surface` was owed is closed without landing: its
    /// disposition reports the refusal until a later settle is paid there.
    pub(super) fn refused(&mut self, surface: UiSemanticSurfaceIdentity) {
        self.owed.remove(&surface);
        self.refused.insert(surface);
    }

    pub(super) fn owes(&self, surface: UiSemanticSurfaceIdentity) -> bool {
        self.owed.contains_key(&surface)
    }

    pub(super) fn refuses(&self, surface: UiSemanticSurfaceIdentity) -> bool {
        self.refused.contains(&surface)
    }
}

impl super::super::WorthUiActiveApplicationSession {
    /// Whether any deferred accepted-sample settlement is still owed a frame.
    pub(in crate::facade::entry) fn awaits_scroll_settle_retry(&self) -> bool {
        !self.owed_scroll_settles.owed.is_empty()
    }

    /// Pay every settle a deferral still owes, each on the surface generation
    /// its witness committed. `Idle` when nothing is owed; otherwise the
    /// disposition of the last surface settled, each of which is recorded as
    /// the last settle disposition when it lands.
    pub(in crate::facade::entry) fn settle_owed_scroll_samples(
        &mut self,
    ) -> UiScrollSettleDisposition {
        self.settle_owed_scroll_samples_where(|_| true)
    }

    /// Pay what every other surface is owed once `committed` has settled its
    /// own witness, so one surface's Motion never holds another's debt.
    pub(super) fn settle_owed_scroll_samples_beside(
        &mut self,
        committed: UiSemanticSurfaceIdentity,
    ) {
        self.settle_owed_scroll_samples_where(|surface| surface != committed);
    }

    /// Pay what `surface` is owed while the generation its witness committed
    /// is still the one displayed. A rebind calls this before it ends that
    /// generation: the host shows the sample and the surface stays on screen,
    /// so the content stops there. A debt left for later would be released
    /// unpaid, leaving the pose short of what the reader last saw.
    ///
    /// Deregistering a surface, or recovering one whose presentation is
    /// indeterminate, releases the debt unpaid instead: no successor shows
    /// the surface, or no witness proves what the host shows.
    pub(in crate::facade::entry) fn settle_owed_scroll_sample_before_ending(
        &mut self,
        surface: UiSemanticSurfaceIdentity,
    ) {
        self.settle_owed_scroll_samples_where(|owed| owed == surface);
    }

    fn settle_owed_scroll_samples_where(
        &mut self,
        owed_by: impl Fn(UiSemanticSurfaceIdentity) -> bool,
    ) -> UiScrollSettleDisposition {
        let owed = self
            .owed_scroll_settles
            .owed
            .iter()
            .filter(|(surface, _)| owed_by(**surface))
            .map(|(_, presented)| *presented)
            .collect::<Vec<_>>();
        owed.into_iter()
            .fold(UiScrollSettleDisposition::Idle, |_, presented| {
                self.settle_owed_scroll_sample(presented)
            })
    }

    fn settle_owed_scroll_sample(
        &mut self,
        owed: UiPresentationMotionPresentedSurface,
    ) -> UiScrollSettleDisposition {
        let generation_displayed = self
            .mounted
            .current_presentation_for_surface(owed.semantic_surface())
            .is_some_and(|displayed| {
                displayed.binding() == owed.displayed().binding()
                    && displayed.host_surface() == owed.displayed().host_surface()
            });
        if generation_displayed {
            return self.settle_accepted_scroll_sample(owed);
        }
        self.owed_scroll_settles.paid(owed.semantic_surface());
        self.last_scroll_settle_disposition = UiScrollSettleDisposition::Superseded;
        self.finish_pending_scroll_chrome_capture();
        UiScrollSettleDisposition::Superseded
    }
}
