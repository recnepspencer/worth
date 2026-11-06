use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use super::ActiveSubscriptionCounters;

impl ActiveSubscriptionCounters {
    pub fn evidence_identity(
        &self,
    ) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        Ok(WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::SubscriptionActivationReceipt,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "active_subscription_counters_v1",
        )
        .field_usize(
            WorthQueryEvidenceTag::new("active_lane_admission"),
            self.active_lane_admission_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("active_lane_registry_lookup"),
            self.active_lane_registry_lookup_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("active_lane_lookup_class"),
            self.active_lane_lookup_class_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("active_lane_linear_scan_debt"),
            self.active_lane_linear_scan_debt_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("active_lane_handle_issue"),
            self.active_lane_handle_issue_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("active_lane_creation"),
            self.active_lane_creation_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("active_lane_join"),
            self.active_lane_join_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("active_lane_join_denial"),
            self.active_lane_join_denial_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("shared_lane"),
            self.shared_lane_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("active_lane_linear_scan_denial"),
            self.active_lane_linear_scan_denial_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("active_lane_allocation_denial"),
            self.active_lane_allocation_denial_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("consumer_attachment"),
            self.consumer_attachment_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("consumer_attachment_denial"),
            self.consumer_attachment_denial_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("fanout_width"),
            self.fanout_width as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("fanout_delivery"),
            self.fanout_delivery_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("affected_consumer_attachment_width"),
            self.affected_consumer_attachment_width as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("acknowledgement_frontier_advance"),
            self.acknowledgement_frontier_advance_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("acknowledgement_receipt_mismatch_denial"),
            self.acknowledgement_receipt_mismatch_denial_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("acknowledgement_sequence_regression_denial"),
            self.acknowledgement_sequence_regression_denial_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("delivery_gap_notice"),
            self.delivery_gap_notice_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("backpressure_denial"),
            self.backpressure_denial_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("delivery_window_open"),
            self.delivery_window_open_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("delivery_window_overflow"),
            self.delivery_window_overflow_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("maintenance_delta_lowering"),
            self.maintenance_delta_lowering_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("maintenance_delta_width"),
            self.maintenance_delta_width as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("active_delivery_work_packet"),
            self.active_delivery_work_packet_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("active_delivery_work_packet_width"),
            self.active_delivery_work_packet_width as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("active_delivery_density_sparse"),
            self.active_delivery_density_sparse_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("active_delivery_density_burst_coalesced"),
            self.active_delivery_density_burst_coalesced_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("active_delivery_density_dense_debt"),
            self.active_delivery_density_dense_debt_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("active_delivery_density_dense_denial"),
            self.active_delivery_density_dense_denial_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("raw_cdc_delivery_denial"),
            self.raw_cdc_delivery_denial_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("raw_bridge_invalidation_denial"),
            self.raw_bridge_invalidation_denial_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("delivery_batch"),
            self.delivery_batch_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("delivery_window_width"),
            self.delivery_window_width as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("patch_group"),
            self.patch_group_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("patch_group_width"),
            self.patch_group_width as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("detail_field_patch_width"),
            self.detail_field_patch_width as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("focused_inspector_patch_width"),
            self.focused_inspector_patch_width as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("collection_membership_patch_width"),
            self.collection_membership_patch_width as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("collection_order_patch_width"),
            self.collection_order_patch_width as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("grouped_membership_patch_width"),
            self.grouped_membership_patch_width as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("bounded_materialization_scope_patch_width"),
            self.bounded_materialization_scope_patch_width as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("continuation_remap"),
            self.continuation_remap_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("continuation_remap_width"),
            self.continuation_remap_width as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("continuation_remap_denial"),
            self.continuation_remap_denial_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("continuation_advisory"),
            self.continuation_advisory_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("continuation_identity_break"),
            self.continuation_identity_break_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("preview_active_lane"),
            self.preview_active_lane_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("preview_authoritative_sharing_denial"),
            self.preview_authoritative_sharing_denial_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("preview_discard_residue_check"),
            self.preview_discard_residue_check_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("preview_residue_width"),
            self.preview_residue_width as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("preview_authoritative_residue"),
            self.preview_authoritative_residue_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("preview_promotion_handoff"),
            self.preview_promotion_handoff_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("consumer_attachment_close"),
            self.consumer_attachment_close_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("active_lane_close"),
            self.active_lane_close_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("subscription_lifecycle_closeout"),
            self.subscription_lifecycle_closeout_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("subscription_lifecycle_closeout_denial"),
            self.subscription_lifecycle_closeout_denial_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("durable_checkpoint_overclaim_denial"),
            self.durable_checkpoint_overclaim_denial_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("store_backed_restart_overclaim_denial"),
            self.store_backed_restart_overclaim_denial_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("subscription_performance_receipt"),
            self.subscription_performance_receipt_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("subscription_budget_consumption_width"),
            self.subscription_budget_consumption_width as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("subscription_budget_remaining_width"),
            self.subscription_budget_remaining_width as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("heap_allocation_debt"),
            self.heap_allocation_debt_count as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("heap_allocation_denial"),
            self.heap_allocation_denial_count as usize,
        )
        .seal()?)
    }
}
