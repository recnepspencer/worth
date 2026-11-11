//! Abandoned real writers retain every refused cleanup until the caller owns it.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryProviderProgressionOutcome as Progression,
    WorthQueryApplicationCleanupRequired, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationUncommitted as Uncommitted,
};
impl<S: ApplicationSchema> CoupledApplicationAssembly<'_, '_, '_, S> {
    pub(in crate::domain_computation::primary_graph) fn abort_writers(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> bool {
        for writer in self.authored.borrow_mut().drain(..) {
            let outcome = match writer.stage {
                Some(CoupledWriterStage::Inspected(writer)) => {
                    writer.abort(execution, self.phase, self.application)
                }
                Some(CoupledWriterStage::Prepared(writer)) => {
                    writer.abort(execution, self.phase, self.application)
                }
                None => continue,
            };
            self.retain_abort_outcome(outcome);
        }
        for writer in self.writers.borrow_mut().drain(..) {
            let Some(stage) = writer.stage else { continue };
            let outcome = match stage {
                CoupledWriterStage::Inspected(writer) => {
                    writer.abort(execution, self.phase, self.application)
                }
                CoupledWriterStage::Prepared(writer) => {
                    writer.abort(execution, self.phase, self.application)
                }
            };
            self.retain_abort_outcome(outcome);
        }
        self.abort_failures.borrow().is_empty()
    }
    pub(super) fn retain_abort_outcome(&self, outcome: WorthQueryApplicationCommitOutcome) {
        match outcome.landed() {
            Err(Uncommitted::Aborted) => {}
            Err(failure) => self.abort_failures.borrow_mut().push(failure),
            Ok(_) => unreachable!("discarding an unpublished writer cannot perform a publication"),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn finish_abort(
        &self,
        outcome: Progression,
    ) -> Progression {
        let mut failures = self.abort_failures.borrow_mut();
        if failures.is_empty() {
            return outcome;
        }
        let Ok(mut evidence) = outcome.into_uncommitted() else {
            unreachable!("only abandoned candidates finish abort")
        };
        for failure in failures.drain(..) {
            evidence = match failure {
                Uncommitted::CleanupRequired(failure) => {
                    let (_, cleanup) = failure.into_parts();
                    Uncommitted::CleanupRequired(WorthQueryApplicationCleanupRequired::from_parts(
                        evidence, cleanup,
                    ))
                }
                failure => match evidence {
                    prior @ Uncommitted::CleanupRequired(_) => prior,
                    _ => failure,
                },
            };
        }
        Progression::CoupledFailure(evidence)
    }
}
