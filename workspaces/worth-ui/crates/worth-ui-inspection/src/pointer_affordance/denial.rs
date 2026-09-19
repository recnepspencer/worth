#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiPointerAffordanceInspectionPresentationDenial {
    Expired,
    Unknown,
    BindingNotPresented,
    PresentationEpochMismatch,
    PresentationTruthUnavailable,
    InstanceNotPresented,
    NodeReceiptMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiPointerAffordanceInspectionTargetDenial {
    ApplicationGenerationChanged,
    PublicationTransitionInFlight,
    NoCurrentPublication,
    Targeting(UiPointerAffordanceInspectionTargetingDenial),
    PayloadInputUnavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiPointerAffordanceInspectionTargetingDenial {
    ExpiredPresentation,
    UnknownPresentation,
    BindingNotPresented,
    PresentationEpochMismatch,
    PresentationTruthUnavailable,
    UnsupportedPositionBasis,
    IncompatibleHitTestCoordinateSpace,
    NoTarget,
    AmbiguousHitTestOrder,
    GraphTargetNotPresented,
    SurfaceNoLongerBound,
    BindingNoLongerCurrent,
    MountedInstanceNoLongerCurrent,
    MountedSurfaceAffinityChanged,
    HitTestNodeBudgetExceeded,
    HitTestCandidateBudgetExceeded,
    InvalidHitTestPoint,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiPointerAffordanceInspectionConfirmationStop {
    CandidateNotExclusivelyConfirmable,
    MonotonicTimeRequired,
    ChallengeExpiryOverflow,
    ChallengeCapacityExceeded,
    ChallengeIdentityExhausted,
    NoPendingChallenge,
    AmbiguousPendingChallenges,
    LifecycleCancelled,
    AlreadyContinued,
    AlreadyStopped,
    MonotonicTimeRegressed,
    Expired,
    ApplicationWorldChanged,
    ApplicationGenerationChanged,
    ConfirmationRouteChanged,
    ProductRouteChanged,
    ConfirmationNotPresented,
    ConfirmationPresentationStale,
    ConfirmationTargetChanged,
    TargetChanged,
    PayloadInputChanged,
    OperabilityDependencyChanged,
    PolicyChanged,
    ConfirmationPolicyChanged,
    OccupancyChanged,
}
