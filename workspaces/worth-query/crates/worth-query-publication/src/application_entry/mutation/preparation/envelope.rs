//! Consume an already resolved envelope's one genuine prefix candidate.
use super::*;
impl<'a, 'p, 's, 'k, S, I, Source>
    WorthQueryApplicationMutationRequestWithIdempotency<'a, 'p, 's, 'k, S, I, Source>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
    <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeResolution<
        S,
        <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity,
    >,
{
    pub(in crate::application_entry) fn prepare_resolved_envelope_candidate<'request>(
        &'request self,
        phase: &AdvancementPhase<'_>,
        identities: ApplicationMutationIdentities<'request, S, I::Binding>,
        extension: WorthQueryCommitExtension,
        outcome: HandlerResult<
            worth_query_execution::facade::primary_graph::WorthQueryCompletedMutationCandidate<
                S,
                I::Binding,
            >,
            <I::Binding as ApplicationMutationBinding<S>>::Denial,
        >,
        work: worth_query_execution::facade::primary_graph::WorthQueryMutationHandlerWork,
        allocation: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        CandidatePreparation<'request, S, I::Binding>,
        WorthQueryApplicationRequestMutationDenial,
    > {
        let completed = match outcome {
            HandlerResult::Completed(completed) => completed,
            HandlerResult::DomainDenied(denial) => {
                return Ok(CandidatePreparation::Settled(
                    WorthQueryApplicationMutationOutcome::DomainDenied(denial),
                ))
            }
            HandlerResult::Cancelled => {
                return Ok(CandidatePreparation::Settled(
                    WorthQueryApplicationMutationOutcome::Cancelled,
                ))
            }
            HandlerResult::DeadlineExceeded => {
                return Ok(CandidatePreparation::Settled(
                    WorthQueryApplicationMutationOutcome::DeadlineExceeded,
                ))
            }
            HandlerResult::ExecutionDenied(cause) => {
                return Err(WorthQueryApplicationRequestMutationDenial::Handler(
                    MutationHandlerExecutionDenial::Handler(cause),
                ))
            }
        };
        let prepared = self.finish_executed_candidate(
            phase,
            handler_execution::ExecutedCandidate {
                completed,
                identities,
                extension,
                work,
                workflow_authority: None,
            },
            allocation,
        )?;
        match prepared {
            CandidatePreparation::Prepared(mut candidate) => {
                let access =
                    worth_query_execution::publication_boundary::program_publication_access();
                if !candidate.program.requires_original_input_retention(&access) {
                    candidate.program = candidate
                        .program
                        .retain_performed_correction_input::<I::Binding>(
                            &access,
                            phase,
                            self.request.application,
                            candidate.identities.encoded_input(),
                        )
                        .map_err(|cause| {
                            WorthQueryApplicationRequestMutationDenial::Handler(
                                MutationHandlerExecutionDenial::Attempt(cause),
                            )
                        })?;
                }
                Ok(CandidatePreparation::Prepared(candidate))
            }
            settled => Ok(settled),
        }
    }
}
