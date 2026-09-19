use super::*;

pub(super) fn change_input(
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
    let schema = application.installed_schema();
    let principal_binding = schema
        .principal_binding(TemporalPrincipalBinding::reference())
        .unwrap();
    let authentication = admit_identity_adapter(schema);
    let request = request_scope();
    let external = block_on(authentication.authenticate((), &request)).unwrap();
    let selected = application.on_branch(branch).select().unwrap();
    let principal = selected
        .resolve_authenticated_principal(
            &principal_binding,
            &external,
            &request,
            primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let intent = selected
        .resolve_entity(
            IntentIdentityField::reference(),
            "intent-1".to_string(),
            &request,
            primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let operation = schema
        .installed_operation(AmendTemporal::reference())
        .unwrap();
    let admission = selected
        .authorize_operation(
            &principal,
            &intent,
            &operation,
            Default::default(),
            &request,
        )
        .unwrap();
    let (_, projection, _) = invariant
        .project_admitted_operation(&admission, |reader, scope| {
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
        })
        .unwrap()
        .into_parts();
    let reads = application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap();
    let mut effects = reads
        .complete_projected_dependencies()
        .unwrap()
        .begin_effect_program();
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
        primary_graph::WorthQueryApplicationIdempotencyBinding::new([0x7A; 32], [0xA7; 32]),
    );
    application
        .on_branch(branch)
        .transaction()
        .apply(admitted)
        .commit_for_program(
            application
                .admit_program_operation::<AmendTemporal>()
                .unwrap(),
        )
        .unwrap()
}
