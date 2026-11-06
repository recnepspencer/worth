//! A genuine handler's complete initialization cannot overwrite a newer parent.
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::field_patch_cases::*;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::own_effect::MaterializingSchema as Schema;
use std::time::Duration;
#[test]
fn sealed_absent_parent_initialization_stales_before_any_native_effect() {
    let runtime = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::install::<Schema>(false);
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    let external = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authentication::authenticate_external(
        runtime.installed_schema(),
        "alice",
        Duration::from_secs(60),
        &request,
    );
    runtime
        .with_application_advancement(&request, |phase| {
            let selected = runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap();
            let principal_binding = runtime
                .installed_schema()
                .principal_binding(ConsumerPrincipalBinding::reference())
                .unwrap();
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &principal_binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let root = selected
                .resolve_entity(
                    &phase,
                    BodyKey::reference(),
                    crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::ANCHOR.to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let operation = runtime
                .installed_schema()
                .installed_operation(Produce::reference())
                .unwrap();
            let admission = selected
                .authorize_operation(
                    &phase,
                    &principal,
                    &root,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap();
            let input = Input {
                scope_key: crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::ANCHOR.to_owned(),
                incomplete: false,
            };
            let allocation = worth_execution::ExecutionAllocationPolicy::SystemAllocation;
            let identities =
                ApplicationMutationIdentities::<Schema, Binding<Schema, false>>::encode(
                    &902, &input,
                )
                .unwrap();
            let executed = runtime
                .execute_mutation_handler::<Binding<Schema, false>>(
                    &phase,
                    &identities,
                    principal.principal_identity(),
                    admission,
                    allocation,
                )
                .unwrap();
            let HandlerResult::Completed(candidate) = executed else {
                panic!("real complete initializer must prepare");
            };
            let (program, _) = candidate.into_parts();
            crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::retained_recovery::materialize_source_in_phase(&runtime, &phase);
            let execution = phase
                .execution_request_for_for_test((*runtime.runtime_for_test()).product_runtime_for_test())
                .unwrap();
            let outcome = runtime.compare_and_commit_application_for_program_output_producer(
                &execution,
                &phase,
                program,
                WorthQueryApplicationIdempotencyBinding::for_mutation_identities(&identities),
                Some((
                    runtime.installed_program().identity(),
                    runtime.installed_program().revision(),
                )),
                None,
            );
            let WorthQueryApplicationCommitOutcome::Stale(stale) = outcome else {
                panic!("competing parent materialization must stale before effects: {outcome:?}");
            };
            assert!(stale.stale_fact_count() > 0);
        })
        .unwrap();
    assert_eq!(
        source_values(&runtime),
        (Some("complete-required-content".to_owned()), None),
        "the initializer's replacement Content/Selection never commits over the competing parent"
    );
}
