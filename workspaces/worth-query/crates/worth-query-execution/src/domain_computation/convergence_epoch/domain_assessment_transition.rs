use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::domain_computation::WorthQueryBoundGraphExecutionReceipt;

use super::core::WorthQueryConvergenceEpochCore;
use crate::domain_computation::provider_session::graph_provider::convergence_port::WorthQueryConvergenceDomainAssessmentOutcome;
use {
    super::WorthQueryConvergenceDomainInvocationFailure,
    super::WorthQueryConvergenceDomainInvocationFailureKind as FailureKind,
    super::WorthQueryConvergenceDomainPhase as Phase,
    crate::domain_computation::provider_session::graph_provider::convergence_port::WorthQueryConvergenceAssessment,
    crate::domain_computation::provider_session::graph_provider::convergence_port::WorthQueryConvergenceComparison,
    crate::domain_computation::provider_session::graph_provider::convergence_port::WorthQueryConvergenceDomainFailure,
    crate::domain_computation::provider_session::graph_provider::convergence_port::WorthQueryConvergenceDomainProvider,
    crate::domain_computation::provider_session::graph_provider::convergence_port::WorthQueryConvergenceDomainWorkEvidence,
    crate::domain_computation::provider_session::graph_provider::convergence_port::WorthQueryConvergenceProgress,
    crate::domain_computation::provider_session::graph_provider::convergence_port::WorthQueryConvergenceRepeatedState,
};

pub(super) fn assess_domain_report(
    core: &mut WorthQueryConvergenceEpochCore,
    provider: &dyn WorthQueryConvergenceDomainProvider,
    receipt: &WorthQueryBoundGraphExecutionReceipt,
) -> Result<
    WorthQueryConvergenceDomainAssessmentOutcome,
    WorthQueryConvergenceDomainInvocationFailure,
> {
    let iteration_ordinal = core.counters().iteration_count();
    core.record_lifecycle_event(DomainAssessmentLifecycleEvent::provider_work(
        receipt.work_report().completed_work_units(),
    ));
    let mut work = WorthQueryConvergenceDomainWorkEvidence::empty();
    let result = {
        let assessment = WorthQueryConvergenceAssessment::new(
            core.contract(),
            receipt,
            iteration_ordinal,
            core.incumbents(),
        );
        invoke_comparator(provider, &assessment, &mut work).and_then(|comparison| {
            invoke_progress(provider, &assessment, &comparison, &mut work).and_then(|progress| {
                invoke_repeated_state(provider, &assessment, &comparison, progress, &mut work).map(
                    |repeated_state| {
                        WorthQueryConvergenceDomainAssessmentOutcome::new(
                            comparison,
                            progress,
                            repeated_state,
                            work,
                        )
                    },
                )
            })
        })
    };
    core.record_lifecycle_event(DomainAssessmentLifecycleEvent::domain_work(work));
    result
}

pub(in crate::domain_computation::convergence_epoch) struct DomainAssessmentLifecycleEvent {
    kind: DomainAssessmentLifecycleEventKind,
}

pub(in crate::domain_computation::convergence_epoch) enum DomainAssessmentLifecycleEventKind {
    ProviderWork(u64),
    DomainWork(WorthQueryConvergenceDomainWorkEvidence),
}

impl DomainAssessmentLifecycleEvent {
    fn provider_work(completed_work_units: u64) -> Self {
        Self {
            kind: DomainAssessmentLifecycleEventKind::ProviderWork(completed_work_units),
        }
    }

    fn domain_work(work: WorthQueryConvergenceDomainWorkEvidence) -> Self {
        Self {
            kind: DomainAssessmentLifecycleEventKind::DomainWork(work),
        }
    }

    pub(in crate::domain_computation::convergence_epoch) fn into_kind(
        self,
    ) -> DomainAssessmentLifecycleEventKind {
        self.kind
    }
}

fn invoke_comparator(
    provider: &dyn WorthQueryConvergenceDomainProvider,
    assessment: &WorthQueryConvergenceAssessment<'_>,
    work: &mut WorthQueryConvergenceDomainWorkEvidence,
) -> Result<WorthQueryConvergenceComparison, WorthQueryConvergenceDomainInvocationFailure> {
    work.called_comparator();
    invoke_domain_port(
        Phase::Comparator,
        *work,
        || provider.compare(assessment),
        "installed convergence comparator panicked",
    )
}

fn invoke_progress(
    provider: &dyn WorthQueryConvergenceDomainProvider,
    assessment: &WorthQueryConvergenceAssessment<'_>,
    comparison: &WorthQueryConvergenceComparison,
    work: &mut WorthQueryConvergenceDomainWorkEvidence,
) -> Result<WorthQueryConvergenceProgress, WorthQueryConvergenceDomainInvocationFailure> {
    work.checked_progress();
    invoke_domain_port(
        Phase::ProgressMeasure,
        *work,
        || provider.measure_progress(assessment, comparison),
        "installed convergence progress measure panicked",
    )
}

fn invoke_repeated_state(
    provider: &dyn WorthQueryConvergenceDomainProvider,
    assessment: &WorthQueryConvergenceAssessment<'_>,
    comparison: &WorthQueryConvergenceComparison,
    progress: WorthQueryConvergenceProgress,
    work: &mut WorthQueryConvergenceDomainWorkEvidence,
) -> Result<WorthQueryConvergenceRepeatedState, WorthQueryConvergenceDomainInvocationFailure> {
    work.probed_repeated_state();
    invoke_domain_port(
        Phase::RepeatedStateDetector,
        *work,
        || provider.detect_repeated_state(assessment, comparison, progress),
        "installed convergence repeated-state detector panicked",
    )
}

fn invoke_domain_port<T>(
    phase: Phase,
    work: WorthQueryConvergenceDomainWorkEvidence,
    invoke: impl FnOnce() -> Result<T, WorthQueryConvergenceDomainFailure>,
    panic_detail: &'static str,
) -> Result<T, WorthQueryConvergenceDomainInvocationFailure> {
    match catch_unwind(AssertUnwindSafe(invoke)) {
        Ok(Ok(outcome)) => Ok(outcome),
        Ok(Err(failure)) => Err(WorthQueryConvergenceDomainInvocationFailure::new(
            phase,
            FailureKind::Rejected,
            failure.detail(),
            work,
        )),
        Err(_) => Err(WorthQueryConvergenceDomainInvocationFailure::new(
            phase,
            FailureKind::Panicked,
            panic_detail,
            work,
        )),
    }
}
