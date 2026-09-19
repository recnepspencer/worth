#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedParticipationStatus {
    Admitted,
    Deferred,
    Withheld,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedOmissionReason {
    AwaitingRuntimeMutation,
    NotDefinedByCurrentRuntime,
    NoCommittedAllocation,
    AllocationBoundsUnknown,
    SurfacePolicyWithheld,
    NotProducedByExecutedLane,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum UiMountedMechanicalRole {
    Surface,
    Container,
    Control,
    Diagnostic,
    Portal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiMountedParticipationFact {
    status: UiMountedParticipationStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiMountedParticipation {
    paint: UiMountedParticipationFact,
    clip: UiMountedParticipationFact,
    input: UiMountedParticipationFact,
    focus: UiMountedParticipationFact,
    hit_test: UiMountedParticipationFact,
    accessibility: UiMountedParticipationFact,
    motion: UiMountedParticipationFact,
    diagnostic: UiMountedParticipationFact,
}

pub struct UiMountedParticipationInput {
    pub paint: UiMountedParticipationFact,
    pub clip: UiMountedParticipationFact,
    pub input: UiMountedParticipationFact,
    pub focus: UiMountedParticipationFact,
    pub hit_test: UiMountedParticipationFact,
    pub accessibility: UiMountedParticipationFact,
    pub motion: UiMountedParticipationFact,
    pub diagnostic: UiMountedParticipationFact,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedPaintProjection {
    CountOnlyBatch(super::UiMountedPaintBatchReference),
    Omitted(UiMountedOmissionReason),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedAccessibilityProjection {
    Admitted(UiMountedMechanicalRole),
    Omitted(UiMountedOmissionReason),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedMotionProjection {
    Admitted,
    Omitted(UiMountedOmissionReason),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiMountedDiagnosticReference(u16);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedDiagnosticProjection {
    Reference(UiMountedDiagnosticReference),
    IdentityOverlay(super::UiMountedIdentityOverlayMechanic),
    Omitted(UiMountedOmissionReason),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiMountedProjectionAudience {
    accessibility_disclosed: bool,
    diagnostics_disclosed: bool,
}

impl UiMountedParticipationFact {
    pub const fn new(status: UiMountedParticipationStatus) -> Self {
        Self { status }
    }
    pub fn status(self) -> UiMountedParticipationStatus {
        self.status
    }
}

impl UiMountedParticipation {
    pub fn new(input: UiMountedParticipationInput) -> Self {
        Self {
            paint: input.paint,
            clip: input.clip,
            input: input.input,
            focus: input.focus,
            hit_test: input.hit_test,
            accessibility: input.accessibility,
            motion: input.motion,
            diagnostic: input.diagnostic,
        }
    }
    pub fn paint(self) -> UiMountedParticipationFact {
        self.paint
    }
    pub fn clip(self) -> UiMountedParticipationFact {
        self.clip
    }
    pub fn input(self) -> UiMountedParticipationFact {
        self.input
    }
    pub fn focus(self) -> UiMountedParticipationFact {
        self.focus
    }
    pub fn hit_test(self) -> UiMountedParticipationFact {
        self.hit_test
    }
    pub fn accessibility(self) -> UiMountedParticipationFact {
        self.accessibility
    }
    pub fn motion(self) -> UiMountedParticipationFact {
        self.motion
    }
    pub fn diagnostic(self) -> UiMountedParticipationFact {
        self.diagnostic
    }
}

impl UiMountedDiagnosticReference {
    pub fn new(index: u16) -> Self {
        Self(index)
    }
    pub fn index(self) -> u16 {
        self.0
    }
}

impl UiMountedProjectionAudience {
    pub const fn new(accessibility_disclosed: bool, diagnostics_disclosed: bool) -> Self {
        Self {
            accessibility_disclosed,
            diagnostics_disclosed,
        }
    }
    pub const fn full() -> Self {
        Self::new(true, true)
    }
    pub fn accessibility_disclosed(self) -> bool {
        self.accessibility_disclosed
    }
    pub fn diagnostics_disclosed(self) -> bool {
        self.diagnostics_disclosed
    }
}
