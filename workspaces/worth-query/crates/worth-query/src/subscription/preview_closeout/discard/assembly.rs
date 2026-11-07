use super::super::super::active_budget::ActiveSubscriptionAllocationPosture;
use super::super::super::active_counters::ActiveSubscriptionCounters;
use super::super::super::delivery_density::ActiveDeliveryDensityPosture;
use super::super::super::evidence_identities::preview_discard_closeout_identity;
use super::super::super::performance_receipt::SubscriptionPerformanceReceipt;
use super::super::super::preview_isolation::{
    PreviewSubscriptionIsolationArtifact, PreviewSubscriptionLifecycleState,
};
use super::super::super::preview_isolation_error::{
    PreviewSubscriptionIsolationDenialKind, PreviewSubscriptionIsolationError,
};
use super::super::super::preview_residue::PreviewSubscriptionResidueReport;
use super::PreviewSubscriptionDiscardCloseout;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

impl PreviewSubscriptionDiscardCloseout {
    pub(super) fn new(
        isolation: PreviewSubscriptionIsolationArtifact,
        residue_report: PreviewSubscriptionResidueReport,
    ) -> Result<Self, IdentityDerivationDenial> {
        let mut counters = ActiveSubscriptionCounters::default();
        counters.preview_discard_residue_check_count = 1;
        counters.preview_residue_width = residue_report.preview_residue_width();
        counters.preview_authoritative_residue_count = residue_report.authoritative_residue_width();
        let performance_receipt = SubscriptionPerformanceReceipt::new(
            residue_report.preview_residue_width(),
            isolation.preview_residue_budget_width(),
            ActiveDeliveryDensityPosture::SparseDelta,
            ActiveSubscriptionAllocationPosture::LifecycleArena,
            isolation.attachment_digest().evidence_identity(),
        );
        counters.subscription_performance_receipt_count = 1;
        counters.subscription_budget_consumption_width = performance_receipt.consumed_width();
        counters.subscription_budget_remaining_width = performance_receipt.remaining_width();
        let closeout_identity = preview_discard_closeout_identity(
            isolation.active_lane_digest().evidence_identity(),
            isolation.attachment_digest().evidence_identity(),
            isolation.future_selection().projection_identity(),
            isolation.basis_binding_identity(),
            isolation.checkpoint_identity(),
            isolation.preview_epoch_identity(),
            isolation.isolation_identity(),
            residue_report.report_identity(),
            performance_receipt.performance_receipt_identity(),
            PreviewSubscriptionLifecycleState::PreviewDiscarded.as_str(),
            &counters.evidence_identity(),
        )?;
        Ok(Self {
            active_lane_digest: isolation.active_lane_digest().clone(),
            attachment_digest: isolation.attachment_digest().clone(),
            future_selection: isolation.future_selection().clone(),
            basis_binding_identity: isolation.basis_binding_identity().clone(),
            checkpoint_identity: isolation.checkpoint_identity().clone(),
            preview_epoch_identity: isolation.preview_epoch_identity().clone(),
            residue_report_identity: residue_report.report_identity().clone(),
            performance_receipt,
            counters,
            closeout_identity,
        })
    }
}

pub fn discard_preview_subscription(
    isolation: PreviewSubscriptionIsolationArtifact,
    residue_report: PreviewSubscriptionResidueReport,
) -> Result<PreviewSubscriptionDiscardCloseout, PreviewSubscriptionIsolationError> {
    if isolation.lifecycle_state() != PreviewSubscriptionLifecycleState::PreviewActive {
        let mut counters = ActiveSubscriptionCounters::default();
        counters.preview_discard_residue_check_count = 1;
        return Err(PreviewSubscriptionIsolationError::new(
            PreviewSubscriptionIsolationDenialKind::PreviewLifecycleStateMismatch,
            "preview discard requires an active preview isolation artifact",
            isolation.isolation_identity().clone(),
            counters,
        ));
    }
    if residue_report.authoritative_residue_width() > 0 {
        let mut counters = ActiveSubscriptionCounters::default();
        counters.preview_discard_residue_check_count = 1;
        counters.preview_residue_width = residue_report.preview_residue_width();
        counters.preview_authoritative_residue_count = residue_report.authoritative_residue_width();
        return Err(PreviewSubscriptionIsolationError::new(
            PreviewSubscriptionIsolationDenialKind::PreviewDiscardResidueDenied,
            "preview discard cannot close while authoritative routing, checkpoint, replay, diagnostics, or writeback residue remains",
            residue_report.report_identity().clone(),
            counters,
        ));
    }
    if residue_report.preview_residue_width() > isolation.preview_residue_budget_width() {
        let mut counters = ActiveSubscriptionCounters::default();
        counters.preview_discard_residue_check_count = 1;
        counters.preview_residue_width = residue_report.preview_residue_width();
        return Err(PreviewSubscriptionIsolationError::new(
            PreviewSubscriptionIsolationDenialKind::PreviewDiscardResidueDenied,
            "preview discard cannot exceed the admitted preview residue budget",
            residue_report.report_identity().clone(),
            counters,
        ));
    }

    Ok(PreviewSubscriptionDiscardCloseout::new(
        isolation,
        residue_report,
    ))
}
