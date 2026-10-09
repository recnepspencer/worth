use super::*;

pub(in crate::domain_computation::primary_graph::application_attempt::provider_execution) fn prepare_application_commit<
    Schema,
    Operation,
    Input,
    Scope,
>(
    phase: &WorthQueryAdvancementPhase<'_>,

    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    request: WorthQueryApplicationCommitPreparationRequest<Schema, Operation, Input, Scope>,
) -> WorthQueryApplicationCommitPreparation<Schema, Operation, Input, Scope>
where
    Schema: ApplicationSchema,
    Input: Clone + Send + Sync + 'static,
{
    if let Err(cause) = phase.execution_request_for(&application.product_runtime) {
        return terminal(
            crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::from(cause)
                .into_commit_outcome(),
        );
    }
    let WorthQueryApplicationCommitPreparationRequest {
        program,
        idempotency,
        elevation_currentness,
        aftermath_causality,
    } = request;
    let WorthQueryApplicationEffectProgram {
        read_set,
        effects,
        emission_retained_bytes,
        emission_retained_bytes_ceiling,
        conditional_definition,
        effect_posture,
        validator_work_admission,
        output_correspondence,
        retain_output_demand_observation,
        retain_client_observation,
        producer_required_invariants,
        output_currentness_facts,
    } = program;
    let workflow_settlement = read_set.workflow_authority_binding;
    let workflow_deadline = read_set.workflow_deadline;
    let new_commit_refusal = read_set.new_commit_refusal;
    let mutation_proof = read_set.mutation_handler_binding;
    let workflow_approval_authority = workflow_settlement
        .as_ref()
        .map(|binding| binding.approval_authority.clone());
    let mut admission = read_set.admission;
    let required_output_demand = admission.take_required_output_demand();
    let preimage_demand = installed_preimage_demand(admission.allowed_graph_contract().aftermath());
    let idempotency =
        bind_commit_idempotency(&admission, conditional_definition.as_ref(), idempotency);
    if let Some(denial) = mutation_proof
        .as_ref()
        .and_then(|proof| proof.refusal(&idempotency))
    {
        return terminal(WorthQueryApplicationCommitOutcome::Denied(denial));
    }
    if let Err(outcome) = validate_operation_currentness(&admission) {
        return terminal(outcome);
    }
    let current_product = match select_current_product(application, read_set.lease.product()) {
        Ok(product) => product,
        Err(outcome) => return terminal(outcome),
    };
    if let Some(outcome) = resolve_retained_idempotency(
        application,
        &mut admission,
        &current_product,
        idempotency,
        aftermath_causality.as_ref(),
    ) {
        return terminal(outcome);
    }
    if let Some(denial) = new_commit_refusal {
        return terminal(WorthQueryApplicationCommitOutcome::Denied(
            WorthQueryApplicationCommitDenial::workflow_settlement_denied(&denial),
        ));
    }
    let ordinary_basis = conditional_definition.is_none()
        && aftermath_causality.is_none()
        && elevation_currentness.is_none();
    if let Err(outcome) = validate_elevation_currentness(application, elevation_currentness) {
        return terminal(outcome);
    }
    if let Err(outcome) = validate_workflow_deadline(application, workflow_deadline) {
        return terminal(outcome);
    }
    let lease = match readmit_current_basis(
        application,
        &mut admission,
        read_set.lease,
        current_product,
        ordinary_basis,
    ) {
        Ok(lease) => lease,
        Err(outcome) => return terminal(outcome),
    };
    prepare_authorized_application_commit(
        application,
        WorthQueryCurrentApplicationCommit {
            admission,
            lease,
            provider: WorthQueryProviderAttemptPreparation {
                required_output_demand,
                installed_read_scopes: read_set.installed_read_scopes,
                facts: read_set.facts,
                consumed_outputs: read_set.consumed_outputs,
                application_effect_count: effects.len(),
                effects,
                emission_retained_bytes,
                emission_retained_bytes_ceiling,
                preimage_demand,
                conditional_definition,
                effect_posture,
                validator_work_admission,
                output_correspondence,
                retain_output_demand_observation,
                retain_client_observation,
                producer_required_invariants,
                output_currentness_facts,
                workflow_settlement,
            },
            idempotency,
            aftermath_causality,
            workflow_approval_authority,
        },
    )
}
