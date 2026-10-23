//! Publish one correction with exactly its installed action's delivery obligation.
use super::*;
use crate::application_entry::correction::publication::PreparedInverse;
use worth_query_execution::facade::application_installation::WorthQueryProgramOwner;
use worth_query_execution::facade::primary_graph::WorthQueryApplicationRetainedCommitOutcome;

/// An unpublished inverse and its already-constructed native delivery owner.
pub struct WorthQueryPreparedOperationCorrection<'a, S, B, P, A>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
    B::Input: ApplicationMutationIntent<S, Binding = B>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, B>,
    Mode<S, B, A>: WorthQuerySourceDeliveryMode,
{
    pub(super) inverse: PreparedInverse<'a, S, B>,
    pub(super) operation: Operation<S, B, P, A>,
}

impl<S, B, P, A> WorthQueryPreparedOperationCorrection<'_, S, B, P, A>
where
    S: ApplicationSchema + 'static,
    B: ApplicationMutationAdmissionBinding<S>,
    B::Admission: WorthQueryBindingAdmission<S, B>,
    B::Input: Clone + Send + Sync + ApplicationMutationIntent<S, Binding = B>,
    B::ScopeBinding: ApplicationMutationScopeResolution<S, B::PrincipalIdentity>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, B>,
    Mode<S, B, A>: WorthQuerySourceDeliveryMode,
{
    /// Publishes once and transfers every native outcome into ordinary source custody.
    pub fn publish(self) -> Result<Advance<S, B, P, A>, WorthQueryApplicationCorrectionDenial> {
        use WorthQueryApplicationCorrectionDenial as Denial;
        let Self { inverse, operation } = self;
        let PreparedInverse {
            application,
            owner,
            program,
            identities,
            result,
            source,
        } = inverse;
        let scope = program.request_scope().clone();
        application
            .with_application_advancement(&scope, |phase| {
                let execution = phase
                    .checked_execution_request_for(application.product_runtime())
                    .map_err(Denial::ExecutionRequest)?;
                let outcome = if Mode::<S, B, A>::RETAIN_COMMIT {
                    owner.commit_program_correction_retained_in_advancement(
                    &worth_query_execution::publication_boundary::program_publication_access(),
                    &phase,
                    program,
                    &identities.borrow(),
                    source,
                    execution.allocation_policy(),
                    )
                } else {
                    WorthQueryApplicationRetainedCommitOutcome::Other(
                        owner.commit_program_correction_in_advancement(
                            &worth_query_execution::publication_boundary::program_publication_access(),
                            &phase, program, &identities.borrow(), source,
                            execution.allocation_policy(),
                        )
                    )
                };
                let outcome = match outcome {
                    WorthQueryApplicationRetainedCommitOutcome::Committed { receipt, retained } => {
                        WorthQueryApplicationRetainedMutationOutcome::Committed {
                            receipt,
                            result,
                            retained:
                                crate::application_entry::WorthQueryApplicationReadObservation::new(
                                    retained,
                                ),
                        }
                    }
                    WorthQueryApplicationRetainedCommitOutcome::Other(outcome) => {
                        let outcome = match outcome.landed() {
                            Ok((receipt, true)) => {
                                WorthQueryApplicationMutationOutcome::AlreadyCommitted(receipt)
                            }
                            Ok((receipt, false)) => {
                                WorthQueryApplicationMutationOutcome::Committed { receipt, result }
                            }
                            Err(custody) => WorthQueryApplicationMutationOutcome::Commit(custody),
                        };
                        WorthQueryApplicationRetainedMutationOutcome::Other(outcome)
                    }
                };
                Ok(operation.accept_recorded_publication(outcome))
            })
            .map_err(Denial::ExecutionRequest)?
    }
}
