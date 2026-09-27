//! Host viewport evidence and the successor presentation it still owes.
//!
//! Every observation records itself here. A settlement measures only the
//! extent it was prepared from, and only a frame the host accepted after that
//! measurement ends the extent's debt. A frame rejected before effects, or an
//! extent observed after the measurement, leaves a successor presentation owed.

use crate::mounting::{UiMountedFrameOutcome, UiSurfaceBindingGeneration};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct UiNativeViewportBasis {
    pub(super) client_physical_extent: [u32; 2],
    pub(super) scale_factor_milli: u32,
    pub(super) binding: UiSurfaceBindingGeneration,
}

pub(super) struct UiNativeViewportExtent {
    observed: Option<UiNativeViewportBasis>,
    owed: Option<UiNativeViewportBasis>,
    measured: Option<UiNativeViewportBasis>,
}

/// The settlement of one owed extent, prepared from the basis it measures.
pub(super) struct UiNativeViewportSettlement {
    basis: UiNativeViewportBasis,
}

impl UiNativeViewportExtent {
    pub(super) const fn new() -> Self {
        Self {
            observed: None,
            owed: None,
            measured: None,
        }
    }

    pub(super) const fn observed(&self) -> Option<UiNativeViewportBasis> {
        self.observed
    }

    /// The extent still owed a successor presentation.
    pub(super) const fn owed(&self) -> Option<UiNativeViewportBasis> {
        self.owed
    }

    /// Records `basis` as observed. A changed basis is owed a successor
    /// presentation when `owes_successor` admits one.
    pub(super) fn observe(
        &mut self,
        basis: UiNativeViewportBasis,
        owes_successor: impl FnOnce() -> bool,
    ) {
        let changed = self.observed != Some(basis);
        self.observed = Some(basis);
        if changed && owes_successor() {
            self.owed = Some(basis);
        }
    }

    /// The settlement the owed extent still needs. An extent already
    /// measured awaits only its presentation.
    pub(super) fn prepare(&self) -> Option<UiNativeViewportSettlement> {
        self.owed
            .filter(|basis| self.measured != Some(*basis))
            .map(|basis| UiNativeViewportSettlement { basis })
    }

    /// Records that the session now measures `settlement`'s extent.
    pub(super) fn measure(&mut self, settlement: UiNativeViewportSettlement) {
        self.measured = Some(settlement.basis);
    }

    /// Lands a frame outcome. Only a frame the host accepted presents the
    /// measured extent, so only it ends that extent's debt; a newer extent
    /// observed since the measurement stays owed.
    pub(super) fn land_outcome(&mut self, outcome: &UiMountedFrameOutcome) {
        let accepted = match outcome {
            UiMountedFrameOutcome::Published(_)
            | UiMountedFrameOutcome::Unchanged(_)
            | UiMountedFrameOutcome::Reconciled(_) => true,
            UiMountedFrameOutcome::RejectedBeforeEffects(_)
            | UiMountedFrameOutcome::InFlight(_)
            | UiMountedFrameOutcome::PresentationIndeterminate(_)
            | UiMountedFrameOutcome::Superseded(_)
            | UiMountedFrameOutcome::RetentionDenied(_)
            | UiMountedFrameOutcome::AdmissionDenied(_)
            | UiMountedFrameOutcome::CompletionDenied(_) => false,
        };
        if accepted {
            self.land_presented();
        }
    }

    fn land_presented(&mut self) {
        if self.owed.is_some() && self.owed == self.measured {
            self.owed = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{UiNativeViewportBasis, UiNativeViewportExtent};
    use crate::mounting::UiSurfaceBindingGeneration;

    // Which outcomes count as presented is covered through the shell in
    // `viewport_measurement/tests.rs`.
    #[test]
    fn only_a_presented_measurement_ends_the_extent_it_measured() {
        let binding = UiSurfaceBindingGeneration::mint_unbound().expect("binding generation");
        let basis = |width| UiNativeViewportBasis {
            client_physical_extent: [width, 600],
            scale_factor_milli: 1_000,
            binding,
        };
        let mut extent = UiNativeViewportExtent::new();
        extent.observe(basis(800), || true);
        let older = extent.prepare().expect("a changed extent is owed");
        extent.measure(older);
        extent.observe(basis(960), || true);

        // The older extent's frame presents after the newer extent arrived.
        extent.land_presented();
        assert_eq!(extent.owed(), Some(basis(960)));
        let newer = extent.prepare().expect("the newer extent is owed");
        extent.measure(newer);

        // Its frame is rejected before effects: measured, still unpresented.
        assert!(extent.prepare().is_none());
        assert_eq!(extent.owed(), Some(basis(960)));
        extent.land_presented();
        assert_eq!(extent.owed(), None);
        assert_eq!(extent.observed(), Some(basis(960)));
    }
}
