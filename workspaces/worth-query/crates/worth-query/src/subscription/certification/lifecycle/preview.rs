use crate::evidence_identity::WorthQueryEvidenceIdentity;

use super::super::super::active_handle::ActiveSubscriptionLaneHandle;
use super::super::super::attachment::SubscriptionConsumerAttachment;
use super::super::super::closeout::{
    SubscriptionLifecycleCloseout, SubscriptionLifecycleCloseoutKind,
};
use super::super::super::evidence_identities::{
    lifecycle_absent_performance_receipt_identity, lifecycle_absent_preview_isolation_identity,
    lifecycle_absent_preview_residue_identity, lifecycle_labeled_counter_identity,
    lifecycle_preview_promotion_residue_identity, typed_identity_drift,
};
use super::super::super::validation_evidence::{
    validation_role_evidence_identity, validation_shape_role_evidence_identity,
};
use super::error::SubscriptionLifecycleCertificationError;
use super::vocabulary::{
    SubscriptionLifecycleCertificationDenialKind, SubscriptionLifecyclePreviewCertification,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct PreviewCertificationEvidence {
    pub(super) preview_isolation_identity: WorthQueryEvidenceIdentity,
    pub(super) preview_residue_identity: WorthQueryEvidenceIdentity,
    pub(super) counter_identities: Vec<WorthQueryEvidenceIdentity>,
    pub(super) support_identities: Vec<WorthQueryEvidenceIdentity>,
    pub(super) performance_receipt_identity: WorthQueryEvidenceIdentity,
}

pub(super) fn preview_certification_evidence(
    preview: SubscriptionLifecyclePreviewCertification<'_>,
    active_lane_handle: &ActiveSubscriptionLaneHandle,
    attachment: &SubscriptionConsumerAttachment,
    lifecycle_closeout: &SubscriptionLifecycleCloseout,
) -> Result<PreviewCertificationEvidence, SubscriptionLifecycleCertificationError> {
    match preview {
        SubscriptionLifecyclePreviewCertification::None => {
            if matches!(
                lifecycle_closeout.closeout_kind(),
                SubscriptionLifecycleCloseoutKind::PreviewDiscarded
                    | SubscriptionLifecycleCloseoutKind::PreviewPromoted
            ) {
                return Err(SubscriptionLifecycleCertificationError::new(
                    SubscriptionLifecycleCertificationDenialKind::PreviewEvidenceMissing,
                    "preview lifecycle closeout requires explicit preview certification evidence",
                    &[validation_shape_role_evidence_identity(
                        "closeout_kind",
                        lifecycle_closeout.closeout_kind().as_str(),
                    )?],
                ));
            }

            Ok(PreviewCertificationEvidence {
                preview_isolation_identity: lifecycle_absent_preview_isolation_identity(),
                preview_residue_identity: lifecycle_absent_preview_residue_identity(),
                counter_identities: Vec::new(),
                support_identities: vec![WorthQueryEvidenceIdentity::compose(
                    crate::evidence_identity::WorthQueryEvidenceScope::SubscriptionActivationReceipt,
                )
                .field_shape(
                    crate::evidence_identity::WorthQueryEvidenceTag::new("identity_family"),
                    "subscription_preview_support_absent_v1",
                )
                .seal()],
                performance_receipt_identity: lifecycle_absent_performance_receipt_identity(),
            })?
        }
        SubscriptionLifecyclePreviewCertification::Discard {
            isolation,
            residue_report,
            discard_closeout,
        } => {
            if lifecycle_closeout.closeout_kind()
                != &SubscriptionLifecycleCloseoutKind::PreviewDiscarded
                || isolation.active_lane_digest() != active_lane_handle.lane_digest()
                || isolation.attachment_digest() != attachment.attachment_digest()
                || lifecycle_closeout.future_selection() != isolation.future_selection()
                || typed_identity_drift(
                    lifecycle_closeout.basis_binding_identity(),
                    isolation.basis_binding_identity(),
                )
                || typed_identity_drift(
                    lifecycle_closeout.checkpoint_identity(),
                    isolation.checkpoint_identity(),
                )
                || discard_closeout.active_lane_digest() != active_lane_handle.lane_digest()
                || discard_closeout.attachment_digest() != attachment.attachment_digest()
                || discard_closeout.future_selection() != isolation.future_selection()
                || typed_identity_drift(
                    discard_closeout.basis_binding_identity(),
                    isolation.basis_binding_identity(),
                )
                || typed_identity_drift(
                    discard_closeout.checkpoint_identity(),
                    isolation.checkpoint_identity(),
                )
                || typed_identity_drift(
                    discard_closeout.preview_epoch_identity(),
                    isolation.preview_epoch_identity(),
                )
                || discard_closeout.residue_report_identity() != residue_report.report_identity()
                || typed_identity_drift(
                    lifecycle_closeout.source_identity(),
                    discard_closeout.closeout_identity(),
                )
            {
                return Err(SubscriptionLifecycleCertificationError::new(
                    SubscriptionLifecycleCertificationDenialKind::PreviewSourceMismatch,
                    "preview discard certification requires aligned isolation, residue, closeout, and lifecycle closeout evidence",
                    &[
                        validation_shape_role_evidence_identity(
                            "closeout_kind",
                            lifecycle_closeout.closeout_kind().as_str(),
                        )?,
                        validation_role_evidence_identity(
                            "lane",
                            active_lane_handle.lane_digest().evidence_identity(),
                        )?,
                        validation_role_evidence_identity(
                            "attachment",
                            attachment.attachment_digest().evidence_identity(),
                        )?,
                        validation_role_evidence_identity(
                            "preview_lane",
                            isolation.active_lane_digest().evidence_identity(),
                        )?,
                        validation_role_evidence_identity(
                            "preview_attachment",
                            isolation.attachment_digest().evidence_identity(),
                        )?,
                        validation_role_evidence_identity(
                            "discard_attachment",
                            discard_closeout.attachment_digest().evidence_identity(),
                        )?,
                    ],
                ));
            }

            Ok(PreviewCertificationEvidence {
                preview_isolation_identity: isolation.isolation_identity().clone(),
                preview_residue_identity: residue_report.report_identity().clone(),
                counter_identities: vec![
                    lifecycle_labeled_counter_identity(
                        "preview_isolation",
                        &isolation.counters().evidence_identity(),
                    )?,
                    lifecycle_labeled_counter_identity(
                        "preview_discard",
                        &discard_closeout.counters().evidence_identity(),
                    )?,
                ],
                support_identities: vec![
                    WorthQueryEvidenceIdentity::compose(
                        crate::evidence_identity::WorthQueryEvidenceScope::SubscriptionActivationReceipt,
                    )
                    .field_shape(
                        crate::evidence_identity::WorthQueryEvidenceTag::new("identity_family"),
                        "subscription_preview_isolation_support_v1",
                    )
                    .field_evidence_identity(
                        crate::evidence_identity::WorthQueryEvidenceTag::new("isolation"),
                        isolation.isolation_identity(),
                    )
                    .seal(),
                    WorthQueryEvidenceIdentity::compose(
                        crate::evidence_identity::WorthQueryEvidenceScope::SubscriptionActivationReceipt,
                    )
                    .field_shape(
                        crate::evidence_identity::WorthQueryEvidenceTag::new("identity_family"),
                        "subscription_preview_residue_support_v1",
                    )
                    .field_evidence_identity(
                        crate::evidence_identity::WorthQueryEvidenceTag::new("residue"),
                        residue_report.report_identity(),
                    )
                    .seal(),
                    WorthQueryEvidenceIdentity::compose(
                        crate::evidence_identity::WorthQueryEvidenceScope::SubscriptionActivationReceipt,
                    )
                    .field_shape(
                        crate::evidence_identity::WorthQueryEvidenceTag::new("identity_family"),
                        "subscription_preview_discard_support_v1",
                    )
                    .field_evidence_identity(
                        crate::evidence_identity::WorthQueryEvidenceTag::new("discard"),
                        discard_closeout.closeout_identity(),
                    )
                    .seal(),
                ],
                performance_receipt_identity: discard_closeout
                    .performance_receipt()
                    .performance_receipt_identity()
                    .clone(),
            })?
        }
        SubscriptionLifecyclePreviewCertification::Promotion {
            isolation,
            residue_report,
            promotion_handoff,
        } => {
            if lifecycle_closeout.closeout_kind()
                != &SubscriptionLifecycleCloseoutKind::PreviewPromoted
                || isolation.active_lane_digest() != active_lane_handle.lane_digest()
                || isolation.attachment_digest() != attachment.attachment_digest()
                || lifecycle_closeout.future_selection() != isolation.future_selection()
                || typed_identity_drift(
                    lifecycle_closeout.basis_binding_identity(),
                    isolation.basis_binding_identity(),
                )
                || typed_identity_drift(
                    lifecycle_closeout.checkpoint_identity(),
                    isolation.checkpoint_identity(),
                )
                || promotion_handoff.preview_lane_digest() != active_lane_handle.lane_digest()
                || promotion_handoff.attachment_digest() != attachment.attachment_digest()
                || promotion_handoff.future_selection() != isolation.future_selection()
                || typed_identity_drift(
                    promotion_handoff.preview_basis_binding_identity(),
                    isolation.basis_binding_identity(),
                )
                || typed_identity_drift(
                    promotion_handoff.preview_checkpoint_identity(),
                    isolation.checkpoint_identity(),
                )
                || typed_identity_drift(
                    promotion_handoff.preview_epoch_identity(),
                    isolation.preview_epoch_identity(),
                )
                || promotion_handoff.residue_report_identity() != residue_report.report_identity()
                || typed_identity_drift(
                    lifecycle_closeout.source_identity(),
                    promotion_handoff.handoff_identity(),
                )
            {
                return Err(SubscriptionLifecycleCertificationError::new(
                    SubscriptionLifecycleCertificationDenialKind::PreviewSourceMismatch,
                    "preview promotion certification requires aligned isolation, residue, handoff, and lifecycle closeout evidence",
                    &[
                        validation_shape_role_evidence_identity(
                            "closeout_kind",
                            lifecycle_closeout.closeout_kind().as_str(),
                        )?,
                        validation_role_evidence_identity(
                            "lane",
                            active_lane_handle.lane_digest().evidence_identity(),
                        )?,
                        validation_role_evidence_identity(
                            "attachment",
                            attachment.attachment_digest().evidence_identity(),
                        )?,
                        validation_role_evidence_identity(
                            "promotion_lane",
                            promotion_handoff.preview_lane_digest().evidence_identity(),
                        )?,
                    ],
                ));
            }

            Ok(PreviewCertificationEvidence {
                preview_isolation_identity: isolation.isolation_identity().clone(),
                preview_residue_identity: lifecycle_preview_promotion_residue_identity(
                    residue_report.report_identity(),
                    promotion_handoff.handoff_identity(),
                    promotion_handoff
                        .authoritative_active_lane_digest()
                        .evidence_identity(),
                ),
                counter_identities: vec![
                    lifecycle_labeled_counter_identity(
                        "preview_isolation",
                        &isolation.counters().evidence_identity(),
                    )?,
                    lifecycle_labeled_counter_identity(
                        "preview_promotion",
                        &promotion_handoff.counters().evidence_identity(),
                    )?,
                ],
                support_identities: vec![
                    WorthQueryEvidenceIdentity::compose(
                        crate::evidence_identity::WorthQueryEvidenceScope::SubscriptionActivationReceipt,
                    )
                    .field_shape(
                        crate::evidence_identity::WorthQueryEvidenceTag::new("identity_family"),
                        "subscription_preview_isolation_support_v1",
                    )
                    .field_evidence_identity(
                        crate::evidence_identity::WorthQueryEvidenceTag::new("isolation"),
                        isolation.isolation_identity(),
                    )
                    .seal(),
                    WorthQueryEvidenceIdentity::compose(
                        crate::evidence_identity::WorthQueryEvidenceScope::SubscriptionActivationReceipt,
                    )
                    .field_shape(
                        crate::evidence_identity::WorthQueryEvidenceTag::new("identity_family"),
                        "subscription_preview_residue_support_v1",
                    )
                    .field_evidence_identity(
                        crate::evidence_identity::WorthQueryEvidenceTag::new("residue"),
                        residue_report.report_identity(),
                    )
                    .seal(),
                    WorthQueryEvidenceIdentity::compose(
                        crate::evidence_identity::WorthQueryEvidenceScope::SubscriptionActivationReceipt,
                    )
                    .field_shape(
                        crate::evidence_identity::WorthQueryEvidenceTag::new("identity_family"),
                        "subscription_preview_promotion_support_v1",
                    )
                    .field_evidence_identity(
                        crate::evidence_identity::WorthQueryEvidenceTag::new("promotion"),
                        promotion_handoff.handoff_identity(),
                    )
                    .seal(),
                ],
                performance_receipt_identity: promotion_handoff
                    .performance_receipt()
                    .performance_receipt_identity()
                    .clone(),
            })?
        }
    }
}
