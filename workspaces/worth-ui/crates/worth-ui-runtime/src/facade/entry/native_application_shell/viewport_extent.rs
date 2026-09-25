//! Host viewport evidence and the successor presentation it still owes.
//!
//! Every observation records itself here, and a settlement lands only the
//! extent it was prepared from. An extent observed after that settlement was
//! prepared stays owed, so its successor presentation still runs.

use crate::mounting::UiSurfaceBindingGeneration;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct UiNativeViewportBasis {
    pub(super) client_physical_extent: [u32; 2],
    pub(super) scale_factor_milli: u32,
    pub(super) binding: UiSurfaceBindingGeneration,
}

pub(super) struct UiNativeViewportExtent {
    observed: Option<UiNativeViewportBasis>,
    owed: Option<UiNativeViewportBasis>,
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

    pub(super) fn prepare(&self) -> Option<UiNativeViewportSettlement> {
        self.owed.map(|basis| UiNativeViewportSettlement { basis })
    }

    /// Lands `settlement`, ending the extent it was prepared from. A newer
    /// extent observed since then stays owed.
    pub(super) fn land(&mut self, settlement: UiNativeViewportSettlement) {
        if self.owed == Some(settlement.basis) {
            self.owed = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{UiNativeViewportBasis, UiNativeViewportExtent};
    use crate::mounting::UiSurfaceBindingGeneration;

    #[test]
    fn a_settlement_leaves_an_extent_observed_after_it_owed() {
        let binding = UiSurfaceBindingGeneration::mint_unbound().expect("binding generation");
        let basis = |width| UiNativeViewportBasis {
            client_physical_extent: [width, 600],
            scale_factor_milli: 1_000,
            binding,
        };
        let mut extent = UiNativeViewportExtent::new();
        extent.observe(basis(800), || true);
        let older = extent.prepare().expect("a changed extent is owed");
        extent.observe(basis(960), || true);

        extent.land(older);
        assert_eq!(extent.owed(), Some(basis(960)));
        let newer = extent.prepare().expect("the newer extent is owed");
        extent.land(newer);
        assert_eq!(extent.owed(), None);
        assert_eq!(extent.observed(), Some(basis(960)));
    }
}
