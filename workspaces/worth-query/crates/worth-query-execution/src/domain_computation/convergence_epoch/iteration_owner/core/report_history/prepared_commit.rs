use super::WorthQueryConvergenceReportHistory;
use {
    crate::domain_computation::provider_session::graph_provider::convergence_port::WorthQueryConvergenceIncumbentUpdate,
    crate::domain_computation::provider_session::graph_provider::convergence_port::WorthQueryRetainedConvergenceCandidateEvidence,
    crate::domain_computation::WorthQueryBoundConvergenceReport,
};

pub(in crate::domain_computation::convergence_epoch::iteration_owner::core) struct WorthQueryPreparedConvergenceReportCommit
{
    kind: PreparedReportHistoryKind,
    report: WorthQueryBoundConvergenceReport,
}

enum PreparedReportHistoryKind {
    Retain,
    ReplaceWithCandidate(WorthQueryRetainedConvergenceCandidateEvidence),
    AddCandidate(WorthQueryRetainedConvergenceCandidateEvidence),
    RemoveCandidatesAndAdd(WorthQueryRetainedConvergenceCandidateEvidence),
    Clear,
}

impl WorthQueryPreparedConvergenceReportCommit {
    pub(super) fn retain(report: WorthQueryBoundConvergenceReport) -> Self {
        Self {
            kind: PreparedReportHistoryKind::Retain,
            report,
        }
    }

    pub(super) fn replace_with_candidate(
        report: WorthQueryBoundConvergenceReport,
        candidate: WorthQueryRetainedConvergenceCandidateEvidence,
    ) -> Self {
        Self {
            kind: PreparedReportHistoryKind::ReplaceWithCandidate(candidate),
            report,
        }
    }

    pub(super) fn add_candidate(
        report: WorthQueryBoundConvergenceReport,
        candidate: WorthQueryRetainedConvergenceCandidateEvidence,
    ) -> Self {
        Self {
            kind: PreparedReportHistoryKind::AddCandidate(candidate),
            report,
        }
    }

    pub(super) fn remove_candidates_and_add(
        report: WorthQueryBoundConvergenceReport,
        candidate: WorthQueryRetainedConvergenceCandidateEvidence,
    ) -> Self {
        Self {
            kind: PreparedReportHistoryKind::RemoveCandidatesAndAdd(candidate),
            report,
        }
    }

    pub(super) fn clear(report: WorthQueryBoundConvergenceReport) -> Self {
        Self {
            kind: PreparedReportHistoryKind::Clear,
            report,
        }
    }
}

impl WorthQueryConvergenceReportHistory {
    pub(in crate::domain_computation::convergence_epoch::iteration_owner::core) fn commit(
        &mut self,
        prepared: WorthQueryPreparedConvergenceReportCommit,
    ) -> ReportHistoryLifecycleEvent {
        let WorthQueryPreparedConvergenceReportCommit { kind, report } = prepared;
        let event = match kind {
            PreparedReportHistoryKind::Retain => ReportHistoryLifecycleEvent::retained(),
            PreparedReportHistoryKind::ReplaceWithCandidate(candidate) => {
                self.incumbents.clear();
                self.incumbents.push(candidate);
                ReportHistoryLifecycleEvent::incumbent_set_replaced()
            }
            PreparedReportHistoryKind::AddCandidate(candidate) => {
                self.incumbents.push(candidate);
                ReportHistoryLifecycleEvent::incumbent_set_replaced()
            }
            PreparedReportHistoryKind::RemoveCandidatesAndAdd(candidate) => {
                let WorthQueryConvergenceIncumbentUpdate::RemoveCandidatesAndAdd {
                    removed_occurrence_identities,
                } = report.decision().incumbent_update()
                else {
                    unreachable!("prepared remove-and-add commit must retain its exact decision")
                };
                self.incumbents.retain(|incumbent| {
                    !removed_occurrence_identities
                        .iter()
                        .any(|removed| removed.as_ref() == incumbent.occurrence_identity())
                });
                self.incumbents.push(candidate);
                ReportHistoryLifecycleEvent::incumbent_set_replaced()
            }
            PreparedReportHistoryKind::Clear => {
                self.incumbents.clear();
                ReportHistoryLifecycleEvent::incumbent_set_replaced()
            }
        };
        self.latest_report = Some(report);
        event
    }
}

pub(in crate::domain_computation::convergence_epoch::iteration_owner::core) struct ReportHistoryLifecycleEvent
{
    kind: ReportHistoryLifecycleEventKind,
}

pub(in crate::domain_computation::convergence_epoch::iteration_owner::core) enum ReportHistoryLifecycleEventKind
{
    Retained,
    IncumbentSetReplaced,
}

impl ReportHistoryLifecycleEvent {
    fn retained() -> Self {
        Self {
            kind: ReportHistoryLifecycleEventKind::Retained,
        }
    }

    fn incumbent_set_replaced() -> Self {
        Self {
            kind: ReportHistoryLifecycleEventKind::IncumbentSetReplaced,
        }
    }

    pub(in crate::domain_computation::convergence_epoch::iteration_owner::core) fn into_kind(
        self,
    ) -> ReportHistoryLifecycleEventKind {
        self.kind
    }
}
