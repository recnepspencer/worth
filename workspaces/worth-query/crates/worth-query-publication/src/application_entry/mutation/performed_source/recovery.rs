//! Native program-source recovery transitions shared by fixed and discovered outputs.
use worth_query_execution::facade::application_installation::WorthQueryRecoveredProgramOutputSource;
use worth_query_execution::facade::domain_computation::{
    WorthQueryProductUnpublishedRecovery, WorthQueryProductUnpublishedRecoveryReleaseFailure,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationIdempotencyBinding,
    WorthQueryManagedApplicationRecoveryDenial, WorthQueryManagedApplicationRecoveryOutcome,
    WorthQueryManagedApplicationRecoveryPerformed, WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_installation::facade::ApplicationSchema;
pub(in crate::application_entry::mutation) enum ProgramSourceRecoveryPhase {
    Unpublished(WorthQueryProductUnpublishedRecovery),
    Performed {
        carrier: RecoveredCarrier,
        outcome: WorthQueryManagedApplicationRecoveryPerformed,
    },
}

pub(in crate::application_entry::mutation) enum RecoveredCarrier {
    Retained(WorthQueryRecoveredProgramOutputSource),
    Unavailable,
}

impl ProgramSourceRecoveryPhase {
    pub(in crate::application_entry::mutation) fn advance<Schema, Operation, Input, Scope>(
        &mut self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        prior_cleanup: &mut Vec<WorthQueryProductUnpublishedRecoveryReleaseFailure>,
    ) -> Result<WorthQueryProgramSourceRecoveryProgress, WorthQueryManagedApplicationRecoveryDenial>
    where
        Schema: ApplicationSchema,
        Input: Clone + Send + Sync + 'static,
    {
        use WorthQueryProgramSourceRecoveryProgress as Progress;
        let partial = match self {
            Self::Unpublished(partial) => partial,
            Self::Performed { .. } => return Ok(Progress::Performed),
        };
        prior_cleanup
            .try_reserve(1)
            .map_err(|_| WorthQueryManagedApplicationRecoveryDenial::ProviderCapacity)?;
        let (outcome, carrier) = runtime.recover_admitted_unpublished_program_source(
            phase,
            &worth_query_execution::publication_boundary::program_publication_access(),
            partial,
            admission,
            idempotency,
        )?;
        Ok(match outcome {
            WorthQueryManagedApplicationRecoveryOutcome::NoEffect => Progress::NoEffect,
            WorthQueryManagedApplicationRecoveryOutcome::ProductUnpublished {
                next,
                prior_cleanup_failure,
            } => {
                *self = Self::Unpublished(next);
                if let Some(failure) = prior_cleanup_failure {
                    prior_cleanup.push(failure);
                }
                Progress::ProductUnpublished
            }
            WorthQueryManagedApplicationRecoveryOutcome::Performed(outcome) => {
                *self = Self::Performed {
                    carrier: carrier
                        .map_or(RecoveredCarrier::Unavailable, RecoveredCarrier::Retained),
                    outcome,
                };
                Progress::Performed
            }
        })
    }
}

/// Native progress only; Performed retains independent publication/read/cleanup obligations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProgramSourceRecoveryProgress {
    NoEffect,
    ProductUnpublished,
    Performed,
}
