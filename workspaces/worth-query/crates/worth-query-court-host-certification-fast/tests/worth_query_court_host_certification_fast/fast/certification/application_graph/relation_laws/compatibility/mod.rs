pub(crate) use worth_query_worlds::facade::certification::relation_laws::compatibility::*;

#[test]
fn changing_only_the_relation_law_changes_the_same_named_program_revision() {
    changed_revision::<Unrestricted, Acyclic>();
    changed_revision::<Unrestricted, Symmetric>();
    changed_revision::<Unrestricted, DirectedUnique>();
    changed_revision::<Unrestricted, NormalizedUnique>();
    changed_revision::<Unrestricted, SymmetricDirectedUnique>();
    changed_revision::<CrossPartition, Isolated>();
    changed_revision::<Unrestricted, MinimumTwo>();
    changed_revision::<Unrestricted, Orbit>();
}

#[test]
fn omitting_completed_schema_cannot_bypass_law_binding_at_installation() {
    unbound_refused::<Acyclic>();
    unbound_refused::<Symmetric>();
    unbound_refused::<DirectedUnique>();
    unbound_refused::<NormalizedUnique>();
    unbound_refused::<SymmetricDirectedUnique>();
    unbound_refused::<Isolated>();
    unbound_refused::<MinimumTwo>();
    unbound_refused::<Orbit>();
}

#[test]
fn an_installed_checkpoint_cannot_silently_acquire_a_new_relation_law() {
    checkpoint_laws::<Unrestricted, Acyclic>(&[]);
    checkpoint_laws::<Unrestricted, Symmetric>(&[]);
    checkpoint_laws::<Unrestricted, DirectedUnique>(&[]);
    checkpoint_laws::<Unrestricted, NormalizedUnique>(&[]);
    checkpoint_laws::<Unrestricted, SymmetricDirectedUnique>(&[]);
    checkpoint_laws::<CrossPartition, Isolated>(&[]);
    checkpoint_laws::<Unrestricted, MinimumTwo>(&[(0, 1), (1, 2), (2, 0)]);
    checkpoint_laws::<Unrestricted, Orbit>(&[(0, 1), (1, 2), (2, 0)]);
}

#[test]
fn reordered_duplicate_minimum_kinds_preserve_schema_and_program_meaning() {
    #[derive(Clone, Debug)]
    struct RepeatedKinds;
    impl LawPolicy for RepeatedKinds {
        const INTEGRITY: ApplicationRelationIntegrity =
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
                .reaches_at_least(2, &["Node", "Node"]);
    }
    let plain = program::<MinimumTwo>();
    let repeated = program::<RepeatedKinds>();
    assert_eq!(plain.revision(), repeated.revision());
    assert_eq!(
        GraphSchema::<MinimumTwo>::declaration().unwrap().identity(),
        GraphSchema::<RepeatedKinds>::declaration()
            .unwrap()
            .identity()
    );
    assert!(plain.matches_relation_laws(
        GraphSchema::<RepeatedKinds>::declaration()
            .unwrap()
            .erased()
    ));
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let archive = decode_application_program_description(
        &encode_application_program_description(&plain, limits).unwrap(),
        limits,
    )
    .unwrap();
    assert_eq!(
        archive.compatibility_with_validated(&repeated),
        WorthQueryApplicationProgramArchiveCompatibility::ExactRevision
    );
}
