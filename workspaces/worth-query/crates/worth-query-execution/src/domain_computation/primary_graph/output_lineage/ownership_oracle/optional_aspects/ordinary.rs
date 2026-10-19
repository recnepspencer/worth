//! A real installed Program action, without any producer association.
use super::*;
use crate::facade::application_installation::WorthQueryProgramOwner;
use std::{num::NonZeroUsize, time::Duration};
use worth_query_decl::facade::application_query::ApplicationQueryParameterSet;

pub(super) fn commit<Schema: BirthSchema>(
    runtime: &installation::Application<Schema>,
    key: &str,
) -> WorthQueryApplicationCommitReceipt {
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
            let root = selected
                .resolve_entity(
                    &phase,
                    BodyKey::reference(),
                    key.to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let operation = runtime
                .installed_schema()
                .installed_operation(OrdinaryProduce::reference())
                .unwrap();
            let mut admission = selected
                .authorize_operation(
                    &phase,
                    &principal,
                    &root,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap();
            let query = runtime
                .installed_schema()
                .installed_query_binding::<PlanarReadBinding<Schema>>()
                .unwrap();
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &root);
            let plan = selected
                .admit_application_query(
                    &phase,
                    query.query(),
                    &access,
                    ApplicationQueryParameterSet::new(),
                    WorthQueryProductQueryControls::new(
                        NonZeroUsize::new(1).unwrap(),
                        NonZeroUsize::new(100_000).unwrap(),
                        &request,
                    ),
                )
                .unwrap();
            let (_, source) = runtime
                .execute_application_query_one_shot(&phase, plan)
                .unwrap()
                .into_admitted_disclosed()
                .into_output_demand_source()
                .into_single_source()
                .unwrap();
            let input = Input {
                scope_key: key.to_owned(),
                incomplete: false,
            };
            let source = runtime
                .bind_application_source_expectation::<OrdinaryBinding<Schema>, _>(
                    &mut admission,
                    source,
                    &input,
                )
                .unwrap();
            let bound = source.bound_source();
            let allocation = crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation;
            source
                .consume_into(runtime, &mut admission, allocation)
                .unwrap();
            let identities =
                ApplicationMutationIdentities::<Schema, OrdinaryBinding<Schema>>::encode(
                    &901, &input,
                )
                .unwrap();
            let executed = runtime
                .execute_mutation_handler::<OrdinaryBinding<Schema>>(
                    &phase,
                    &identities,
                    principal.principal_identity(),
                    admission,
                    allocation,
                )
                .unwrap();
            let HandlerResult::Completed(candidate) = executed else {
                panic!("ordinary birth handler must complete");
            };
            let (program, _) = candidate.into_parts();
            let outcome = runtime.commit_program_action_in_advancement(
                &phase,
                program,
                &identities,
                |identity| bound.bind_idempotency(identity),
                allocation,
            );
            let WorthQueryApplicationCommitOutcome::Committed(receipt) = outcome else {
                panic!("actual ordinary Program action must commit: {outcome:?}")
            };
            receipt
        })
        .unwrap()
}

pub(super) fn required_read_refuses(
    runtime: &installation::Application<installation::RequiredSchema>,
) {
    type Schema = installation::RequiredSchema;
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
            let root = selected
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
                .installed_operation(KeepProduce::reference())
                .unwrap();
            let mut admission = selected
                .authorize_operation(
                    &phase,
                    &principal,
                    &root,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap();
            let query = runtime
                .installed_schema()
                .installed_query_binding::<PlanarReadBinding<Schema>>()
                .unwrap();
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &root);
            let plan = selected
                .admit_application_query(
                    &phase,
                    query.query(),
                    &access,
                    ApplicationQueryParameterSet::new(),
                    WorthQueryProductQueryControls::new(
                        NonZeroUsize::new(1).unwrap(),
                        NonZeroUsize::new(100_000).unwrap(),
                        &request,
                    ),
                )
                .unwrap();
            let (_, source) = runtime
                .execute_application_query_one_shot(&phase, plan)
                .unwrap()
                .into_admitted_disclosed()
                .into_output_demand_source()
                .into_single_source()
                .unwrap();
            let input = KeepInput {
                scope_key: installation::ANCHOR.to_owned(),
                incomplete: false,
            };
            let pending = runtime
                .bind_application_source_expectation::<Binding<Schema, true>, _>(
                    &mut admission,
                    source,
                    &input,
                )
                .unwrap();
            let allocation = crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation;
            pending
                .consume_into(runtime, &mut admission, allocation)
                .unwrap();
            let projected = runtime.mutation_projection.project_admitted_operation(
                &phase,
                &admission,
                |reader, _| {
                    let node = reader
                        .prior_output::<OrdinaryBinding<Schema>, BornNode<Schema>>()
                        .expect("actual ordinary correspondence supplies the real live member");
                    assert_eq!(
                        reader.decision_field(&node, Note::reference()).unwrap(),
                        None
                    );
                },
                allocation,
            );
            let (_, snapshot, _) = projected.unwrap().into_parts();
            let denial = runtime
                .begin_projected_application_read_attempt(&phase, admission, snapshot, allocation)
                .and_then(|read| read.complete_projected_dependencies(&phase, allocation))
                .err()
                .expect("Required aspect omission cannot seal an admitted read");
            assert_eq!(
                denial.kind(),
                WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
                "actual Required-aspect read seal, after a genuine ordinary birth"
            );
        })
        .unwrap();
}
