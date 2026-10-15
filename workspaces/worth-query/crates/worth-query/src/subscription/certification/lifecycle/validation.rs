use super::super::super::evidence_identities::typed_identity_drift;
use super::super::super::validation_evidence::validation_role_evidence_identity;
use super::super::activation::{
    certify_query_subscription_activation, QuerySubscriptionCertificationBundle,
};
use super::error::SubscriptionLifecycleCertificationError;
use super::inputs::LifecycleCertificationInputs;
use super::preview::{preview_certification_evidence, PreviewCertificationEvidence};
use super::vocabulary::SubscriptionLifecycleCertificationDenialKind;

pub(super) struct ValidatedLifecycleSources {
    pub(super) base: QuerySubscriptionCertificationBundle,
    pub(super) preview_evidence: PreviewCertificationEvidence,
}

pub(super) fn validate_lifecycle_sources(
    inputs: &LifecycleCertificationInputs<'_>,
) -> Result<ValidatedLifecycleSources, super::error::SubscriptionLifecycleCertificationError> {
    let LifecycleCertificationInputs {
        context: _,
        admission,
        activation,
        scale_report,
        active_admission,
        active_lane_handle,
        attachment,
        delivery_window_identity,
        maintenance_delta,
        lowering_report,
        work_packet,
        delivery_batch,
        acknowledged_attachment,
        continuation,
        preview,
        lifecycle_closeout,
    } = *inputs;

    let base = certify_query_subscription_activation(
        admission.clone(),
        activation.clone(),
        scale_report.clone(),
    )
    .map_err(|error| {
        SubscriptionLifecycleCertificationError::new(
            SubscriptionLifecycleCertificationDenialKind::ActivationAdmissionMismatch,
            "subscription lifecycle certification requires aligned admission, activation, and scale evidence",
            &[validation_role_evidence_identity(
                "activation_certification",
                error.failure_identity(),
            )],
        )
    })?;

    if typed_identity_drift(
        active_admission.activation_identity(),
        activation.evidence_identity(),
    ) || typed_identity_drift(
        active_admission.admission_identity(),
        admission.evidence_identity(),
    ) || typed_identity_drift(
        active_admission.query_declaration_identity(),
        admission.query_declaration_identity(),
    ) || typed_identity_drift(
        active_admission.bridge_declaration_identity(),
        admission.bridge_declaration_identity(),
    ) || typed_identity_drift(
        active_admission.basis_binding_identity(),
        admission.basis_binding_identity(),
    ) || typed_identity_drift(
        active_admission.signal_strategy_identity(),
        admission.signal_strategy_identity(),
    ) {
        return Err(SubscriptionLifecycleCertificationError::new(
            SubscriptionLifecycleCertificationDenialKind::ActiveLaneSourceMismatch,
            "active lane admission must certify the same admitted subscription source",
            &[
                validation_role_evidence_identity(
                    "lane_activation",
                    active_admission.activation_identity(),
                ),
                validation_role_evidence_identity("activation", activation.evidence_identity()),
                validation_role_evidence_identity(
                    "lane_admission",
                    active_admission.admission_identity(),
                ),
                validation_role_evidence_identity("admission", admission.evidence_identity()),
            ],
        ));
    }

    if typed_identity_drift(
        attachment.lane_digest().evidence_identity(),
        active_lane_handle.lane_digest().evidence_identity(),
    ) {
        return Err(SubscriptionLifecycleCertificationError::new(
            SubscriptionLifecycleCertificationDenialKind::AttachmentSourceMismatch,
            "consumer attachment must belong to the certified active lane handle",
            &[
                validation_role_evidence_identity(
                    "attachment_lane",
                    attachment.lane_digest().evidence_identity(),
                ),
                validation_role_evidence_identity(
                    "handle_lane",
                    active_lane_handle.lane_digest().evidence_identity(),
                ),
            ],
        ));
    }

    if typed_identity_drift(
        maintenance_delta.active_lane_digest().evidence_identity(),
        active_lane_handle.lane_digest().evidence_identity(),
    ) || typed_identity_drift(
        lowering_report.maintenance_delta_identity(),
        maintenance_delta.evidence_identity(),
    ) {
        return Err(SubscriptionLifecycleCertificationError::new(
            SubscriptionLifecycleCertificationDenialKind::MaintenanceDeltaSourceMismatch,
            "maintenance delta and lowering report must belong to the certified lane",
            &[
                validation_role_evidence_identity(
                    "delta_lane",
                    maintenance_delta.active_lane_digest().evidence_identity(),
                ),
                validation_role_evidence_identity(
                    "handle_lane",
                    active_lane_handle.lane_digest().evidence_identity(),
                ),
                validation_role_evidence_identity(
                    "lowering_delta",
                    lowering_report.maintenance_delta_identity(),
                ),
                validation_role_evidence_identity("delta", maintenance_delta.evidence_identity()),
            ],
        ));
    }

    if typed_identity_drift(
        work_packet.active_lane_digest().evidence_identity(),
        active_lane_handle.lane_digest().evidence_identity(),
    ) || typed_identity_drift(
        work_packet.attachment_digest().evidence_identity(),
        attachment.attachment_digest().evidence_identity(),
    ) || typed_identity_drift(
        work_packet.maintenance_delta().evidence_identity(),
        maintenance_delta.evidence_identity(),
    ) || typed_identity_drift(
        work_packet.lowering_report().evidence_identity(),
        lowering_report.evidence_identity(),
    ) {
        return Err(SubscriptionLifecycleCertificationError::new(
            SubscriptionLifecycleCertificationDenialKind::WorkPacketSourceMismatch,
            "active delivery work packet must consume the certified lane, attachment, delta, and lowering report",
            &[
                validation_role_evidence_identity(
                    "packet_lane",
                    work_packet.active_lane_digest().evidence_identity(),
                ),
                validation_role_evidence_identity(
                    "handle_lane",
                    active_lane_handle.lane_digest().evidence_identity(),
                ),
                validation_role_evidence_identity(
                    "packet_attachment",
                    work_packet.attachment_digest().evidence_identity(),
                ),
                validation_role_evidence_identity(
                    "attachment",
                    attachment.attachment_digest().evidence_identity(),
                ),
                validation_role_evidence_identity(
                    "packet_delta",
                    work_packet.maintenance_delta().evidence_identity(),
                ),
                validation_role_evidence_identity(
                    "delta",
                    maintenance_delta.evidence_identity(),
                ),
                validation_role_evidence_identity(
                    "packet_lowering",
                    work_packet.lowering_report().evidence_identity(),
                ),
                validation_role_evidence_identity(
                    "lowering",
                    lowering_report.evidence_identity(),
                ),
            ],
        ));
    }

    if typed_identity_drift(
        delivery_batch.delivery_window_identity(),
        delivery_window_identity,
    ) || typed_identity_drift(
        delivery_batch.attachment_digest().evidence_identity(),
        attachment.attachment_digest().evidence_identity(),
    ) || typed_identity_drift(
        delivery_batch
            .receipt()
            .attachment_digest()
            .evidence_identity(),
        attachment.attachment_digest().evidence_identity(),
    ) {
        return Err(SubscriptionLifecycleCertificationError::new(
            SubscriptionLifecycleCertificationDenialKind::DeliveryBatchSourceMismatch,
            "delivery batch and receipt must belong to the certified window and consumer attachment",
            &[
                validation_role_evidence_identity(
                    "batch_window",
                    delivery_batch.delivery_window_identity(),
                ),
                validation_role_evidence_identity("window", delivery_window_identity),
                validation_role_evidence_identity(
                    "batch_attachment",
                    delivery_batch.attachment_digest().evidence_identity(),
                ),
                validation_role_evidence_identity(
                    "attachment",
                    attachment.attachment_digest().evidence_identity(),
                ),
                validation_role_evidence_identity(
                    "receipt_attachment",
                    delivery_batch
                        .receipt()
                        .attachment_digest()
                        .evidence_identity(),
                ),
            ],
        ));
    }

    if typed_identity_drift(
        acknowledged_attachment
            .attachment_digest()
            .evidence_identity(),
        attachment.attachment_digest().evidence_identity(),
    ) {
        return Err(SubscriptionLifecycleCertificationError::new(
            SubscriptionLifecycleCertificationDenialKind::DeliveryBatchSourceMismatch,
            "acknowledged attachment must advance the frontier for the certified consumer attachment",
            &[
                validation_role_evidence_identity(
                    "ack_attachment",
                    acknowledged_attachment
                        .attachment_digest()
                        .evidence_identity(),
                ),
                validation_role_evidence_identity(
                    "attachment",
                    attachment.attachment_digest().evidence_identity(),
                ),
            ],
        ));
    }

    if let Some(report) = continuation
        && typed_identity_drift(
            report.active_lane_digest().evidence_identity(),
            active_lane_handle.lane_digest().evidence_identity(),
        )
    {
        return Err(SubscriptionLifecycleCertificationError::new(
            SubscriptionLifecycleCertificationDenialKind::ContinuationSourceMismatch,
            "continuation report must belong to the certified active lane",
            &[
                validation_role_evidence_identity(
                    "continuation_lane",
                    report.active_lane_digest().evidence_identity(),
                ),
                validation_role_evidence_identity(
                    "handle_lane",
                    active_lane_handle.lane_digest().evidence_identity(),
                ),
            ],
        ));
    }

    let preview_evidence = preview_certification_evidence(
        preview,
        active_lane_handle,
        attachment,
        lifecycle_closeout,
    )?;

    if typed_identity_drift(
        lifecycle_closeout.active_lane_digest().evidence_identity(),
        active_lane_handle.lane_digest().evidence_identity(),
    ) || typed_identity_drift(
        lifecycle_closeout.attachment_digest().evidence_identity(),
        attachment.attachment_digest().evidence_identity(),
    ) {
        return Err(SubscriptionLifecycleCertificationError::new(
            SubscriptionLifecycleCertificationDenialKind::CloseoutSourceMismatch,
            "lifecycle closeout must terminate the certified lane and attachment",
            &[
                validation_role_evidence_identity(
                    "closeout_lane",
                    lifecycle_closeout.active_lane_digest().evidence_identity(),
                ),
                validation_role_evidence_identity(
                    "handle_lane",
                    active_lane_handle.lane_digest().evidence_identity(),
                ),
                validation_role_evidence_identity(
                    "closeout_attachment",
                    lifecycle_closeout.attachment_digest().evidence_identity(),
                ),
                validation_role_evidence_identity(
                    "attachment",
                    attachment.attachment_digest().evidence_identity(),
                ),
            ],
        ));
    }
    Ok(ValidatedLifecycleSources {
        base,
        preview_evidence,
    })
}
