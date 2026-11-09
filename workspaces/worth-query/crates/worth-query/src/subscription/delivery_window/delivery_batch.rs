use super::*;

#[derive(Debug, Eq, PartialEq)]
pub struct QueryDeliveryBatch {
    delivery_batch_identity: WorthQueryEvidenceIdentity,
    delivery_window_identity: WorthQueryEvidenceIdentity,
    attachment_digest: SubscriptionConsumerAttachmentDigest,
    sequence: QueryDeliverySequence,
    delivery_cause: QuerySubscriptionDeliveryCause,
    has_relational_patch: bool,
    patch_group: QueryPatchGroup,
    receipt: QueryDeliveryBatchReceipt,
    counters: ActiveSubscriptionCounters,
}

impl QueryDeliveryBatch {
    pub(in crate::subscription) fn new(
        window: QueryDeliveryWindow,
        work_packet: ActiveDeliveryWorkPacket,
    ) -> Result<Self, QueryDeliveryError> {
        let mut counters = ActiveSubscriptionCounters::default();
        admit_work_packet_to_window(&window, &work_packet, &mut counters)?;

        let delivery_cause = QuerySubscriptionDeliveryCause::relational_patch(
            work_packet.maintenance_delta().evidence_identity(),
        );
        let patch_group = QueryPatchGroup::new(
            QueryPatchGroupKind::from_delta_kind(work_packet.maintenance_delta().kind()),
            work_packet.maintenance_delta().evidence_identity(),
            work_packet.patch_group_width(),
        );
        let receipt = QueryDeliveryBatchReceipt::new(
            window.attachment_digest().clone(),
            window.next_sequence(),
        );
        let delivery_batch_identity = lifecycle_delivery_batch_identity(
            window.evidence_identity(),
            work_packet.evidence_identity(),
            delivery_cause.delivery_cause_identity(),
            delivery_cause.has_relational_patch(),
            patch_group.patch_group_identity(),
            receipt.evidence_identity(),
            work_packet
                .performance_receipt()
                .performance_receipt_identity(),
            work_packet.maintenance_delta().delivery_posture().as_str(),
        ).expect("delivery batch identity contains six sealed tokens, a boolean and a closed density posture");

        counters.delivery_batch_count = 1;
        counters.fanout_delivery_count = 1;
        counters.patch_group_count = 1;
        counters.patch_group_width = patch_group.width();
        retain_patch_group_width(&patch_group, &mut counters);

        Ok(Self {
            delivery_batch_identity,
            delivery_window_identity: window.evidence_identity().clone(),
            attachment_digest: window.attachment_digest().clone(),
            sequence: window.next_sequence(),
            delivery_cause,
            has_relational_patch: true,
            patch_group,
            receipt,
            counters,
        })
    }

    #[cfg(test)]
    pub(in crate::subscription) fn new_time_only(
        window: QueryDeliveryWindow,
        delivery_cause: QuerySubscriptionDeliveryCause,
    ) -> Result<Self, QueryDeliveryError> {
        assert!(
            !delivery_cause.has_relational_patch(),
            "time-only delivery constructor must not consume relational patch causes"
        );
        let patch_group = QueryPatchGroup::new(
            QueryPatchGroupKind::TimeOnlyDeliveryGroup,
            delivery_cause.delivery_cause_identity(),
            0,
        );
        let receipt = QueryDeliveryBatchReceipt::new(
            window.attachment_digest().clone(),
            window.next_sequence(),
        );
        let absent_work_packet = lifecycle_absent_work_packet_identity();
        let absent_performance = lifecycle_absent_performance_receipt_identity();
        let delivery_batch_identity = lifecycle_delivery_batch_identity(
            window.evidence_identity(),
            &absent_work_packet,
            delivery_cause.delivery_cause_identity(),
            false,
            patch_group.patch_group_identity(),
            receipt.evidence_identity(),
            &absent_performance,
            "time_only",
        )
        .expect("time-only batch identity uses sealed tokens and a literal posture");
        let mut counters = ActiveSubscriptionCounters::default();
        counters.delivery_batch_count = 1;
        counters.fanout_delivery_count = 1;
        counters.patch_group_count = 1;

        Ok(Self {
            delivery_batch_identity,
            delivery_window_identity: window.evidence_identity().clone(),
            attachment_digest: window.attachment_digest().clone(),
            sequence: window.next_sequence(),
            delivery_cause,
            has_relational_patch: false,
            patch_group,
            receipt,
            counters,
        })
    }

    #[cfg(test)]
    pub(in crate::subscription) fn new_mixed_cause(
        window: QueryDeliveryWindow,
        delivery_cause: QuerySubscriptionDeliveryCause,
        has_relational_patch: bool,
        patch_group: QueryPatchGroup,
    ) -> Result<Self, QueryDeliveryError> {
        let receipt = QueryDeliveryBatchReceipt::new(
            window.attachment_digest().clone(),
            window.next_sequence(),
        );
        let absent_work_packet = lifecycle_absent_work_packet_identity();
        let absent_performance = lifecycle_absent_performance_receipt_identity();
        let delivery_batch_identity = lifecycle_delivery_batch_identity(
            window.evidence_identity(),
            &absent_work_packet,
            delivery_cause.delivery_cause_identity(),
            has_relational_patch,
            patch_group.patch_group_identity(),
            receipt.evidence_identity(),
            &absent_performance,
            "mixed_cause",
        )
        .expect("mixed-cause batch identity uses sealed tokens and a literal posture");
        let mut counters = ActiveSubscriptionCounters::default();
        counters.delivery_batch_count = 1;
        counters.fanout_delivery_count = 1;
        counters.patch_group_count = 1;
        counters.patch_group_width = patch_group.width();

        Ok(Self {
            delivery_batch_identity,
            delivery_window_identity: window.evidence_identity().clone(),
            attachment_digest: window.attachment_digest().clone(),
            sequence: window.next_sequence(),
            delivery_cause,
            has_relational_patch,
            patch_group,
            receipt,
            counters,
        })
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.delivery_batch_identity
    }

    pub fn delivery_window_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.delivery_window_identity
    }

    pub(crate) fn attachment_digest(&self) -> &SubscriptionConsumerAttachmentDigest {
        &self.attachment_digest
    }

    pub fn sequence(&self) -> QueryDeliverySequence {
        self.sequence
    }

    pub fn delivery_cause(&self) -> &QuerySubscriptionDeliveryCause {
        &self.delivery_cause
    }

    pub fn delivery_cause_kind(&self) -> QuerySubscriptionDeliveryCauseKind {
        self.delivery_cause.kind()
    }

    pub fn has_relational_patch(&self) -> bool {
        self.has_relational_patch
    }

    pub fn patch_group(&self) -> &QueryPatchGroup {
        &self.patch_group
    }

    pub fn receipt(&self) -> &QueryDeliveryBatchReceipt {
        &self.receipt
    }

    pub fn counters(&self) -> &ActiveSubscriptionCounters {
        &self.counters
    }
}

fn admit_work_packet_to_window(
    window: &QueryDeliveryWindow,
    work_packet: &ActiveDeliveryWorkPacket,
    counters: &mut ActiveSubscriptionCounters,
) -> Result<(), QueryDeliveryError> {
    if typed_identity_drift(
        window.active_lane_digest().evidence_identity(),
        work_packet.active_lane_digest().evidence_identity(),
    ) || typed_identity_drift(
        window.attachment_digest().evidence_identity(),
        work_packet.attachment_digest().evidence_identity(),
    ) {
        counters.delivery_window_overflow_count = 1;
        return Err(QueryDeliveryError::new(
            QueryDeliveryDenialKind::WorkPacketWindowMismatch,
            "delivery work packet must target the delivery window lane and attachment",
            work_packet.evidence_identity().clone(),
            counters.clone(),
        ));
    }
    if work_packet.patch_group_width() > window.patch_group_width()
        || work_packet.maintenance_delta().width() > window.maintenance_delta_width()
    {
        counters.delivery_window_overflow_count = 1;
        return Err(QueryDeliveryError::new(
            QueryDeliveryDenialKind::DeliveryWindowBudgetExceeded,
            "delivery work packet exceeds the opened delivery window budget",
            work_packet.evidence_identity().clone(),
            counters.clone(),
        ));
    }
    Ok(())
}

fn retain_patch_group_width(
    patch_group: &QueryPatchGroup,
    counters: &mut ActiveSubscriptionCounters,
) {
    match patch_group.kind() {
        QueryPatchGroupKind::DetailFieldPatchGroup => {
            counters.detail_field_patch_width = patch_group.width()
        }
        QueryPatchGroupKind::InspectorFocusedPatchGroup => {
            counters.focused_inspector_patch_width = patch_group.width()
        }
        QueryPatchGroupKind::CollectionMembershipPatchGroup => {
            counters.collection_membership_patch_width = patch_group.width()
        }
        QueryPatchGroupKind::CollectionOrderPatchGroup => {
            counters.collection_order_patch_width = patch_group.width()
        }
        QueryPatchGroupKind::GroupedMembershipPatchGroup => {
            counters.grouped_membership_patch_width = patch_group.width()
        }
        QueryPatchGroupKind::BoundedMaterializationScopePatchGroup => {
            counters.bounded_materialization_scope_patch_width = patch_group.width()
        }
        QueryPatchGroupKind::ContinuationPatchGroup => {
            counters.continuation_remap_count = patch_group.width()
        }
        QueryPatchGroupKind::DeliveryGapPatchGroup => {
            counters.delivery_gap_notice_count = 1;
        }
        QueryPatchGroupKind::TimeOnlyDeliveryGroup
        | QueryPatchGroupKind::MixedCauseDeliveryGroup => {}
    }
}
