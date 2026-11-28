pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::relation_laws::derived::court::*;

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
        matches!(denied,worth_query_host::facade::application_installation::WorthQueryApplicationPreparationDenial::
        Contributions(ref cause) if cause.kind()==WorthQueryPrimaryGraphInstallationDenialKind::CollectionOwnerMeaningMismatch),
        "actual exact installation refusal: {denied:?}"
    );
}

#[test]
fn proposed_derived_violation_and_count_use_the_authoritative_recipe() {
    let world = install::<Derived>(6, &[(0, 1)]);
    let before = read::committed(&world);
    assert_eq!(before.edges, std::collections::BTreeSet::from([(0, 1)]));
    let violation = outcome::denied(world.edit(101, &[(1, 2)]));
    assert_eq!(violation.decode_invariant_refusal::<
        worth_query_host::facade::application_invariants::WorthQueryEmptyInvariantRefusal>().unwrap(),());
    assert_eq!(read::committed(&world).edges, before.edges);
    // The lawful twin changes the same physical adjacency to another target.
    outcome::committed(world.edit(102, &[(1, 3)]));
    value(&world, 103, 1);
    // Physical support changes while the deduplicated endpoint set remains {3}.
    outcome::committed(world.replace_edges(104, &[(0, 4), (4, 3)], &[(1, 3)]));
    value(&world, 105, 1);
    assert_eq!(
        read::committed(&world).edges,
        std::collections::BTreeSet::from([(0, 1), (0, 4), (4, 3)])
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
    world.close();
    let dir = worth_query_testkit_document_retention::facade::directory_home::unique_directory(
        "relation-laws-derived",
    );
    let durable = super::super::compatibility::open_directory::<Derived>(
        dir,
        6,
        &[(0, 1), (0, 4), (4, 3)],
    )
    .unwrap_or_else(|refused| panic!("derived directory open refused: {refused:?}"));
    let observed = count(&durable, 108, false).unwrap();
    assert_eq!(observed.result().map(|result| result.count), Some(1));
    drop(observed);
    let dir = durable.checkpoint_and_close_for_reopen();
    let restored = super::super::compatibility::open_directory::<Derived>(
        dir,
        6,
        &[(0, 1), (0, 4), (4, 3)],
    )
    .unwrap_or_else(|refused| panic!("derived directory reopen refused: {refused:?}"));
    let observed = count(&restored, 109, false).unwrap();
    assert_eq!(
        observed.result().map(|result| result.count),
        Some(1),
        "the genuine reopened Query layout keeps its actual Native source issuer: {observed:?}"
    );
    drop(observed);
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
    drop((principal, scope));
    drop(restored);
}
