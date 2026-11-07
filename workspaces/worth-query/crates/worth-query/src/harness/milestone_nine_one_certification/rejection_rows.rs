use super::lane_builders::certified_lane;
use super::rejection_evidence::{
    bridge_family_rejection, broken_relationship_proof_rejection, durable_reload_rejection,
    masked_slice_rejection, scale_source_mismatch_rejection, scale_zero_row_rejection,
    view_family_mismatch_rejection,
};
use super::{MilestoneNineOnePerturbationClass, MilestoneNineOneRejectionRow};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::live::LiveQueryFamily;
use crate::subscription::QuerySubscriptionConstructionSource;
use crate::view_shape_live::LiveViewShapeFamily;

pub(super) fn rejection_rows() -> Vec<MilestoneNineOneRejectionRow> {
    vec![
        MilestoneNineOneRejectionRow {
            row_name: "view-family-mismatch-denies-before-declaration",
            perturbation_class: MilestoneNineOnePerturbationClass::ViewFamilyMismatch,
            control_lane: certified_lane(
                LiveQueryFamily::Detail,
                Some(LiveViewShapeFamily::Detail),
                QuerySubscriptionConstructionSource::Direct,
            )?,
            hostile_lane: view_family_mismatch_rejection()?,
            parity_lane: certified_lane(
                LiveQueryFamily::Detail,
                Some(LiveViewShapeFamily::Detail),
                QuerySubscriptionConstructionSource::FacadeLive,
            )?,
        },
        MilestoneNineOneRejectionRow {
            row_name: "bridge-family-unsupported-denies-before-admission",
            perturbation_class: MilestoneNineOnePerturbationClass::BridgeFamilyUnsupported,
            control_lane: certified_lane(
                LiveQueryFamily::OrderedCollection,
                Some(LiveViewShapeFamily::Table),
                QuerySubscriptionConstructionSource::Direct,
            )?,
            hostile_lane: bridge_family_rejection()?,
            parity_lane: certified_lane(
                LiveQueryFamily::OrderedCollection,
                Some(LiveViewShapeFamily::Table),
                QuerySubscriptionConstructionSource::FacadeLive,
            )?,
        },
        MilestoneNineOneRejectionRow {
            row_name: "masked-detail-slice-denies-before-bridge-lowering",
            perturbation_class: MilestoneNineOnePerturbationClass::MaskedDetailSlice,
            control_lane: certified_lane(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionConstructionSource::Direct,
            )?,
            hostile_lane: masked_slice_rejection(LiveQueryFamily::Detail, None)?,
            parity_lane: certified_lane(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionConstructionSource::FacadeLive,
            )?,
        },
        MilestoneNineOneRejectionRow {
            row_name: "masked-table-ordering-denies-before-bridge-lowering",
            perturbation_class: MilestoneNineOnePerturbationClass::MaskedTableOrderingSlice,
            control_lane: certified_lane(
                LiveQueryFamily::OrderedCollection,
                Some(LiveViewShapeFamily::Table),
                QuerySubscriptionConstructionSource::Direct,
            )?,
            hostile_lane: masked_slice_rejection(
                LiveQueryFamily::OrderedCollection,
                Some(LiveViewShapeFamily::Table),
            )?,
            parity_lane: certified_lane(
                LiveQueryFamily::OrderedCollection,
                Some(LiveViewShapeFamily::Table),
                QuerySubscriptionConstructionSource::FacadeLive,
            )?,
        },
        MilestoneNineOneRejectionRow {
            row_name: "masked-grouped-membership-denies-before-bridge-lowering",
            perturbation_class: MilestoneNineOnePerturbationClass::MaskedGroupedMembershipSlice,
            control_lane: certified_lane(
                LiveQueryFamily::OrderedCollection,
                Some(LiveViewShapeFamily::KanbanGrouped),
                QuerySubscriptionConstructionSource::Direct,
            )?,
            hostile_lane: masked_slice_rejection(
                LiveQueryFamily::OrderedCollection,
                Some(LiveViewShapeFamily::KanbanGrouped),
            )?,
            parity_lane: certified_lane(
                LiveQueryFamily::OrderedCollection,
                Some(LiveViewShapeFamily::KanbanGrouped),
                QuerySubscriptionConstructionSource::FacadeLive,
            )?,
        },
        MilestoneNineOneRejectionRow {
            row_name: "broken-relationship-proof-denies-before-bridge-lowering",
            perturbation_class: MilestoneNineOnePerturbationClass::BrokenRelationshipProof,
            control_lane: certified_lane(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionConstructionSource::Direct,
            )?,
            hostile_lane: broken_relationship_proof_rejection()?,
            parity_lane: certified_lane(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionConstructionSource::FacadeLive,
            )?,
        },
        MilestoneNineOneRejectionRow {
            row_name: "durable-reload-overclaim-denies-before-activation",
            perturbation_class: MilestoneNineOnePerturbationClass::DurableReloadOverclaim,
            control_lane: certified_lane(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionConstructionSource::Direct,
            )?,
            hostile_lane: durable_reload_rejection()?,
            parity_lane: certified_lane(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionConstructionSource::FacadeLive,
            )?,
        },
        MilestoneNineOneRejectionRow {
            row_name: "scale-report-source-mismatch-denies-certification",
            perturbation_class: MilestoneNineOnePerturbationClass::ScaleReportSourceMismatch,
            control_lane: certified_lane(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionConstructionSource::Direct,
            )?,
            hostile_lane: scale_source_mismatch_rejection()?,
            parity_lane: certified_lane(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionConstructionSource::FacadeLive,
            )?,
        },
        MilestoneNineOneRejectionRow {
            row_name: "scale-zero-row-baseline-denied",
            perturbation_class: MilestoneNineOnePerturbationClass::ScaleZeroRowBaseline,
            control_lane: certified_lane(
                LiveQueryFamily::OrderedCollection,
                Some(LiveViewShapeFamily::Table),
                QuerySubscriptionConstructionSource::Direct,
            )?,
            hostile_lane: scale_zero_row_rejection()?,
            parity_lane: certified_lane(
                LiveQueryFamily::OrderedCollection,
                Some(LiveViewShapeFamily::Table),
                QuerySubscriptionConstructionSource::FacadeLive,
            )?,
        },
    ]
}
