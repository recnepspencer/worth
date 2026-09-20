#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum UiHostScrollObservationOutcome {
    Applied(super::UiScrollRouteReceipt),
    Denied(UiHostScrollObservationDenial),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiHostScrollObservationDenial {
    Targeting(crate::runtime::interaction::UiInteractionTargetingDenial),
    PresentedSurfaceFallbackIsAmbiguous,
    MountedBasisUnavailable,
    Ownership(super::UiScrollOwnershipResolutionDenial),
    NoDeclaredScrollOwner,
    AllocationUnavailable,
    ViewportUnavailable,
    BoundsOutOfRange,
    DeltaOutOfRange,
    Route(super::UiScrollRouteDenial),
    /// The declared wheel behaviour is smooth, and the settle this notch asked
    /// for could not be staged or published. Nothing scrolled: the route that
    /// preceded it moved no accepted offset by design.
    SettleUnpublished,
    /// A thumb drag holds every axis this delta asked to move. The drag is
    /// placing that offset directly, one pointer position at a time, and a
    /// wheel moving the same offset underneath it would fight the pointer.
    /// Nothing scrolled, and nothing was staged.
    AxisHeldByChromeDrag,
    /// Mounted geometry refused the displayed pose the routed offset named.
    /// The route was not committed: the accepted offset stays where the last
    /// applied pose left it, so state and pixels keep describing one frame.
    Geometry(crate::mounting::UiMountedOccurrenceGeometryDenial),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiScrollBoundsResolutionDenial {
    AllocationUnavailable,
    ViewportUnavailable,
    OutOfRange,
}
