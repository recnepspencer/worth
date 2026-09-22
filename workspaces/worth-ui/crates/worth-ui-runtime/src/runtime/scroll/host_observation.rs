#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum UiHostScrollObservationOutcome {
    /// The input was admitted and its direct frame or smooth transition staged.
    /// This route receipt is not proof of physical presentation acceptance.
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
    /// A layout or an earlier direct effect must be published before a
    /// different policy can take ownership of its candidate geometry.
    PendingGeometryPublication,
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
    /// A gesture already holds a latch, and the frame this event is reported
    /// against no longer admits input to the occurrence that latch names --
    /// a modal Portal has been accepted above it, or the occurrence has left
    /// the presented frame. Nothing scrolled, and the latch was neither taken
    /// nor released. Carrying on would scroll content the reader can no longer
    /// reach, and falling back to the pointer would hand the rest of one
    /// physical gesture to whatever the modal put under it, which is the
    /// handover the latch exists to prevent.
    LatchedOwnerNotAdmitted(crate::mounting::UiPresentedFrameBasisDenial),
    /// Mounted geometry refused the displayed pose the routed offset named.
    /// The route was not committed: the accepted offset stays where the last
    /// applied pose left it, so state and pixels keep describing one frame.
    Geometry(crate::mounting::UiMountedOccurrenceGeometryDenial),
    /// The host reported this wheel in lines, and the owner the gesture
    /// latched to -- the one that will move -- declares no line extent. A line
    /// is a distance only the author can state, so there is no travel to
    /// route: a notch whose owner never said how tall its lines are moves
    /// nothing rather than moving some number the runtime picked.
    OwnerDeclaresNoLineExtent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiScrollBoundsResolutionDenial {
    AllocationUnavailable,
    ViewportUnavailable,
    OutOfRange,
}
