use super::{
    SubscriptionLifecycleCertificationBundle, MILESTONE_NINE_TWO_REQUIRED_COMPILE_FAIL_TARGETS,
};
use crate::harness::certification::digest_parts;

pub(super) fn shipped_bundle(
    bundle: crate::subscription::SubscriptionLifecycleCertificationBundle,
) -> SubscriptionLifecycleCertificationBundle {
    SubscriptionLifecycleCertificationBundle {
        query_digest: bundle.query_scope_projection().label().to_string(),
        subscription_family_digest: bundle.subscription_family_projection().label().to_string(),
        subscription_declaration_digest: bundle
            .subscription_declaration_projection()
            .label()
            .to_string(),
        subscription_equivalence_digest: bundle
            .subscription_equivalence_projection()
            .label()
            .to_string(),
        active_lane_digest: bundle.active_lane_projection().label().to_string(),
        active_lane_handle_digest: bundle.active_lane_handle_projection().label().to_string(),
        active_lane_lookup_class_digest: bundle
            .active_lane_lookup_class_projection()
            .label()
            .to_string(),
        subscription_budget_digest: bundle.subscription_budget_projection().label().to_string(),
        subscription_performance_receipt_digest: bundle
            .subscription_performance_receipt_projection()
            .label()
            .to_string(),
        consumer_attachment_digest: bundle.consumer_attachment_projection().label().to_string(),
        acknowledgement_frontier_digest: bundle
            .acknowledgement_frontier_projection()
            .label()
            .to_string(),
        delivery_window_digest: bundle.delivery_window_projection().label().to_string(),
        maintenance_delta_digest: bundle.maintenance_delta_projection().label().to_string(),
        active_delivery_work_packet_digest: bundle
            .active_delivery_work_packet_projection()
            .label()
            .to_string(),
        active_delivery_density_posture_digest: bundle
            .active_delivery_density_posture_projection()
            .label()
            .to_string(),
        allocation_posture_digest: bundle.allocation_posture_projection().label().to_string(),
        delivery_batch_digest: bundle.delivery_batch_projection().label().to_string(),
        patch_group_digest: bundle.patch_group_projection().label().to_string(),
        delivery_receipt_digest: bundle.delivery_receipt_projection().label().to_string(),
        continuation_digest: bundle.continuation_projection().label().to_string(),
        preview_isolation_digest: bundle.preview_isolation_projection().label().to_string(),
        preview_residue_digest: bundle.preview_residue_projection().label().to_string(),
        policy_digest: bundle.policy_projection().label().to_string(),
        tenant_basis_digest: bundle.tenant_basis_projection().label().to_string(),
        relationship_proof_digest: bundle.relationship_proof_projection().label().to_string(),
        view_shape_digest: bundle.view_shape_projection().label().to_string(),
        basis_digest: bundle.basis_posture_projection().label().to_string(),
        bridge_declaration_digest: bundle.bridge_declaration_projection().label().to_string(),
        signal_strategy_digest: bundle.signal_strategy_projection().label().to_string(),
        failure_digest: "none".to_string(),
        lifecycle_denial_digest: "none".to_string(),
        counter_snapshot: bundle.counter_snapshot_projection().label().to_string(),
        counter_evidence: Vec::new(),
        subscription_lifecycle_scale_slope_digest: bundle
            .subscription_lifecycle_scale_slope_projection()
            .label()
            .to_string(),
        compile_fail_boundary_digest: compile_fail_boundary_digest(),
        support_matrix_digest: bundle.support_matrix_projection().label().to_string(),
    }
}

fn compile_fail_boundary_digest() -> String {
    let mut parts = MILESTONE_NINE_TWO_REQUIRED_COMPILE_FAIL_TARGETS
        .iter()
        .flat_map(|target| {
            [
                format!("target:{target}"),
                format!(
                    "stderr:{}",
                    target.trim_end_matches(".rs").to_string() + ".stderr"
                ),
            ]
        })
        .collect::<Vec<_>>();
    parts.sort();
    digest_parts(&parts)
}
