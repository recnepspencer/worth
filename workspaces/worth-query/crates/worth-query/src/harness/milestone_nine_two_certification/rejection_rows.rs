use super::active_lifecycle::lifecycle_lane;
use super::rejection_evidence::{
    dense_refresh_rejection, masked_sharing_rejection, preview_residue_rejection,
    preview_sharing_rejection, raw_bridge_rejection, raw_cdc_rejection,
    store_backed_restart_rejection,
};
use super::{
    MilestoneNineTwoPerturbationClass, MilestoneNineTwoRejectionBundle,
    MilestoneNineTwoRejectionRow,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::live::LiveQueryFamily;
use crate::subscription::QuerySubscriptionMaintenanceDeltaKind;

pub(super) fn rejection_rows() -> Result<Vec<MilestoneNineTwoRejectionRow>, IdentityDerivationDenial>
{
    Ok(vec![
        rejection_row(
            "masked-sharing-denies-before-join",
            MilestoneNineTwoPerturbationClass::MaskedSharingDenied,
            masked_sharing_rejection()?,
        )?,
        rejection_row(
            "raw-cdc-delivery-denied-before-batch",
            MilestoneNineTwoPerturbationClass::RawCdcDeliveryDenied,
            raw_cdc_rejection(),
        )?,
        rejection_row(
            "raw-bridge-invalidation-denied-before-batch",
            MilestoneNineTwoPerturbationClass::RawBridgeInvalidationDenied,
            raw_bridge_rejection(),
        )?,
        rejection_row(
            "preview-authoritative-sharing-denied",
            MilestoneNineTwoPerturbationClass::PreviewAuthoritativeSharingDenied,
            preview_sharing_rejection()?,
        )?,
        rejection_row(
            "preview-discard-authoritative-residue-denied",
            MilestoneNineTwoPerturbationClass::PreviewDiscardResidueDenied,
            preview_residue_rejection()?,
        )?,
        rejection_row(
            "dense-refresh-denied-before-work-packet",
            MilestoneNineTwoPerturbationClass::DenseRefreshDenied,
            dense_refresh_rejection()?,
        )?,
        rejection_row(
            "store-backed-restart-denied-before-lane",
            MilestoneNineTwoPerturbationClass::StoreBackedRestartDenied,
            store_backed_restart_rejection()?,
        )?,
    ])
}

fn rejection_row(
    row_name: &'static str,
    perturbation_class: MilestoneNineTwoPerturbationClass,
    hostile_lane: MilestoneNineTwoRejectionBundle,
) -> Result<MilestoneNineTwoRejectionRow, IdentityDerivationDenial> {
    Ok(MilestoneNineTwoRejectionRow {
        row_name,
        perturbation_class,
        control_lane: lifecycle_lane(
            LiveQueryFamily::Detail,
            None,
            QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta,
            "control",
            1,
            0,
        )?,
        hostile_lane,
        parity_lane: lifecycle_lane(
            LiveQueryFamily::Detail,
            None,
            QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta,
            "control",
            1,
            0,
        )?,
    })
}
