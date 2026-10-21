use super::*;

pub(super) fn change_input(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &application_installation::WorthQueryProgramApplicationRuntime<
        TemporalHostSchema,
        TemporalInstallationProgram,
    >,
    invariant: &primary_graph::WorthQueryApplicationInvariantProjectionAuthority<
        TemporalHostSchema,
    >,
    branch: product::WorthQueryProductBranch,
    input: &str,
) -> primary_graph::WorthQueryApplicationCommitOutcome {
    change_input_with_commit_probe(
        request_execution,
        application,
        invariant,
        branch,
        input,
        || (),
    )
}

pub(super) fn change_input_with_commit_probe<Guard>(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &application_installation::WorthQueryProgramApplicationRuntime<
        TemporalHostSchema,
        TemporalInstallationProgram,
    >,
    invariant: &primary_graph::WorthQueryApplicationInvariantProjectionAuthority<
        TemporalHostSchema,
    >,
    branch: product::WorthQueryProductBranch,
    input: &str,
    arm: impl FnOnce() -> Guard,
) -> primary_graph::WorthQueryApplicationCommitOutcome {
    let schema = application.installed_schema();
    let principal_binding = schema
        .principal_binding(TemporalPrincipalBinding::reference())
        .unwrap();
    let authentication = admit_identity_adapter(schema);
    let request = request_scope();
    let external = block_on(authentication.authenticate((), &request)).unwrap();
    let selected = application
        .with_host_advancement(|host_phase| {
            application.on_branch(branch).select(&host_phase).unwrap()
        })
        .expect("installed host call opens");
    let principal = application
        .with_host_advancement(|host_phase| {
            selected
                .resolve_authenticated_principal(
                    &host_phase,
                    &principal_binding,
                    &external,
                    &request,
                    primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let intent = application
        .with_host_advancement(|host_phase| {
            selected
                .resolve_entity(
                    &host_phase,
                    IntentIdentityField::reference(),
                    "intent-1".to_string(),
                    &request,
                    primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let operation = schema
        .installed_operation(AmendTemporal::reference())
        .unwrap();
    let admission = application
        .with_host_advancement(|host_phase| {
            selected
                .authorize_operation(
                    &host_phase,
                    &principal,
                    &intent,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let (_, projection, _) = application
        .with_host_advancement(|host_phase| {
            invariant
        .project_admitted_operation(
            &host_phase, &admission,
            |reader, scope| {
                reader
                    .decision_field(scope, IntentRevisionField::reference())
                    .unwrap();
                reader
                    .decision_field(scope, IntentLifecycleField::reference())
                    .unwrap();
                reader
                    .decision_field(scope, IntentGateField::reference())
                    .unwrap();
                reader
                    .decision_field(scope, IntentDueField::reference())
                    .unwrap();
                reader
                    .decision_field(scope, IntentInputField::reference())
                    .unwrap();
            },
            worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .into_parts()
        })
        .expect("installed host call opens");
    let mut effects = application
        .with_host_advancement(|phase| {
            let reads = application
                .begin_projected_application_read_attempt(
                    &phase,
                    admission,
                    projection,
                    worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            reads
                .complete_projected_dependencies(
                    &phase,
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .begin_effect_program()
        })
        .expect("installed host call opens");
    let intent = effects.existing_entity(&intent).unwrap();
    effects
        .write_field(&intent, IntentRevisionField::reference(), 2_u64)
        .unwrap();
    effects
        .write_field(
            &intent,
            IntentLifecycleField::reference(),
            "active".to_string(),
        )
        .unwrap();
    effects
        .write_field(&intent, IntentDueField::reference(), 11_u64)
        .unwrap();
    effects
        .write_field(&intent, IntentInputField::reference(), input.to_string())
        .unwrap();
    effects
        .write_field(&intent, IntentGateField::reference(), "ready".to_string())
        .unwrap();
    let admitted = product::WorthQueryAdmittedChange::new(
        effects.finish().unwrap(),
        primary_graph::WorthQueryApplicationIdempotencyBinding::for_host_commit::<
            TemporalHostSchema,
            AmendTemporal,
            _,
            _,
        >("source-change", "source-change")
        .unwrap(),
    );
    let _commit_probe = arm();
    application
        .on_branch(branch)
        .transaction()
        .apply(admitted)
        .commit_for_program(
            request_execution,
            application
                .admit_program_operation::<AmendTemporal>()
                .unwrap(),
        )
        .unwrap()
}
