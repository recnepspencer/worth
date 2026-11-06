use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::identity_authority::admit_query_subscription_authority_identity;

use super::super::super::acknowledgement::SubscriptionAcknowledgementFrontier;
use super::super::super::evidence_identities::{
    lifecycle_absent_continuation_identity, lifecycle_absent_performance_receipt_identity,
    lifecycle_active_delivery_density_posture_identity, lifecycle_active_lane_handle_identity,
    lifecycle_active_lane_lookup_class_identity, lifecycle_allocation_posture_identity,
    lifecycle_certification_bundle_identity, lifecycle_counter_sequence_identity,
    lifecycle_labeled_counter_identity, lifecycle_performance_sequence_identity,
    lifecycle_subscription_budget_identity, lifecycle_support_matrix_identity,
};
use super::bundle::SubscriptionLifecycleCertificationBundle;
use super::inputs::LifecycleCertificationInputs;
use super::validation::ValidatedLifecycleSources;

fn lifecycle_counter_identities(
    admission_counters: &WorthQueryEvidenceIdentity,
    active_counters: &WorthQueryEvidenceIdentity,
    frontier: &SubscriptionAcknowledgementFrontier,
    batch_counters: &WorthQueryEvidenceIdentity,
    closeout_counters: &WorthQueryEvidenceIdentity,
    continuation_report_identity: Option<&WorthQueryEvidenceIdentity>,
    preview_counter_identities: &[WorthQueryEvidenceIdentity],
) -> Result<Vec<WorthQueryEvidenceIdentity>, IdentityDerivationDenial> {
    let mut identities = vec![
        lifecycle_labeled_counter_identity("admission", admission_counters),
        lifecycle_labeled_counter_identity("active", active_counters),
        lifecycle_labeled_counter_identity(
            "frontier",
            &WorthQueryEvidenceIdentity::compose(
                crate::evidence_identity::WorthQueryEvidenceScope::SubscriptionActivationReceipt,
            )
            .field_shape(
                crate::evidence_identity::WorthQueryEvidenceTag::new("identity_family"),
                "subscription_acknowledgement_frontier_v1",
            )
            .field_evidence_identity(
                crate::evidence_identity::WorthQueryEvidenceTag::new("attachment"),
                frontier.attachment_digest().evidence_identity(),
            )
            .field_usize(
                crate::evidence_identity::WorthQueryEvidenceTag::new("sequence"),
                frontier.acknowledged_sequence().get() as usize,
            )
            .seal(),
        ),
        lifecycle_labeled_counter_identity("batch", batch_counters),
        lifecycle_labeled_counter_identity("closeout", closeout_counters),
    ];
    if let Some(report_identity) = continuation_report_identity {
        identities.push(lifecycle_labeled_counter_identity(
            "continuation",
            report_identity,
        )?);
    }
    identities.extend(preview_counter_identities.iter().cloned());
    Ok(identities)
}

pub(super) fn assemble_lifecycle_bundle(
    inputs: &LifecycleCertificationInputs<'_>,
    validated: ValidatedLifecycleSources,
) -> Result<
    SubscriptionLifecycleCertificationBundle,
    super::error::SubscriptionLifecycleCertificationError,
> {
    let LifecycleCertificationInputs {
        context,
        admission,
        activation: _,
        scale_report,
        active_admission,
        active_lane_handle,
        attachment,
        delivery_window_identity: _,
        maintenance_delta,
        lowering_report: _,
        work_packet,
        delivery_batch,
        acknowledged_attachment,
        continuation,
        preview: _,
        lifecycle_closeout,
    } = *inputs;
    let ValidatedLifecycleSources {
        base,
        preview_evidence,
    } = validated;
    let active_lane_handle_identity = lifecycle_active_lane_handle_identity(
        active_admission.lane_digest().evidence_identity(),
        active_lane_handle,
    );
    let active_lane_lookup_class_identity =
        lifecycle_active_lane_lookup_class_identity(active_admission.lookup_class().as_str());
    let subscription_budget_identity = lifecycle_subscription_budget_identity(
        active_admission.budget().registry_lookup_width(),
        active_admission.budget().fanout_width(),
        active_admission.budget().allocation_scope_width(),
        active_admission.budget().lookup_class().as_str(),
        active_admission.budget().allocation_posture().as_str(),
        active_admission.budget().durable_checkpoint_requested(),
        active_admission.budget().store_backed_restart_requested(),
    );
    let absent_performance = lifecycle_absent_performance_receipt_identity();
    let preview_performance_identity = preview_evidence.performance_receipt_identity.clone();
    let performance_receipt_identities = [
        active_admission
            .performance_receipt()
            .performance_receipt_identity(),
        attachment
            .performance_receipt()
            .performance_receipt_identity(),
        continuation
            .map(|report| report.performance_receipt().performance_receipt_identity())
            .unwrap_or(&absent_performance),
        work_packet
            .performance_receipt()
            .performance_receipt_identity(),
        lifecycle_closeout
            .performance_receipt()
            .performance_receipt_identity(),
        &preview_performance_identity,
    ];
    let subscription_performance_receipt_identity =
        lifecycle_performance_sequence_identity(performance_receipt_identities);
    let continuation_identity = continuation
        .map(|report| report.evidence_identity().clone())
        .unwrap_or_else(lifecycle_absent_continuation_identity);
    let allocation_posture_identity = lifecycle_allocation_posture_identity(
        work_packet.allocation_posture().as_str(),
        work_packet.allocation_scope_width(),
    );
    let active_delivery_density_posture_identity =
        lifecycle_active_delivery_density_posture_identity(work_packet.density_posture().as_str());
    let counter_identities = lifecycle_counter_identities(
        &admission.counters().evidence_identity(),
        &active_admission.counters().evidence_identity(),
        acknowledged_attachment.acknowledgement_frontier(),
        &delivery_batch.counters().evidence_identity(),
        &lifecycle_closeout.counters().evidence_identity(),
        continuation.map(|report| report.evidence_identity()),
        &preview_evidence.counter_identities,
    );
    let counter_sequence_identity = {
        let refs: Vec<&WorthQueryEvidenceIdentity> = counter_identities.iter().collect();
        lifecycle_counter_sequence_identity(refs)
    };
    let mut support_identities: Vec<&WorthQueryEvidenceIdentity> = vec![
        admission.support_profile().profile_identity(),
        lifecycle_closeout.support_profile().profile_identity(),
        lifecycle_closeout.evidence_identity(),
    ];
    support_identities.extend(preview_evidence.support_identities.iter());
    let support_matrix_identity = lifecycle_support_matrix_identity(support_identities);
    let delivery_window_identity = delivery_batch.delivery_window_identity();
    let certification_bundle_identity = lifecycle_certification_bundle_identity(
        base.certification_bundle_identity(),
        admission.evidence_identity(),
        admission.query_declaration_identity(),
        admission.bridge_declaration_identity(),
        admission.signal_strategy_identity(),
        context.query_scope_identity(),
        context.subscription_family_identity(),
        context.subscription_equivalence_identity(),
        context.policy_identity(),
        context.tenant_basis_identity(),
        context.relationship_proof_identity(),
        context.view_shape_identity(),
        context.basis_posture_identity(),
        active_admission.lane_digest().evidence_identity(),
        &active_lane_handle_identity,
        &subscription_performance_receipt_identity,
        attachment.attachment_digest().evidence_identity(),
        delivery_window_identity,
        maintenance_delta.evidence_identity(),
        work_packet.evidence_identity(),
        delivery_batch.evidence_identity(),
        delivery_batch.receipt().evidence_identity(),
        &continuation_identity,
        lifecycle_closeout.evidence_identity(),
        &support_matrix_identity,
        &counter_sequence_identity,
    );

    Ok(SubscriptionLifecycleCertificationBundle {
        certification_bundle_authority: admit_query_subscription_authority_identity(
            certification_bundle_identity,
        ),
        query_scope_identity: context.query_scope_identity().clone(),
        subscription_family_identity: context.subscription_family_identity().clone(),
        subscription_declaration_identity: admission.query_declaration_identity().clone(),
        subscription_equivalence_identity: context.subscription_equivalence_identity().clone(),
        admission_identity: admission.evidence_identity().clone(),
        active_lane_identity: active_admission.lane_digest().evidence_identity().clone(),
        active_lane_handle_identity,
        active_lane_lookup_class_identity,
        subscription_budget_identity,
        subscription_performance_receipt_identity,
        consumer_attachment_identity: attachment.attachment_digest().evidence_identity().clone(),
        acknowledgement_frontier_identity: acknowledged_attachment
            .acknowledgement_frontier()
            .evidence_identity()
            .clone(),
        delivery_window_identity: delivery_window_identity.clone(),
        maintenance_delta_identity: maintenance_delta.evidence_identity().clone(),
        active_delivery_work_packet_identity: work_packet.evidence_identity().clone(),
        active_delivery_density_posture_identity,
        allocation_posture_identity,
        delivery_batch_identity: delivery_batch.evidence_identity().clone(),
        patch_group_identity: delivery_batch.patch_group().patch_group_identity().clone(),
        delivery_receipt_identity: delivery_batch.receipt().evidence_identity().clone(),
        continuation_identity,
        preview_isolation_identity: preview_evidence.preview_isolation_identity,
        preview_residue_identity: preview_evidence.preview_residue_identity,
        policy_identity: context.policy_identity().clone(),
        tenant_basis_identity: context.tenant_basis_identity().clone(),
        relationship_proof_identity: context.relationship_proof_identity().clone(),
        view_shape_identity: context.view_shape_identity().clone(),
        basis_posture_identity: context.basis_posture_identity().clone(),
        bridge_declaration_identity: admission.bridge_declaration_identity().clone(),
        signal_strategy_identity: admission.signal_strategy_identity().clone(),
        counter_sequence_identity,
        subscription_lifecycle_scale_slope_identity: scale_report.evidence_identity_ref().clone(),
        support_matrix_identity,
    })
}
