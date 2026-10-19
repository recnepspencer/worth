//! A completed real operation read cannot commit past a competing source publication.
use super::*;
use installation::OptionalSchema as Schema;
use std::time::Duration;
#[test]
fn absent_aspect_read_stales_when_other_field_materializes_before_commit() {
    let runtime = installation::install::<Schema>(false);
    let request = primary_graph::tests::fixture::live_scope();
    let external = primary_graph::tests::fixture::authenticate_external(
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
            let source = selected
                .resolve_entity(
                    &phase,
                    BodyKey::reference(),
                    installation::ANCHOR.to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let operation = runtime
                .installed_schema()
                .installed_operation(Produce::reference())
                .unwrap();
            let admitted = selected
                .authorize_operation(
                    &phase,
                    &principal,
                    &source,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap();
            let (_, snapshot, _) = runtime
                .mutation_projection
                .project_admitted_operation(
                    &phase,
                    &admitted,
                    |reader, root| {
                        assert_eq!(
                            reader
                                .decision_field(root, SourceSelection::reference())
                                .unwrap(),
                            None
                        );
                        let called = std::cell::Cell::new(false);
                        assert_eq!(
                            reader
                                .decision_field_with_predecode_admission(
                                    root,
                                    SourceSelection::reference(),
                                    |_, _| {
                                        called.set(true);
                                        Ok::<_, ()>(())
                                    },
                                    &|| Ok(())
                                )
                                .unwrap()
                                .unwrap(),
                            None
                        );
                        assert!(!called.get());
                    },
                    crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts();
            let reads = runtime
                .begin_projected_application_read_attempt(
                    &phase,
                    admitted,
                    snapshot,
                    crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .complete_projected_dependencies(
                    &phase,
                    crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            let program = reads.begin_effect_program().finish().unwrap();
            // Close the read before entering the distinct actual native publication.
            // The original read/program and its pin remain retained for comparison.
            super::retained_recovery::materialize_source_in_phase(&runtime, &phase);
            let input = Input {
                scope_key: installation::ANCHOR.to_owned(),
                incomplete: false,
            };
            let key = 800_u64;
            let identities =
                ApplicationMutationIdentities::<Schema, Binding<Schema, false>>::encode(
                    &key, &input,
                )
                .unwrap();
            let execution = phase
                .execution_request_for(&runtime.product_runtime)
                .unwrap();
            // This is the real installed conditional-program entry, which checks the
            // current occurrence/program and registered operation before fact comparison.
            let outcome = runtime.compare_and_commit_application_for_program_output_producer(
                &execution,
                &phase,
                program,
                WorthQueryApplicationIdempotencyBinding::for_mutation_identities(&identities),
                Some((
                    runtime.installed_program().identity(),
                    runtime.installed_program().revision(),
                )),
            );
            let WorthQueryApplicationCommitOutcome::Stale(stale) = outcome else {
                panic!("changed absence must stale before effects: {outcome:?}")
            };
            assert!(stale.stale_fact_count() > 0);
        })
        .unwrap();
}
