//! Runtime-retained projection authority remains confined to its owning graph.
use super::checkpoint_transition::configuration;
use super::*;

#[test]
fn runtime_projection_authority_refuses_another_runtime_and_accepts_its_own() {
    let install = || {
        application_installation::program(
            validated_program(),
            TemporalHostSchema::declaration().unwrap(),
            configuration(),
            checkpoint::checkpoint_limits(),
        )
        .initial_state(|graph, installed| {
            let principal = installed
                .principal_binding(TemporalPrincipalBinding::reference())
                .unwrap();
            seed_graph(graph, &principal, "authority-owner", 0, 1, true);
            Ok(())
        })
        .open(application_installation::ApplicationHome::memory())
        .unwrap()
    };
    let first = install();
    let second = install();
    let foreign = first.runtime().retain_invariant_projection_authority();
    let own = second.runtime().retain_invariant_projection_authority();
    let schema = second.installed_schema();
    let binding = schema
        .principal_binding(TemporalPrincipalBinding::reference())
        .unwrap();
    let authentication = admit_identity_adapter(schema);
    let request = request_scope();
    let external = block_on(authentication.authenticate((), &request)).unwrap();
    let selected = second.on_branch(second.current_world()).select().unwrap();
    let principal = selected
        .resolve_authenticated_principal(
            &binding,
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
    let mut foreign_called = false;
    let denial = foreign
        .project_admitted_operation(&admission, |_, _| {
            foreign_called = true;
        })
        .err()
        .expect("authority from the first runtime cannot project the second graph");
    assert_eq!(
        denial.kind(),
        primary_graph::WorthQueryOperationProjectionDenialKind::Authorization(
            primary_graph::WorthQueryOperationAuthorizationDenialKind::ForeignRuntime
        )
    );
    assert!(
        !foreign_called,
        "foreign authority must be denied before graph access"
    );
    let completed = own
        .project_admitted_operation(&admission, |reader, scope| {
            reader
                .decision_field(scope, IntentGateField::reference())
                .unwrap()
        })
        .expect("authority retained from the second runtime may project its own graph");
    assert_eq!(completed.output(), &Some("authority-owner".to_string()));
}
