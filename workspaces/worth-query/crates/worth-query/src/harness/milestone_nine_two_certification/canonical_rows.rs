use super::active_lifecycle::{lifecycle_lane, lifecycle_lane_with_posture};
use super::preview_lifecycle::{preview_discard_lane, preview_promotion_lane};
use super::sharing_lifecycle::sharing_lane;
use super::{
    MilestoneNineTwoCertificationRow, MilestoneNineTwoPerturbationClass,
    SubscriptionLifecycleCertificationBundle,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::harness::certification::{HostileExpectation, ParityAnchor};
use crate::live::LiveQueryFamily;
use crate::subscription::QuerySubscriptionMaintenanceDeltaKind;
use crate::view_shape_live::LiveViewShapeFamily;

pub(super) fn canonical_rows(
) -> Result<Vec<MilestoneNineTwoCertificationRow>, IdentityDerivationDenial> {
    Ok(vec![
        row(
            "detail-active-lifecycle-delivery-ack",
            MilestoneNineTwoPerturbationClass::DetailLifecycleDelivery,
            HostileExpectation::EquivalentToControl,
            ParityAnchor::Control,
            lifecycle_lane(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta,
                "manager_id",
                1,
                0,
            )?,
            lifecycle_lane(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta,
                "manager_id",
                1,
                0,
            )?,
            lifecycle_lane(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta,
                "manager_id",
                1,
                0,
            )?,
        ),
        row(
            "equivalent-subscription-sharing-fanout",
            MilestoneNineTwoPerturbationClass::EquivalentSharingFanout,
            HostileExpectation::EquivalentToControl,
            ParityAnchor::Control,
            sharing_lane("consumer-a", "consumer-b")?,
            sharing_lane("consumer-a", "consumer-b")?,
            sharing_lane("consumer-a", "consumer-b")?,
        ),
        row(
            "grouped-membership-query-shaped-delivery",
            MilestoneNineTwoPerturbationClass::GroupedMembershipDelivery,
            HostileExpectation::DistinctFromControl,
            ParityAnchor::Hostile,
            lifecycle_lane(
                LiveQueryFamily::OrderedCollection,
                Some(LiveViewShapeFamily::Table),
                QuerySubscriptionMaintenanceDeltaKind::CollectionMembershipDelta,
                "employee:engineering-to-design",
                2,
                0,
            )?,
            lifecycle_lane(
                LiveQueryFamily::OrderedCollection,
                Some(LiveViewShapeFamily::KanbanGrouped),
                QuerySubscriptionMaintenanceDeltaKind::GroupedMembershipDelta,
                "employee:engineering-to-design",
                2,
                0,
            )?,
            lifecycle_lane(
                LiveQueryFamily::OrderedCollection,
                Some(LiveViewShapeFamily::KanbanGrouped),
                QuerySubscriptionMaintenanceDeltaKind::GroupedMembershipDelta,
                "employee:engineering-to-design",
                2,
                0,
            )?,
        ),
        row(
            "identity-continuation-remap-delivery",
            MilestoneNineTwoPerturbationClass::IdentityContinuationRemap,
            HostileExpectation::DistinctFromControl,
            ParityAnchor::Hostile,
            lifecycle_lane(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta,
                "employee:name",
                1,
                0,
            )?,
            lifecycle_lane(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionMaintenanceDeltaKind::ContinuationDelta,
                "employee:old-to-new",
                1,
                1,
            )?,
            lifecycle_lane(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionMaintenanceDeltaKind::ContinuationDelta,
                "employee:old-to-new",
                1,
                1,
            )?,
        ),
        row(
            "preview-discard-zero-authoritative-residue",
            MilestoneNineTwoPerturbationClass::PreviewDiscardIsolation,
            HostileExpectation::EquivalentToControl,
            ParityAnchor::Control,
            preview_discard_lane()?,
            preview_discard_lane()?,
            preview_discard_lane()?,
        ),
        row(
            "preview-promotion-boundary-handoff",
            MilestoneNineTwoPerturbationClass::PreviewPromotionBoundary,
            HostileExpectation::DistinctFromControl,
            ParityAnchor::Hostile,
            preview_discard_lane()?,
            preview_promotion_lane()?,
            preview_promotion_lane()?,
        ),
        row(
            "performance-receipt-posture-sensitive",
            MilestoneNineTwoPerturbationClass::PerformanceReceiptPostureSensitive,
            HostileExpectation::DistinctFromControl,
            ParityAnchor::Hostile,
            lifecycle_lane(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta,
                "manager_id",
                1,
                0,
            )?,
            lifecycle_lane_with_posture()?,
            lifecycle_lane_with_posture()?,
        ),
        row(
            "scale-slope-width-bounded-lifecycle",
            MilestoneNineTwoPerturbationClass::ScaleSlopeWidthBounded,
            HostileExpectation::DistinctFromControl,
            ParityAnchor::Hostile,
            lifecycle_lane(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta,
                "manager_id",
                1,
                0,
            )?,
            lifecycle_lane(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta,
                "manager_id",
                3,
                2,
            )?,
            lifecycle_lane(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta,
                "manager_id",
                3,
                2,
            )?,
        ),
    ])
}

fn row(
    row_name: &'static str,
    perturbation_class: MilestoneNineTwoPerturbationClass,
    hostile_expectation: HostileExpectation,
    parity_anchor: ParityAnchor,
    control_lane: SubscriptionLifecycleCertificationBundle,
    hostile_lane: SubscriptionLifecycleCertificationBundle,
    parity_lane: SubscriptionLifecycleCertificationBundle,
) -> MilestoneNineTwoCertificationRow {
    MilestoneNineTwoCertificationRow {
        row_name,
        perturbation_class,
        hostile_expectation,
        parity_anchor,
        control_lane,
        hostile_lane,
        parity_lane,
    }
}
