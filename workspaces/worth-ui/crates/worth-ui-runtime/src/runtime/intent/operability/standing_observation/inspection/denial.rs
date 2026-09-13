use worth_ui_inspection::{
    UiPointerAffordanceInspectionConfirmationStop as Confirmation,
    UiPointerAffordanceInspectionPresentationDenial as Presentation,
    UiPointerAffordanceInspectionTargetDenial as Target,
    UiPointerAffordanceInspectionTargetingDenial as Targeting,
};

pub(super) fn target(cause: &crate::runtime::intent::payload::UiIntentPayloadStop) -> Target {
    use crate::runtime::intent::payload::UiIntentPayloadStop as Owner;
    match cause {
        Owner::ApplicationGenerationChanged => Target::ApplicationGenerationChanged,
        Owner::PublicationTransitionInFlight => Target::PublicationTransitionInFlight,
        Owner::NoCurrentPublication => Target::NoCurrentPublication,
        Owner::Targeting(cause) => Target::Targeting(targeting(*cause)),
        // Standing target admission does not materialize a payload. Preserve
        // a typed family and owner detail if a future input owner denies one.
        Owner::ProjectionUnavailable { .. }
        | Owner::ProjectionIdentityMismatch { .. }
        | Owner::ProjectionNotCurrent { .. }
        | Owner::ProjectionShapeMismatch { .. }
        | Owner::ProjectionValueMissing { .. }
        | Owner::TextByteBudgetExceeded { .. }
        | Owner::DraftInteractionRequired { .. }
        | Owner::DraftFieldMismatch { .. }
        | Owner::SelectionInteractionRequired { .. }
        | Owner::SelectionProjectionMismatch { .. }
        | Owner::SelectionRevisionChanged { .. }
        | Owner::ApplicationFactUnavailable { .. }
        | Owner::ApplicationFactIdentityChanged { .. }
        | Owner::ApplicationFactGenerationChanged { .. }
        | Owner::ApplicationFactKindMismatch { .. }
        | Owner::PayloadProjection(_) => Target::PayloadInputUnavailable,
    }
}

pub(super) fn presentation(cause: crate::mounting::UiPresentedFrameBasisDenial) -> Presentation {
    use crate::mounting::UiPresentedFrameBasisDenial as Owner;
    match cause {
        Owner::Expired => Presentation::Expired,
        Owner::Unknown => Presentation::Unknown,
        Owner::BindingNotPresented => Presentation::BindingNotPresented,
        Owner::PresentationEpochMismatch => Presentation::PresentationEpochMismatch,
        Owner::PresentationTruthUnavailable => Presentation::PresentationTruthUnavailable,
        Owner::InstanceNotPresented => Presentation::InstanceNotPresented,
        Owner::NodeReceiptMismatch => Presentation::NodeReceiptMismatch,
    }
}

fn targeting(cause: crate::runtime::interaction::UiInteractionTargetingDenial) -> Targeting {
    use crate::runtime::interaction::UiInteractionTargetingDenial as Owner;
    match cause {
        Owner::ExpiredPresentation => Targeting::ExpiredPresentation,
        Owner::UnknownPresentation => Targeting::UnknownPresentation,
        Owner::BindingNotPresented => Targeting::BindingNotPresented,
        Owner::PresentationEpochMismatch => Targeting::PresentationEpochMismatch,
        Owner::PresentationTruthUnavailable => Targeting::PresentationTruthUnavailable,
        Owner::UnsupportedPositionBasis(_) => Targeting::UnsupportedPositionBasis,
        Owner::IncompatibleHitTestCoordinateSpace { .. } => {
            Targeting::IncompatibleHitTestCoordinateSpace
        }
        Owner::NoTarget { .. } => Targeting::NoTarget,
        Owner::AmbiguousHitTestOrder { .. } => Targeting::AmbiguousHitTestOrder,
        Owner::GraphTargetNotPresented => Targeting::GraphTargetNotPresented,
        Owner::SurfaceNoLongerBound => Targeting::SurfaceNoLongerBound,
        Owner::BindingNoLongerCurrent => Targeting::BindingNoLongerCurrent,
        Owner::MountedInstanceNoLongerCurrent => Targeting::MountedInstanceNoLongerCurrent,
        Owner::MountedSurfaceAffinityChanged => Targeting::MountedSurfaceAffinityChanged,
        Owner::HitTestNodeBudgetExceeded => Targeting::HitTestNodeBudgetExceeded,
        Owner::HitTestCandidateBudgetExceeded => Targeting::HitTestCandidateBudgetExceeded,
        Owner::InvalidHitTestPoint => Targeting::InvalidHitTestPoint,
    }
}

pub(super) fn confirmation(
    cause: &crate::runtime::intent::UiIntentConfirmationStopReason,
) -> Confirmation {
    use crate::runtime::intent::UiIntentConfirmationStopReason as Owner;
    match cause {
        Owner::CandidateNotExclusivelyConfirmable => {
            Confirmation::CandidateNotExclusivelyConfirmable
        }
        Owner::MonotonicTimeRequired { .. } => Confirmation::MonotonicTimeRequired,
        Owner::ChallengeExpiryOverflow => Confirmation::ChallengeExpiryOverflow,
        Owner::ChallengeCapacityExceeded { .. } => Confirmation::ChallengeCapacityExceeded,
        Owner::ChallengeIdentityExhausted => Confirmation::ChallengeIdentityExhausted,
        Owner::NoPendingChallenge { .. } => Confirmation::NoPendingChallenge,
        Owner::AmbiguousPendingChallenges { .. } => Confirmation::AmbiguousPendingChallenges,
        Owner::LifecycleCancelled(_) => Confirmation::LifecycleCancelled,
        Owner::AlreadyContinued => Confirmation::AlreadyContinued,
        Owner::AlreadyStopped => Confirmation::AlreadyStopped,
        Owner::MonotonicTimeRegressed { .. } => Confirmation::MonotonicTimeRegressed,
        Owner::Expired { .. } => Confirmation::Expired,
        Owner::ApplicationWorldChanged => Confirmation::ApplicationWorldChanged,
        Owner::ApplicationGenerationChanged => Confirmation::ApplicationGenerationChanged,
        Owner::ConfirmationRouteChanged => Confirmation::ConfirmationRouteChanged,
        Owner::ProductRouteChanged => Confirmation::ProductRouteChanged,
        Owner::ConfirmationNotPresented => Confirmation::ConfirmationNotPresented,
        Owner::ConfirmationPresentationStale => Confirmation::ConfirmationPresentationStale,
        Owner::ConfirmationTargetChanged(_) => Confirmation::ConfirmationTargetChanged,
        Owner::TargetChanged(_) => Confirmation::TargetChanged,
        Owner::PayloadInputChanged => Confirmation::PayloadInputChanged,
        Owner::OperabilityDependencyChanged => Confirmation::OperabilityDependencyChanged,
        Owner::PolicyChanged => Confirmation::PolicyChanged,
        Owner::ConfirmationPolicyChanged => Confirmation::ConfirmationPolicyChanged,
        Owner::OccupancyChanged => Confirmation::OccupancyChanged,
    }
}
