use worth_ui_host_contract::{UiMountedCompletedEffects, UiMountedEffectFamily};

/// Whether a presentation crossed the port and painted; only a painted
/// presentation observed pixels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiNativePaintOutcome {
    Unpainted,
    Painted { pixels: [[u8; 4]; 2] },
}

impl UiNativePaintOutcome {
    pub(crate) const fn painted(self) -> bool {
        matches!(self, Self::Painted { .. })
    }

    pub(crate) const fn pixels(self) -> Option<[[u8; 4]; 2]> {
        match self {
            Self::Painted { pixels } => Some(pixels),
            Self::Unpainted => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct UiNativePresentationEffects {
    native_paint: bool,
    identity_overlay: bool,
}

impl UiNativePresentationEffects {
    pub(crate) const fn new(native_paint: bool, identity_overlay: bool) -> Self {
        Self {
            native_paint,
            identity_overlay,
        }
    }

    pub(crate) fn inherit(&mut self, predecessor: Self) {
        self.native_paint |= predecessor.native_paint;
        self.identity_overlay |= predecessor.identity_overlay;
    }

    pub(crate) const fn without_native_paint(self) -> Self {
        Self::new(false, self.identity_overlay)
    }

    pub(crate) fn completion(self) -> UiMountedCompletedEffects {
        let mut families =
            Vec::with_capacity(usize::from(self.native_paint) + usize::from(self.identity_overlay));
        if self.native_paint {
            families.push(UiMountedEffectFamily::NativePaint);
        }
        if self.identity_overlay {
            families.push(UiMountedEffectFamily::IdentityOverlay);
        }
        UiMountedCompletedEffects::new(families)
    }
}
