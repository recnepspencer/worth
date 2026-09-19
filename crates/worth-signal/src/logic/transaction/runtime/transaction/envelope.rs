use serde::{Deserialize, Serialize};

use crate::data::node::AuthorityPolicy;
use crate::data::telemetry::RuntimeTelemetry;
use crate::data::temporal::TemporalExecutionSummary;
use crate::diagnostics::epochs::EventEpochSummary;
use crate::diagnostics::failure::ExecutionFailurePhase;
use crate::diagnostics::failure::FailureSummary;
use crate::diagnostics::failure::RollbackDiagnostic;
use crate::logic::planner::model::ParallelAdmissionReason;
use crate::logic::planner::ExecutionReport;
use crate::logic::transaction::runtime::transaction::ObservationBoundarySummary;

use super::transaction_types::{
    EvaluationSummary, TemporalTransactionEvidence, TransactionOutcome, TransactionReplayEntry,
    TransactionResult, TransactionTiming,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdvisoryRecord {
    pub code: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionDetail {
    TransactionOutcome {
        outcome: TransactionOutcome,
    },
    StageAuthorityPolicy {
        authority_policy: AuthorityPolicy,
    },
    StageParallelAdmission {
        admission_reason: ParallelAdmissionReason,
    },
    Rollback {
        reason: String,
    },
    Failure {
        phase: ExecutionFailurePhase,
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionRecord {
    #[serde(default)]
    pub stage_index: Option<u32>,
    pub detail: DecisionDetail,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DecisionLog {
    pub records: Vec<DecisionRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DecisionSummary {
    pub total_records: u32,
    pub stage_authority_decisions: u32,
    pub stage_parallel_decisions: u32,
    pub rollback_recorded: bool,
    pub failure_recorded: bool,
    pub committed: bool,
    pub rolled_back: bool,
    pub poisoned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrityMarkers {
    pub envelope_version: u16,
    pub execution_report_attached: bool,
    pub rollback_attached: bool,
    pub failure_attached: bool,
    pub event_epochs_attached: bool,
}

impl TransactionResult {
    pub(crate) fn from_boundary_state(
        outcome: TransactionOutcome,
        execution_report: Option<ExecutionReport>,
        timing: TransactionTiming,
        touched_nodes: u32,
        evaluation_summary: EvaluationSummary,
        temporal_summary: TemporalExecutionSummary,
        temporal_evidence: TemporalTransactionEvidence,
        _replay_events: &[TransactionReplayEntry],
        reconstructability: crate::logic::transaction::runtime::state::ReconstructabilityRecord,
        event_epochs: Vec<EventEpochSummary>,
        rollback: Option<RollbackDiagnostic>,
        failure_summary: Option<FailureSummary>,
        observation: ObservationBoundarySummary,
        performance_accounting: RuntimeTelemetry,
    ) -> Self {
        let rollback_reason = rollback
            .as_ref()
            .and_then(|diagnostic| diagnostic.reason.clone());
        let execution_report_attached = execution_report.is_some();
        let rollback_attached = rollback_reason.is_some();
        let event_epochs_attached = !event_epochs.is_empty();
        let mut records = Vec::new();
        records.push(DecisionRecord {
            stage_index: None,
            detail: DecisionDetail::TransactionOutcome { outcome },
        });

        if let Some(report) = execution_report.as_ref() {
            for stage in &report.stages {
                if let Some(authority_policy) = stage.authority_policy {
                    records.push(DecisionRecord {
                        stage_index: Some(stage.stage_index),
                        detail: DecisionDetail::StageAuthorityPolicy { authority_policy },
                    });
                }
                if let Some(reason) = &stage.parallel_admission_reason {
                    records.push(DecisionRecord {
                        stage_index: Some(stage.stage_index),
                        detail: DecisionDetail::StageParallelAdmission {
                            admission_reason: *reason,
                        },
                    });
                }
            }
        }

        if let Some(reason) = rollback_reason.as_deref() {
            records.push(DecisionRecord {
                stage_index: None,
                detail: DecisionDetail::Rollback {
                    reason: reason.to_owned(),
                },
            });
        }

        if let Some(failure) = failure_summary.as_ref() {
            records.push(DecisionRecord {
                stage_index: None,
                detail: DecisionDetail::Failure {
                    phase: failure.phase,
                    message: failure.message.clone(),
                },
            });
        }

        let warnings = advisory_records(
            outcome,
            rollback_reason.as_deref(),
            failure_summary.as_ref(),
        );

        let summary = DecisionSummary {
            total_records: records.len() as u32,
            stage_authority_decisions: records
                .iter()
                .filter(|record| {
                    matches!(record.detail, DecisionDetail::StageAuthorityPolicy { .. })
                })
                .count() as u32,
            stage_parallel_decisions: records
                .iter()
                .filter(|record| {
                    matches!(record.detail, DecisionDetail::StageParallelAdmission { .. })
                })
                .count() as u32,
            rollback_recorded: records
                .iter()
                .any(|record| matches!(record.detail, DecisionDetail::Rollback { .. })),
            failure_recorded: records
                .iter()
                .any(|record| matches!(record.detail, DecisionDetail::Failure { .. })),
            committed: matches!(outcome, TransactionOutcome::Committed),
            rolled_back: matches!(outcome, TransactionOutcome::RolledBack),
            poisoned: matches!(outcome, TransactionOutcome::Poisoned),
        };

        Self {
            outcome,
            diagnostic_publication_work: Default::default(),
            execution_report,
            timing,
            touched_nodes,
            evaluation_summary,
            temporal_summary,
            temporal_evidence,
            reconstructability,
            event_epochs: event_epochs.clone(),
            rollback,
            warnings,
            observation,
            decision_summary: summary,
            decision_log: DecisionLog { records },
            integrity_markers: IntegrityMarkers {
                envelope_version: 1,
                execution_report_attached,
                rollback_attached,
                failure_attached: failure_summary.is_some(),
                event_epochs_attached,
            },
            performance_accounting,
        }
    }
}

fn advisory_records(
    outcome: TransactionOutcome,
    rollback_reason: Option<&str>,
    failure_summary: Option<&FailureSummary>,
) -> Vec<AdvisoryRecord> {
    let mut warnings = Vec::new();

    if let Some(reason) = rollback_reason {
        warnings.push(AdvisoryRecord {
            code: "rollback".to_owned(),
            detail: reason.to_owned(),
        });
    }

    if let Some(failure) = failure_summary {
        warnings.push(AdvisoryRecord {
            code: "failure".to_owned(),
            detail: failure.message.clone(),
        });
    }

    if matches!(outcome, TransactionOutcome::Poisoned) && failure_summary.is_none() {
        warnings.push(AdvisoryRecord {
            code: "poisoned".to_owned(),
            detail: "transaction completed in poisoned state".to_owned(),
        });
    }

    warnings
}

#[cfg(test)]
mod tests {
    use crate::data::node::AuthorityPolicy;

    use super::{advisory_records, DecisionDetail, DecisionRecord, TransactionOutcome};

    #[test]
    fn advisories_do_not_depend_on_replay_side_channels() {
        let warnings = advisory_records(
            TransactionOutcome::RolledBack,
            Some("explicit rollback"),
            None,
        );
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "rollback");
        assert_eq!(warnings[0].detail, "explicit rollback");
    }

    #[test]
    fn decision_record_preserves_structural_payload() {
        let record = DecisionRecord {
            stage_index: Some(3),
            detail: DecisionDetail::StageAuthorityPolicy {
                authority_policy: AuthorityPolicy::AuthoritativeOnly,
            },
        };

        assert_eq!(record.stage_index, Some(3));
        assert!(matches!(
            record.detail,
            DecisionDetail::StageAuthorityPolicy {
                authority_policy: AuthorityPolicy::AuthoritativeOnly
            }
        ));
    }
}
