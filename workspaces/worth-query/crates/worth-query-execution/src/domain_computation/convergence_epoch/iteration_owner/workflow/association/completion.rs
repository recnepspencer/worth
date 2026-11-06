//! Workflow managed completion transitions over one sealed association.

use super::super::super::core::{
    admit_assessed_domain_report, WorthQueryConvergenceReportAdmissionFailure,
};
use super::WorkflowIterationAssociation;
use {
    super::super::super::super::domain_assessment_transition::assess_domain_report,
    super::super::super::super::terminal_outcome::semantic_terminal_kind,
    super::super::super::super::terminal_outcome::workflow_terminal_outcome,
    super::super::super::super::WorthQueryConvergenceIndeterminateCause,
    super::super::super::super::WorthQueryConvergenceTerminalKind,
    crate::domain_computation::provider_session::graph_provider::convergence_port::WorthQueryConvergenceDisposition,
    crate::domain_computation::provider_session::graph_provider::convergence_port::WorthQueryConvergenceDomainAssessmentOutcome,
};
use {
    crate::domain_computation::vocabulary::managed_run::terminal::WorthQueryManagedRunTerminalKind,
    crate::domain_computation::WorthQueryCompletedWorkflowGraphExecution,
    crate::domain_computation::WorthQueryWorkflowRunTerminal,
};

impl WorkflowIterationAssociation<WorthQueryCompletedWorkflowGraphExecution> {
    pub(super) fn admit_completion(
        mut self,
    ) -> super::super::WorthQueryWorkflowConvergenceIterationOutcome {
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
    ) -> super::super::WorthQueryWorkflowConvergenceIterationOutcome {
        let evidence = match self.managed.bind_convergence_candidate_evidence(
            &self.stage_identity,
            assessment.decision().candidate_selection_key(),
        ) {
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
    ) -> super::super::WorthQueryWorkflowConvergenceIterationOutcome {
        let Self {
            core,
            graph,
            provider,
            stage_identity: _,
            managed,
        } = self;
        let running = managed.into_running();
        if disposition == WorthQueryConvergenceDisposition::Continue {
            return continue_or_exhaust(core, graph, provider, running);
        }
        let kind = semantic_terminal_kind(disposition);
        match running.completed() {
            Ok(terminal) => workflow_terminal_outcome(core, terminal, kind, None),
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
    provider: std::sync::Arc<dyn crate::domain_computation::provider_session::graph_provider::convergence_port::WorthQueryConvergenceDomainProvider>,
    running: crate::domain_computation::WorthQueryRunningWorkflowRun,
) -> super::super::WorthQueryWorkflowConvergenceIterationOutcome {
    if core.counters().iteration_count() >= core.contract().iteration_bound() {
        return workflow_terminal_outcome(
            core,
            running.terminate_for_convergence(WorthQueryManagedRunTerminalKind::Exhausted),
            WorthQueryConvergenceTerminalKind::Exhausted,
            None,
        );
    }
    super::super::WorthQueryWorkflowConvergenceIterationOutcome::Continue(
        super::super::WorthQueryIteratingWorkflowConvergenceEpoch {
            association: super::WorkflowIteratingEpochAssociation {
                core,
                managed_run: running,
                graph,
                provider,
            },
        },
    )
}

impl WorkflowIterationAssociation<WorthQueryWorkflowRunTerminal> {
    pub(super) fn admit_terminal(
        mut self,
    ) -> super::super::WorthQueryWorkflowConvergenceIterationOutcome {
        self.core
            .record_lifecycle_event(WorkflowTerminalProviderWorkEvent::new(
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
        workflow_terminal_outcome(self.core, self.managed, kind, cause)
    }
}

pub(in crate::domain_computation::convergence_epoch) struct WorkflowTerminalProviderWorkEvent {
    completed_work_units: u64,
}

impl WorkflowTerminalProviderWorkEvent {
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
    running: crate::domain_computation::WorthQueryRunningWorkflowRun,
    cause: WorthQueryConvergenceIndeterminateCause,
) -> super::super::WorthQueryWorkflowConvergenceIterationOutcome {
    workflow_terminal_outcome(
        core,
        running.terminate_for_convergence(WorthQueryManagedRunTerminalKind::Failed),
        WorthQueryConvergenceTerminalKind::Indeterminate,
        Some(cause),
    )
}
