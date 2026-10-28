//! Genuine facade publication judges proposed sources and maintains disposable counts.
use super::super::{
    installation::{install, World},
    outcome, read,
};
use super::*;
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use std::marker::PhantomData;
#[test]
fn descriptive_collection_association_cannot_install_a_custom_computation_as_source_authority() {
    use super::super::program::GraphProgram;
    let schema = GraphSchema::<CustomDerivedOwner>::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<
        GraphSchema<CustomDerivedOwner>,
        GraphProgram<CustomDerivedOwner>,
    >::begin()
    .with_schema(&schema)
    .validated_program()
    .unwrap();
    let denied = super::super::installation::install_program(program, 3, &[(0, 1)], 0, &[])
        .err()
        .expect("the real setup/seal must reject the custom owner");
    assert!(
        matches!(denied,worth_query_host::facade::application_installation::WorthQueryInMemoryApplicationDenial::
        Contributions(ref cause) if cause.kind()==WorthQueryPrimaryGraphInstallationDenialKind::CollectionOwnerMeaningMismatch),
        "actual exact installation refusal: {denied:?}"
    );
}
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt,
        WorthQueryApplicationRequestMutationDenial,
    },
    application_installation::WorthQueryProgramApplicationRuntime,
    runtime::ExecutionAllocationPolicy,
};

fn count(
    application: &WorthQueryProgramApplicationRuntime<
        GraphSchema<Derived>,
        super::super::program::GraphProgram<Derived>,
    >,
    command: u64,
    bad_reference: bool,
) -> Result<
    WorthQueryApplicationMutationOutcome<super::super::operation::GraphDenial, operation::Counted>,
    WorthQueryApplicationRequestMutationDenial,
> {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    application
        .runtime()
        .request(&principal, &scope)
        .on_branch(application.current_world())
        .advanced()
        .mutate(operation::Intent::<Derived> {
            input: operation::Input {
                root: 0,
                bad_reference,
            },
            marker: PhantomData,
        })
        .without_source()
        .idempotency(&command)
        .execute_in_program(application, ExecutionAllocationPolicy::SystemAllocation)
}
fn value(world: &World<Derived>, command: u64, expected: u64) {
    let outcome = count(&world.application, command, false).unwrap();
    assert_eq!(
        outcome.result().map(|result| result.count),
        Some(expected),
        "{outcome:?}"
    );
}
#[test]
fn proposed_derived_violation_and_managed_count_share_the_real_recipe_without_cached_authority() {
    use WorthQueryRuleReadableCollectionLifecycleForTest as State;
    let world = install::<Derived>(6, &[(0, 1)]);
    assert!(world
        .application
        .runtime()
        .rule_readable_collection_lifecycle_for_test()
        .is_empty());
    value(&world, 100, 0);
    assert_eq!(
        world
            .application
            .runtime()
            .rule_readable_collection_lifecycle_for_test(),
        [State::Warm]
    );
    let before = read::committed(&world);
    assert_eq!(before.edges, std::collections::BTreeSet::from([(0, 1)]));
    let violation = outcome::denied(world.edit(101, &[(1, 2)]));
    assert_eq!(violation.decode_invariant_refusal::<
        worth_query_host::facade::application_invariants::WorthQueryEmptyInvariantRefusal>().unwrap(),());
    assert_eq!(read::committed(&world).edges, before.edges);
    // The lawful twin changes the same physical adjacency to another target.
    outcome::committed(world.edit(102, &[(1, 3)]));
    assert_eq!(
        world
            .application
            .runtime()
            .rule_readable_collection_lifecycle_for_test(),
        [State::Dirty],
        "the formerly empty second-leg adjacency must invalidate the actual registered entry"
    );
    value(&world, 103, 1);
    assert_eq!(
        world
            .application
            .runtime()
            .rule_readable_collection_lifecycle_for_test(),
        [State::Warm]
    );
    // Physical support changes while the deduplicated endpoint set remains {3}.
    outcome::committed(world.replace_edges(104, &[(0, 4), (4, 3)], &[(1, 3)]));
    assert_eq!(
        world
            .application
            .runtime()
            .rule_readable_collection_lifecycle_for_test(),
        [State::Dirty]
    );
    value(&world, 105, 1);
    assert_eq!(
        read::committed(&world).edges,
        std::collections::BTreeSet::from([(0, 1), (0, 4), (4, 3)])
    );
    world
        .application
        .runtime()
        .discard_rule_readable_collections();
    assert!(world
        .application
        .runtime()
        .rule_readable_collection_lifecycle_for_test()
        .is_empty());
    value(&world, 106, 1);
    assert_eq!(
        world
            .application
            .runtime()
            .rule_readable_collection_lifecycle_for_test(),
        [State::Warm]
    );
    let before_failed_read = read::committed(&world).edges;
    let refused = count(&world.application, 107, true).unwrap_err();
    assert!(
        matches!(refused,
            WorthQueryApplicationRequestMutationDenial::Handler(
                MutationHandlerExecutionDenial::Attempt(ref cause))
                if cause.kind() == WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact),
        "a swallowed source failure cannot seal a result: {refused:?}"
    );
    assert_eq!(read::committed(&world).edges, before_failed_read);
    value(&world, 108, 1);
    let checkpoint = super::super::compatibility::capture(&world);
    world.close();
    let mut restored = super::super::compatibility::restore::<Derived>(checkpoint).unwrap();
    let observed = count(&restored, 109, false).unwrap();
    assert_eq!(
        observed.result().map(|result| result.count),
        Some(1),
        "the genuine reopened Query layout keeps its actual Native source issuer: {observed:?}"
    );
    let scope = request_scope();
    let principal = authenticate_operator(restored.installed_schema(), &scope);
    let command = 110_u64;
    let refused = restored
        .runtime()
        .request(&principal, &scope)
        .on_branch(restored.current_world())
        .advanced()
        .mutate(super::super::operation::Edit::<Derived> {
            input: super::super::operation::EdgeEdit {
                serial: command,
                edges: vec![(1, 2)],
                removals: vec![],
            },
            marker: PhantomData,
        })
        .without_source()
        .idempotency(&command)
        .execute_in_program(&restored, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert_eq!(outcome::denied(refused).decode_invariant_refusal::<
        worth_query_host::facade::application_invariants::WorthQueryEmptyInvariantRefusal>().unwrap(),(),
        "reopened rules judge the genuinely proposed 0→2 source, not the lawful committed {{3}}");
    assert!(restored.close_live_delivery().owner_terminal());
    restored.close_conditional_runtime().unwrap();
}
