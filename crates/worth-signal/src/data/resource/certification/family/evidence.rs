use super::super::digest::resource_canonical_digest;
use super::catalog::ResourceCertificationFamily;
use super::digest_basis::{
    ResourceBranchRestoreReplayEvidenceBasis, ResourceInflightBoundednessEvidenceBasis,
    ResourceLifecycleParityEvidenceBasis, ResourceRollbackObservationEvidenceBasis,
    ResourceSupersessionEvidenceBasis,
};
use crate::data::error::SignalError;
use crate::data::resource::ResourceBoundaryKind;
use crate::data::resource::ResourceBoundaryPerformanceEnvelope;
use crate::data::resource::ResourceBranchRestoreReport;
use crate::data::resource::ResourceCompletionRollbackReport;
use crate::data::resource::ResourceDiagnosticsSummary;
use crate::data::resource::ResourceObservationBatchReport;
use crate::data::resource::ResourceReplayReconstructionReport;
use crate::data::resource::ResourceRequestAdmissionReport;
use crate::data::resource::ResourceRuntimeSummary;
use crate::data::telemetry::ResourceTelemetry;
use crate::facade::runtime::ObservationBoundaryOutcome;

#[derive(Debug)]
pub(super) struct ResourceCertificationEvidence {
    pub(super) digest: String,
    pub(super) performance: ResourceBoundaryPerformanceEnvelope,
}

impl ResourceCertificationEvidence {
    pub(super) fn lifecycle_parity(
        baseline: &ResourceReplayReconstructionReport,
        equivalent: &ResourceReplayReconstructionReport,
        baseline_diagnostics: &ResourceDiagnosticsSummary,
        equivalent_diagnostics: &ResourceDiagnosticsSummary,
    ) -> Result<Self, SignalError> {
        if baseline.descriptor_digest() != equivalent.descriptor_digest()
            || baseline.lifecycle_digest() != equivalent.lifecycle_digest()
            || baseline.output_continuity_digest() != equivalent.output_continuity_digest()
            || baseline.denied_completion_digest() != equivalent.denied_completion_digest()
            || baseline.retry_lineage_digest() != equivalent.retry_lineage_digest()
            || baseline.in_flight_digest() != equivalent.in_flight_digest()
            || baseline.replay_digest() != equivalent.replay_digest()
            || baseline.retained_history_unavailable_count()
                != equivalent.retained_history_unavailable_count()
            || baseline.denied_completion_unavailable_count()
                != equivalent.denied_completion_unavailable_count()
            || baseline.retry_lineage_unavailable_count()
                != equivalent.retry_lineage_unavailable_count()
            || baseline_diagnostics.provenance_digest()
                != equivalent_diagnostics.provenance_digest()
        {
            return Err(invalid_resource_certification_evidence(
                ResourceCertificationFamily::AsyncResourceLifecycleParity,
                "requires equivalent replay and diagnostics truth across canonical async executions",
            ));
        }
        Ok(Self {
            digest: resource_canonical_digest(&ResourceLifecycleParityEvidenceBasis {
                descriptor_digest: baseline.descriptor_digest(),
                lifecycle_digest: baseline.lifecycle_digest(),
                output_continuity_digest: baseline.output_continuity_digest(),
                denied_completion_digest: baseline.denied_completion_digest(),
                retry_lineage_digest: baseline.retry_lineage_digest(),
                in_flight_digest: baseline.in_flight_digest(),
                replay_digest: baseline.replay_digest(),
                retained_history_unavailable_count: baseline.retained_history_unavailable_count(),
                denied_completion_unavailable_count: baseline.denied_completion_unavailable_count(),
                retry_lineage_unavailable_count: baseline.retry_lineage_unavailable_count(),
                diagnostics_provenance_digest: baseline_diagnostics.provenance_digest(),
                performance: baseline.performance(),
            }),
            performance: baseline.performance(),
        })
    }

    pub(super) fn out_of_order_supersession(
        admission: ResourceRequestAdmissionReport,
    ) -> Result<Self, SignalError> {
        let Some(supersession) = admission.supersession_record() else {
            return Err(invalid_resource_certification_evidence(
                ResourceCertificationFamily::OutOfOrderCompletionSupersession,
                "requires request admission with supersession evidence",
            ));
        };
        let performance = admission.performance();
        Ok(Self {
            digest: resource_canonical_digest(&ResourceSupersessionEvidenceBasis {
                supersession,
                superseded_request: admission.superseded_request(),
                superseded_transition: admission.superseded_transition(),
                performance,
            }),
            performance,
        })
    }

    pub(super) fn rollback_observation(
        rollback: ResourceCompletionRollbackReport,
        observation: ResourceObservationBatchReport,
        control_observation: ResourceObservationBatchReport,
        pre_rollback: &ResourceReplayReconstructionReport,
        post_rollback: &ResourceReplayReconstructionReport,
        diagnostics: &ResourceDiagnosticsSummary,
    ) -> Result<Self, SignalError> {
        if observation.events().is_empty() {
            return Err(invalid_resource_certification_evidence(
                ResourceCertificationFamily::AsyncRollbackObservationEquivalence,
                "requires rollback-suppressed observation evidence",
            ));
        }
        if !observation
            .events()
            .iter()
            .all(|event| event.outcome() == ObservationBoundaryOutcome::RollbackSuppressed)
        {
            return Err(invalid_resource_certification_evidence(
                ResourceCertificationFamily::AsyncRollbackObservationEquivalence,
                "requires only rollback-suppressed observation events",
            ));
        }
        if control_observation.events().is_empty() {
            return Err(invalid_resource_certification_evidence(
                ResourceCertificationFamily::AsyncRollbackObservationEquivalence,
                "requires a delivered control observation packet",
            ));
        }
        if !control_observation
            .events()
            .iter()
            .all(|event| event.outcome() == ObservationBoundaryOutcome::Delivered)
        {
            return Err(invalid_resource_certification_evidence(
                ResourceCertificationFamily::AsyncRollbackObservationEquivalence,
                "requires only delivered events on the no-failure control path",
            ));
        }
        if observation.events().len() != control_observation.events().len()
            || observation
                .events()
                .iter()
                .zip(control_observation.events())
                .any(|(suppressed, delivered)| {
                    suppressed.observer_id() != delivered.observer_id()
                        || suppressed.handle_id() != delivered.handle_id()
                        || suppressed.policy() != delivered.policy()
                        || suppressed.visited() != delivered.visited()
                        || suppressed.recomputed() != delivered.recomputed()
                        || suppressed.meaningful_change() != delivered.meaningful_change()
                        || suppressed.trigger_matched() != delivered.trigger_matched()
                        || suppressed
                            .matched_resource_nodes()
                            .iter()
                            .map(|node| node.node())
                            .collect::<Vec<_>>()
                            != delivered
                                .matched_resource_nodes()
                                .iter()
                                .map(|node| node.node())
                                .collect::<Vec<_>>()
                })
        {
            return Err(invalid_resource_certification_evidence(
                ResourceCertificationFamily::AsyncRollbackObservationEquivalence,
                "requires rollback-suppressed observation to match the no-failure control delivery exactly in packet shape apart from boundary outcome",
            ));
        }
        if pre_rollback.replay_digest() != post_rollback.replay_digest()
            || pre_rollback.lifecycle_digest() != post_rollback.lifecycle_digest()
            || pre_rollback.descriptor_digest() != post_rollback.descriptor_digest()
            || pre_rollback.output_continuity_digest() != post_rollback.output_continuity_digest()
            || pre_rollback.in_flight_digest() != post_rollback.in_flight_digest()
            || pre_rollback.denied_completion_digest() != post_rollback.denied_completion_digest()
            || pre_rollback.retry_lineage_digest() != post_rollback.retry_lineage_digest()
        {
            return Err(invalid_resource_certification_evidence(
                ResourceCertificationFamily::AsyncRollbackObservationEquivalence,
                "requires rollback lane to preserve canonical replay truth",
            ));
        }
        let performance = rollback.performance();
        let rolled_back = rollback.rolled_back_completion();
        Ok(Self {
            digest: resource_canonical_digest(&ResourceRollbackObservationEvidenceBasis {
                subject: rolled_back.subject(),
                observation,
                control_observation,
                pre_rollback_descriptor_digest: pre_rollback.descriptor_digest(),
                pre_rollback_lifecycle_digest: pre_rollback.lifecycle_digest(),
                pre_rollback_output_continuity_digest: pre_rollback.output_continuity_digest(),
                pre_rollback_denied_completion_digest: pre_rollback.denied_completion_digest(),
                pre_rollback_retry_lineage_digest: pre_rollback.retry_lineage_digest(),
                pre_rollback_in_flight_digest: pre_rollback.in_flight_digest(),
                pre_rollback_replay_digest: pre_rollback.replay_digest(),
                post_rollback_descriptor_digest: post_rollback.descriptor_digest(),
                post_rollback_lifecycle_digest: post_rollback.lifecycle_digest(),
                post_rollback_output_continuity_digest: post_rollback.output_continuity_digest(),
                post_rollback_denied_completion_digest: post_rollback.denied_completion_digest(),
                post_rollback_retry_lineage_digest: post_rollback.retry_lineage_digest(),
                post_rollback_in_flight_digest: post_rollback.in_flight_digest(),
                post_rollback_replay_digest: post_rollback.replay_digest(),
                diagnostics_provenance_digest: diagnostics.provenance_digest(),
                performance,
            }),
            performance,
        })
    }

    pub(super) fn branch_restore_replay(
        restore: ResourceBranchRestoreReport,
        replay: &ResourceReplayReconstructionReport,
    ) -> Self {
        Self {
            digest: resource_canonical_digest(&ResourceBranchRestoreReplayEvidenceBasis {
                restore,
                descriptor_digest: replay.descriptor_digest(),
                lifecycle_digest: replay.lifecycle_digest(),
                denied_completion_digest: replay.denied_completion_digest(),
                in_flight_digest: replay.in_flight_digest(),
                replay_digest: replay.replay_digest(),
                replay_performance: replay.performance(),
            }),
            performance: restore.performance(),
        }
    }

    pub(super) fn inflight_boundedness(
        summary: ResourceRuntimeSummary,
        replay: &ResourceReplayReconstructionReport,
        telemetry: ResourceTelemetry,
        performance: ResourceBoundaryPerformanceEnvelope,
    ) -> Result<Self, SignalError> {
        match performance.boundary() {
            ResourceBoundaryKind::RequestAdmission
            | ResourceBoundaryKind::Cancellation
            | ResourceBoundaryKind::TimeoutAdmission
            | ResourceBoundaryKind::RetryAdmission
            | ResourceBoundaryKind::RevalidationAdmission
            | ResourceBoundaryKind::CompletionAdmission
            | ResourceBoundaryKind::CompletionBatchAdmission
            | ResourceBoundaryKind::BranchRestore
            | ResourceBoundaryKind::ReplayReconstruction => {}
            _ => {
                return Err(invalid_resource_certification_evidence(
                    ResourceCertificationFamily::AsyncInflightBoundedness,
                    "requires an in-flight or replay resource boundary performance envelope",
                ));
            }
        }
        if summary.in_flight_request_count() != replay.in_flight_width() as u64 {
            return Err(invalid_resource_certification_evidence(
                ResourceCertificationFamily::AsyncInflightBoundedness,
                "requires runtime summary and replay reconstruction to agree on in-flight width",
            ));
        }
        if telemetry.resource_retry_admission_count == 0
            || telemetry.resource_branch_restore_count == 0
            || telemetry.resource_superseded_completion_denial_count == 0
            || telemetry.resource_duplicate_completion_denial_count == 0
            || telemetry.resource_contradictory_completion_denial_count == 0
            || telemetry.resource_unknown_request_completion_denial_count == 0
        {
            return Err(invalid_resource_certification_evidence(
                ResourceCertificationFamily::AsyncInflightBoundedness,
                "requires hostile async pressure evidence for retry, restore, supersession, duplicate, contradictory, and unknown completion lanes",
            ));
        }
        Ok(Self {
            digest: resource_canonical_digest(&ResourceInflightBoundednessEvidenceBasis {
                summary,
                replay_in_flight_width: replay.in_flight_width(),
                replay_digest: replay.replay_digest().to_string(),
                retry_admission_count: telemetry.resource_retry_admission_count,
                retry_duplicate_denial_count: telemetry
                    .resource_retry_already_scheduled_denial_count,
                branch_restore_count: telemetry.resource_branch_restore_count,
                branch_restore_broad_rebuild_denial_count: telemetry
                    .resource_branch_restore_broad_rebuild_denial_count,
                superseded_completion_denial_count: telemetry
                    .resource_superseded_completion_denial_count,
                duplicate_completion_denial_count: telemetry
                    .resource_duplicate_completion_denial_count,
                contradictory_completion_denial_count: telemetry
                    .resource_contradictory_completion_denial_count,
                unknown_request_completion_denial_count: telemetry
                    .resource_unknown_request_completion_denial_count,
                broad_scan_denial_count: telemetry.resource_broad_scan_denial_count,
                hot_in_flight_lookup_count: telemetry.resource_hot_in_flight_lookup_count,
                performance,
            }),
            performance,
        })
    }
}

pub(super) fn invalid_resource_certification_evidence(
    family: ResourceCertificationFamily,
    reason: &'static str,
) -> SignalError {
    SignalError::invalid_input(format!(
        "invalid resource certification evidence for {family:?}: {reason}"
    ))
}
