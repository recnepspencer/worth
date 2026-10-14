//! Even lawful operation admission cannot replace an authored body's values.
use super::{
    authenticate_operator, binding::BodyBinding, install_with_projection, read_retention,
    request_scope, schema::*,
};
use worth_query_host::facade::{
    application_installation::WorthQueryProgramOwner,
    declaration::application_operation::ApplicationMutationIdentities,
    primary_graph::{
        WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitOutcome,
        WorthQueryPrincipalResolutionMode,
    },
    runtime::ExecutionAllocationPolicy,
};

#[test]
fn authored_body_advanced_effects_cannot_replace_native_execution() {
    let (app, projection) = install_with_projection::<8>();
    let runtime = app.runtime();
    let scope = request_scope();
    let external = authenticate_operator(runtime.installed_schema(), &scope);
    let program = runtime
        .with_application_advancement(&scope, |phase| {
            let selected = runtime
                .on_branch(app.current_world())
                .select(&phase)
                .unwrap();
            let binding = runtime
                .installed_schema()
                .principal_binding(DocumentPrincipalBinding::reference())
                .unwrap();
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &binding,
                    &external,
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let document = selected
                .resolve_entity(
                    &phase,
                    DocumentIdentityField::reference(),
                    super::DOCUMENT_IDENTITY.to_owned(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let operation = runtime
                .installed_schema()
                .installed_operation(SetRetention::reference())
                .unwrap();
            let admission = selected
                .authorize_operation(
                    &phase,
                    &principal,
                    &document,
                    &operation,
                    Default::default(),
                    &scope,
                )
                .unwrap();
            let (_, snapshot, _) = projection
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, target| {
                        assert_eq!(
                            reader
                                .decision_field(target, DocumentRetentionField::reference())
                                .unwrap(),
                            Some(7)
                        );
                    },
                    ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts();
            let reads = runtime
                .begin_projected_application_read_attempt(
                    &phase,
                    admission,
                    snapshot,
                    ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            let mut effects = reads
                .complete_projected_dependencies(
                    &phase,
                    ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .begin_effect_program();
            let target = effects.existing_entity(&document).unwrap();
            // Valid type, same admitted operation and footprint, wrong body value.
            effects
                .write_field(&target, DocumentRetentionField::reference(), 99)
                .unwrap();
            effects.finish().unwrap()
        })
        .unwrap();
    let input = SetRetentionInput {
        identity: super::DOCUMENT_IDENTITY.to_owned(),
        retention_days: 99,
        ..Default::default()
    };
    let key = 87_u64;
    let identities =
        ApplicationMutationIdentities::<DocumentRetentionSchema<8>, BodyBinding<8>>::encode(
            &key, &input,
        )
        .unwrap();
    let outcome = app.compare_and_commit_program_action(
        program,
        &identities,
        std::convert::identity,
        ExecutionAllocationPolicy::SystemAllocation,
    );
    match outcome {
        WorthQueryApplicationCommitOutcome::Denied(denial) => assert_eq!(
            denial.kind(),
            WorthQueryApplicationCommitDenialKind::OperationBodyExecutionRequired,
        ),
        other => panic!("advanced effects must not override the body: {other:?}"),
    }
    assert_eq!(read_retention(&app), 7);
    // The denied request did not consume the key or damage lawful body execution.
    assert!(matches!(
        super::invoke(&app, key),
        super::Outcome::Committed { .. }
    ));
    assert_eq!(read_retention(&app), 8);
}
