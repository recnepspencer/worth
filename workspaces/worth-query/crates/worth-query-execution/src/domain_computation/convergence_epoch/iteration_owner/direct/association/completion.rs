//! Direct managed completion transitions over one sealed association.

use super::super::super::super::{
    domain_assessment_transition::assess_domain_report,
    domain_work::WorthQueryConvergenceDomainAssessmentOutcome,
    terminal_outcome::{direct_terminal_outcome, semantic_terminal_kind},
    WorthQueryConvergenceDisposition, WorthQueryConvergenceIndeterminateCause,
    WorthQueryConvergenceTerminalKind,
};
use super::super::super::core::{
    admit_assessed_domain_report, WorthQueryConvergenceReportAdmissionFailure,
};
use super::DirectIterationAssociation;
use {crate::domain_computation::WorthQueryCompletedDirectGraphExecution, crate::domain_computation::WorthQueryDirectRunTerminal, crate::domain_computation::vocabulary::managed_run::terminal::WorthQueryManagedRunTerminalKind};

impl DirectIterationAssociation<WorthQueryCompletedDirectGraphExecution> {
    pub(super) fn admit_completion(
        mut self,
    ) -> super::super::WorthQueryDirectConvergenceIterationOutcome {
        let assessment = match assess_domain_report(
            &mut self.core,
            self.provider.as_ref(),
            self.managed.receipt(),
        ) {
            Ok(assessment) => assessment,
            Err(failure) => {
                return indeterminate(
                    self.core,
                    self.managed.into_running(),
                    WorthQueryConvergenceIndeterminateCause::DomainInvocation(failure),
                )
            }
        };
        self.admit_assessment(assessment)
    }

    fn admit_assessment(
        mut self,
        assessment: WorthQueryConvergenceDomainAssessmentOutcome,
    ) -> super::super::WorthQueryDirectConvergenceIterationOutcome {
        let evidence = match self
            .managed
            .bind_convergence_candidate_evidence(assessment.decision().candidate_selection_key())
        {
            Ok(evidence) => evidence,
            Err(denial) => {
                return indeterminate(
                    self.core,
                    self.managed.into_running(),
                    WorthQueryConvergenceIndeterminateCause::DomainEvidenceBinding(denial),
                )
            }
        };
        let disposition = match admit_assessed_domain_report(
            &mut self.core,
            self.managed.receipt(),
            assessment,
            evidence,
        ) {
            Ok(disposition) => disposition,
            Err(WorthQueryConvergenceReportAdmissionFailure::Epoch(denial)) => {
                return indeterminate(
                    self.core,
                    self.managed.into_running(),
                    WorthQueryConvergenceIndeterminateCause::ReportAdmission(denial),
                )
            }
        };
        self.finish(disposition)
    }

    fn finish(
        self,
        disposition: WorthQueryConvergenceDisposition,
    ) -> super::super::WorthQueryDirectConvergenceIterationOutcome {
        let Self {
            core,
            graph,
            provider,
            managed,
        } = self;
        let running = managed.into_running();
        if disposition == WorthQueryConvergenceDisposition::Continue {
            return continue_or_exhaust(core, graph, provider, running);
        }
        let kind = semantic_terminal_kind(disposition);
        match running.completed() {
            Ok(terminal) => direct_terminal_outcome(core, terminal, kind, None),
            Err(rejection) => {
                let denial = rejection.denial().clone();
                indeterminate(
                    core,
                    rejection.into_running(),
                    WorthQueryConvergenceIndeterminateCause::ManagedCompletion(denial),
                )
            }
        }
    }
}

fn continue_or_exhaust(
    core: super::super::super::core::WorthQueryConvergenceEpochCore,
    graph: worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
    provider: std::sync::Arc<dyn super::super::super::super::WorthQueryConvergenceDomainProvider>,
    running: crate::domain_computation::WorthQueryRunningDirectRun,
) -> super::super::WorthQueryDirectConvergenceIterationOutcome {
    if core.counters().iteration_count() >= core.contract().iteration_bound() {
        return direct_terminal_outcome(
            core,
            running.terminate_for_convergence(WorthQueryManagedRunTerminalKind::Exhausted),
            WorthQueryConvergenceTerminalKind::Exhausted,
            None,
        );
    }
    super::super::WorthQueryDirectConvergenceIterationOutcome::Continue(
        super::super::WorthQueryIteratingDirectConvergenceEpoch {
            association: super::DirectIteratingEpochAssociation {
                core,
                managed_run: running,
                graph,
                provider,
            },
        },
    )
}

impl DirectIterationAssociation<WorthQueryDirectRunTerminal> {
    pub(super) fn admit_terminal(
        mut self,
    ) -> super::super::WorthQueryDirectConvergenceIterationOutcome {
        self.core
            .record_lifecycle_event(DirectTerminalProviderWorkEvent::new(
                self.managed.provider_work().completed_work_units(),
            ));
        let managed_kind = self.managed.kind();
        let kind = match managed_kind {
            WorthQueryManagedRunTerminalKind::Cancelled => {
                WorthQueryConvergenceTerminalKind::Cancelled
            }
            WorthQueryManagedRunTerminalKind::Exhausted => {
                WorthQueryConvergenceTerminalKind::Exhausted
            }
            _ => WorthQueryConvergenceTerminalKind::Indeterminate,
        };
        let cause = (kind == WorthQueryConvergenceTerminalKind::Indeterminate).then_some(
            WorthQueryConvergenceIndeterminateCause::ManagedTerminal(managed_kind),
        );
        direct_terminal_outcome(self.core, self.managed, kind, cause)
    }
}

pub(in crate::domain_computation::convergence_epoch) struct DirectTerminalProviderWorkEvent {
    completed_work_units: u64,
}

impl DirectTerminalProviderWorkEvent {
    fn new(completed_work_units: u64) -> Self {
        Self {
            completed_work_units,
        }
    }

    pub(in crate::domain_computation::convergence_epoch) fn completed_work_units(self) -> u64 {
        self.completed_work_units
    }
}

fn indeterminate(
    core: super::super::super::core::WorthQueryConvergenceEpochCore,
    running: crate::domain_computation::WorthQueryRunningDirectRun,
    cause: WorthQueryConvergenceIndeterminateCause,
) -> super::super::WorthQueryDirectConvergenceIterationOutcome {
    direct_terminal_outcome(
        core,
        running.terminate_for_convergence(WorthQueryManagedRunTerminalKind::Failed),
        WorthQueryConvergenceTerminalKind::Indeterminate,
        Some(cause),
    )
}
