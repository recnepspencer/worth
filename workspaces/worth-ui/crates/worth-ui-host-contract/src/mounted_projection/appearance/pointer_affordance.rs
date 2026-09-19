#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHostPrimaryPointerKind {
    Mouse,
    Pen,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiPointerAffordanceFamily {
    Default,
    Activation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiMountedPointerAffordanceMechanic {
    pointer: crate::UiHostPointerIdentity,
    surface: crate::UiSemanticSurfaceIdentity,
    target: crate::UiMountedInstanceIdentity,
    family: UiPointerAffordanceFamily,
}

impl UiMountedPointerAffordanceMechanic {
    #[doc(hidden)]
    pub const fn complete_from_runtime_mounting(
        pointer: crate::UiHostPointerIdentity,
        surface: crate::UiSemanticSurfaceIdentity,
        target: crate::UiMountedInstanceIdentity,
        family: UiPointerAffordanceFamily,
    ) -> Self {
        Self {
            pointer,
            surface,
            target,
            family,
        }
    }
    pub const fn pointer(self) -> crate::UiHostPointerIdentity {
        self.pointer
    }
    pub const fn surface(self) -> crate::UiSemanticSurfaceIdentity {
        self.surface
    }
    pub const fn target(self) -> crate::UiMountedInstanceIdentity {
        self.target
    }
    pub const fn family(self) -> UiPointerAffordanceFamily {
        self.family
    }
}
