//! Fresh authorization lowers to the original source, workflow and history admissions.
use super::*;
pub(super) fn prepare_authorized<Schema, Intent, SourcePreparation>(
    phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    request: &Request<'_, '_, '_, '_, Schema, Intent, SourcePreparation>,
    identities: &Identities<'_, Schema, Intent>,
    source: Option<
        WorthQueryMutationExpectedSource<
            <<IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::SourceExpectation as ApplicationMutationSourceExpectation<Schema>>::Query,
        >,
    >,
    authorized: AuthorizedMutation<Schema, IntentBinding<Schema, Intent>>,
) -> Result<PreparedMutation<Schema, IntentBinding<Schema, Intent>>, WorthQueryApplicationRequestMutationDenial>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
{
    let principal_identity = authorized.principal_identity;
    let mut admission = authorized.admission;
    if let Some(expected) = &request.workflow_captured_input {
        use worth_query_execution::publication_boundary::workflow_proposal::WorkflowCapturedInputSource;
        let original_source = match source.as_ref() {
            None => WorkflowCapturedInputSource::None,
            Some(WorthQueryMutationExpectedSource::Row(s)) => WorkflowCapturedInputSource::Row(s),
            Some(WorthQueryMutationExpectedSource::ResultSet(s)) => {
                WorkflowCapturedInputSource::ResultSet(s)
            }
        };
        expected
            .validate_admission(
                &worth_query_execution::publication_boundary::program_publication_access(),
                &admission,
                identities,
                original_source,
            )
            .map_err(WorthQueryApplicationRequestMutationDenial::WorkflowCapturedInput)?;
    }
    // The request encoded its key and input once, before admission; report that
    // work in the admission phase so a fresh commit and a replayed retry both
    // carry it.
    admission.record_request_identity_work(
        &worth_query_execution::publication_boundary::program_publication_access(),
        identities.canonical_work(),
    );

    let expected_source = <<IntentBinding<Schema, Intent> as ApplicationMutationBinding<
        Schema,
    >>::SourceExpectation as ApplicationMutationSourceExpectation<Schema>>::QUERY_IDENTIFIER;
    let mut pending_source = None;
    match (expected_source, source) {
        (Some(_expected), Some(source)) => {
            pending_source = Some(match source {
                WorthQueryMutationExpectedSource::Row(source) => request
                    .request
                    .application
                    .bind_application_source_expectation::<IntentBinding<Schema, Intent>, _>(
                        phase,
                        &mut admission,
                        source,
                        identities.mutation_input(),
                    ),
                WorthQueryMutationExpectedSource::ResultSet(source) => request
                    .request
                    .application
                    .bind_application_result_set_expectation::<IntentBinding<Schema, Intent>, _>(
                        phase,
                        &mut admission,
                        source,
                        identities.mutation_input(),
                    ),
            }
            .map_err(WorthQueryApplicationRequestMutationDenial::SourceExpectation)?);
        }
        (Some(expected), None) => {
            return Err(WorthQueryApplicationRequestMutationDenial::SourceExpectation(
                worth_query_execution::facade::primary_graph::WorthQuerySourceExpectationDenial::new_missing(expected),
            ));
        }
        (None, None) => {}
        (None, Some(_)) => unreachable!("a no-source binding has no constructible source marker"),
    }
    let extension = WorthQueryCommitExtension {
        source: pending_source
            .as_ref()
            .map(|pending| pending.bound_source()),
        workflow_transition: request.workflow_transition_identity,
        workflow_birth: request.workflow_birth_identity,
        workflow_succession: request.workflow_succession_identity,
        correction: None,
    };
    let idempotency = extension
        .apply(WorthQueryApplicationIdempotencyBinding::for_mutation_identities(identities));
    if let Some(restriction) = request.history_constraint {
        request
            .request
            .application
            .validate_history_operation(
                phase,
                &worth_query_execution::publication_boundary::program_publication_access(),
                restriction,
                &admission,
                idempotency,
            )
            .map_err(WorthQueryApplicationRequestMutationDenial::DocumentHistory)?;
    }
    Ok(PreparedMutation {
        principal_identity,
        admission,
        pending_source,
        extension,
        idempotency,
    })
}
