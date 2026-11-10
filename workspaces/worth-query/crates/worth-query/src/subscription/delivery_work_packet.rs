use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::active_budget::ActiveSubscriptionAllocationPosture;
use super::active_counters::ActiveSubscriptionCounters;
use super::active_digest::ActiveSubscriptionLaneDigest;
use super::attachment_digest::SubscriptionConsumerAttachmentDigest;
use super::delivery_density::ActiveDeliveryDensityPosture;
use super::delivery_dimensions::{
    ActiveDeliveryAffectedAttachmentWidth, ActiveDeliveryAffectedLaneWidth,
    ActiveDeliveryContinuationWidth, ActiveDeliveryPreviewResidueWidth, PatchGroupWidth,
};
use super::delivery_error::{QueryDeliveryDenialKind, QueryDeliveryError};
use super::evidence_identities::{lifecycle_work_packet_identity, typed_identity_drift};
use super::maintenance_delta::{
    QueryMaintenanceDeltaLoweringReport, QuerySubscriptionMaintenanceDelta,
};
use super::performance_receipt::SubscriptionPerformanceReceipt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveDeliveryWorkPacket {
    active_lane_digest: ActiveSubscriptionLaneDigest,
    attachment_digest: SubscriptionConsumerAttachmentDigest,
    maintenance_delta: QuerySubscriptionMaintenanceDelta,
    lowering_report: QueryMaintenanceDeltaLoweringReport,
    density_posture: ActiveDeliveryDensityPosture,
    affected_lane_width: u64,
    affected_attachment_width: u64,
    patch_group_width: u64,
    continuation_width: u64,
    preview_residue_width: u64,
    allocation_scope_width: u64,
    allocation_posture: ActiveSubscriptionAllocationPosture,
    performance_receipt: SubscriptionPerformanceReceipt,
    work_packet_identity: WorthQueryEvidenceIdentity,
    counters: ActiveSubscriptionCounters,
}

pub(super) struct ActiveDeliveryWorkPacketRequest {
    pub(super) active_lane_digest: ActiveSubscriptionLaneDigest,
    pub(super) attachment_digest: SubscriptionConsumerAttachmentDigest,
    pub(super) maintenance_delta: QuerySubscriptionMaintenanceDelta,
    pub(super) lowering_report: QueryMaintenanceDeltaLoweringReport,
    pub(super) density_posture: ActiveDeliveryDensityPosture,
    pub(super) affected_lane_width: ActiveDeliveryAffectedLaneWidth,
    pub(super) affected_attachment_width: ActiveDeliveryAffectedAttachmentWidth,
    pub(super) patch_group_width: PatchGroupWidth,
    pub(super) continuation_width: ActiveDeliveryContinuationWidth,
    pub(super) preview_residue_width: ActiveDeliveryPreviewResidueWidth,
    pub(super) allocation_scope_width: super::active_dimensions::ActiveAllocationScopeWidth,
    pub(super) allocation_posture: ActiveSubscriptionAllocationPosture,
}

struct ActiveDeliveryWorkWidths {
    affected_lane: u64,
    affected_attachment: u64,
    patch_group: u64,
    continuation: u64,
    preview_residue: u64,
    allocation_scope: u64,
}

impl ActiveDeliveryWorkWidths {
    fn from_request(request: &ActiveDeliveryWorkPacketRequest) -> Self {
        Self {
            affected_lane: request.affected_lane_width.get(),
            affected_attachment: request.affected_attachment_width.get(),
            patch_group: request.patch_group_width.get(),
            continuation: request.continuation_width.get(),
            preview_residue: request.preview_residue_width.get(),
            allocation_scope: request.allocation_scope_width.get(),
        }
    }

    fn consumed(&self) -> u64 {
        self.affected_lane
            + self.affected_attachment
            + self.patch_group
            + self.continuation
            + self.preview_residue
            + self.allocation_scope
    }
}

impl ActiveDeliveryWorkPacket {
    pub(super) fn new(
        request: ActiveDeliveryWorkPacketRequest,
    ) -> Result<Self, QueryDeliveryError> {
        let mut counters = ActiveSubscriptionCounters::default();
        admit_work_packet_request(&request, &mut counters)?;
        let widths = ActiveDeliveryWorkWidths::from_request(&request);
        let consumed_width = widths.consumed();
        let budgeted_width = consumed_width;
        let performance_receipt = SubscriptionPerformanceReceipt::new(
            consumed_width,
            budgeted_width,
            request.density_posture,
            request.allocation_posture,
            request.maintenance_delta.evidence_identity(),
        );
        let work_packet_identity = work_packet_identity(&request, &widths, &performance_receipt)
            .map_err(|cause| {
                QueryDeliveryError::identity_refused(
                    cause,
                    request.maintenance_delta.evidence_identity().clone(),
                    counters.clone(),
                )
            })?;
        retain_work_packet_counters(
            &mut counters,
            &request,
            consumed_width,
            &performance_receipt,
        );
        let ActiveDeliveryWorkPacketRequest {
            active_lane_digest,
            attachment_digest,
            maintenance_delta,
            lowering_report,
            density_posture,
            allocation_posture,
            ..
        } = request;

        Ok(Self {
            active_lane_digest,
            attachment_digest,
            maintenance_delta,
            lowering_report,
            density_posture,
            affected_lane_width: widths.affected_lane,
            affected_attachment_width: widths.affected_attachment,
            patch_group_width: widths.patch_group,
            continuation_width: widths.continuation,
            preview_residue_width: widths.preview_residue,
            allocation_scope_width: widths.allocation_scope,
            allocation_posture,
            performance_receipt,
            work_packet_identity,
            counters,
        })
    }

    pub(crate) fn active_lane_digest(&self) -> &ActiveSubscriptionLaneDigest {
        &self.active_lane_digest
    }

    pub(crate) fn attachment_digest(&self) -> &SubscriptionConsumerAttachmentDigest {
        &self.attachment_digest
    }

    pub fn maintenance_delta(&self) -> &QuerySubscriptionMaintenanceDelta {
        &self.maintenance_delta
    }

    pub fn lowering_report(&self) -> &QueryMaintenanceDeltaLoweringReport {
        &self.lowering_report
    }

    pub fn density_posture(&self) -> ActiveDeliveryDensityPosture {
        self.density_posture
    }

    pub fn patch_group_width(&self) -> u64 {
        self.patch_group_width
    }

    pub fn performance_receipt(&self) -> &SubscriptionPerformanceReceipt {
        &self.performance_receipt
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.work_packet_identity
    }

    pub fn affected_lane_width(&self) -> u64 {
        self.affected_lane_width
    }

    pub fn affected_attachment_width(&self) -> u64 {
        self.affected_attachment_width
    }

    pub fn continuation_width(&self) -> u64 {
        self.continuation_width
    }

    pub fn preview_residue_width(&self) -> u64 {
        self.preview_residue_width
    }

    pub fn allocation_scope_width(&self) -> u64 {
        self.allocation_scope_width
    }

    pub fn allocation_posture(&self) -> ActiveSubscriptionAllocationPosture {
        self.allocation_posture
    }

    pub fn counters(&self) -> &ActiveSubscriptionCounters {
        &self.counters
    }
}

fn admit_work_packet_request(
    request: &ActiveDeliveryWorkPacketRequest,
    counters: &mut ActiveSubscriptionCounters,
) -> Result<(), QueryDeliveryError> {
    if request.allocation_posture.is_heap_denied()
        || !request.allocation_posture.admits_patch_scratch_phase()
    {
        counters.heap_allocation_denial_count = 1;
        return Err(QueryDeliveryError::new(
            QueryDeliveryDenialKind::AllocationPostureForbidden,
            "active delivery work packets require patch-scratch allocation posture",
            request.maintenance_delta.evidence_identity().clone(),
            counters.clone(),
        ));
    }
    if request.density_posture == ActiveDeliveryDensityPosture::DenseRefreshDenied {
        counters.active_delivery_density_dense_denial_count = 1;
        return Err(QueryDeliveryError::new(
            QueryDeliveryDenialKind::DenseRefreshDenied,
            "dense refresh delivery must be explicit debt or typed denial",
            request.maintenance_delta.evidence_identity().clone(),
            counters.clone(),
        ));
    }
    if typed_identity_drift(
        request
            .maintenance_delta
            .active_lane_digest()
            .evidence_identity(),
        request.active_lane_digest.evidence_identity(),
    ) || typed_identity_drift(
        request.lowering_report.maintenance_delta_identity(),
        request.maintenance_delta.evidence_identity(),
    ) {
        counters.delivery_window_overflow_count = 1;
        return Err(QueryDeliveryError::new(
            QueryDeliveryDenialKind::WorkPacketDeltaMismatch,
            "active delivery work packet must consume a lowered delta for the target lane",
            request.maintenance_delta.evidence_identity().clone(),
            counters.clone(),
        ));
    }
    Ok(())
}

fn work_packet_identity(
    request: &ActiveDeliveryWorkPacketRequest,
    widths: &ActiveDeliveryWorkWidths,
    performance_receipt: &SubscriptionPerformanceReceipt,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    lifecycle_work_packet_identity(
        request.active_lane_digest.evidence_identity(),
        request.attachment_digest.evidence_identity(),
        request.maintenance_delta.evidence_identity(),
        request.lowering_report.evidence_identity(),
        request.density_posture.as_str(),
        widths.affected_lane,
        widths.affected_attachment,
        widths.patch_group,
        widths.continuation,
        widths.preview_residue,
        widths.allocation_scope,
        request.allocation_posture,
        performance_receipt.performance_receipt_identity(),
    )
}

fn retain_work_packet_counters(
    counters: &mut ActiveSubscriptionCounters,
    request: &ActiveDeliveryWorkPacketRequest,
    consumed_width: u64,
    performance_receipt: &SubscriptionPerformanceReceipt,
) {
    counters.active_delivery_work_packet_count = 1;
    counters.active_delivery_work_packet_width = consumed_width;
    match request.density_posture {
        ActiveDeliveryDensityPosture::SparseDelta => {
            counters.active_delivery_density_sparse_count = 1
        }
        ActiveDeliveryDensityPosture::BurstCoalesced => {
            counters.active_delivery_density_burst_coalesced_count = 1
        }
        ActiveDeliveryDensityPosture::DenseRefreshDebtExplicit => {
            counters.active_delivery_density_dense_debt_count = 1
        }
        ActiveDeliveryDensityPosture::DenseRefreshDenied => {}
    }
    counters.subscription_performance_receipt_count = 1;
    counters.subscription_budget_consumption_width = consumed_width;
    counters.subscription_budget_remaining_width = performance_receipt.remaining_width();
    if request.allocation_posture.is_heap_debt() {
        counters.heap_allocation_debt_count = 1;
    }
}
