//! Authored law meaning changes identity and cannot silently reopen old installed truth.
use super::{installation::*, program::GraphProgram, schema::*};
use crate::document_retention_model::host::host_limits;
use worth_query_host::facade::{
    application_installation::*,
    declaration::{application_program::*, application_schema::*},
};
use worth_query_package_archive::facade::{
    decode_application_program_description, encode_application_program_description,
    WorthQueryApplicationProgramArchiveCompatibility, WorthQueryPackageArchiveLimits,
};

fn program<L: LawPolicy>() -> ValidatedApplicationProgram<GraphSchema<L>, GraphProgram<L>> {
    ApplicationProgramAuthoring::begin()
        .with_schema(&GraphSchema::<L>::declaration().unwrap())
        .validated_program()
        .unwrap()
}

#[test]
fn changing_only_the_relation_law_changes_the_same_named_program_revision() {
    changed_revision::<Unrestricted, Acyclic>();
    changed_revision::<Unrestricted, Symmetric>();
    changed_revision::<Unrestricted, DirectedUnique>();
    changed_revision::<Unrestricted, NormalizedUnique>();
    changed_revision::<Unrestricted, SymmetricDirectedUnique>();
    changed_revision::<CrossPartition, Isolated>();
}
fn changed_revision<Old: LawPolicy, L: LawPolicy>() {
    let old = program::<Old>();
    let new = program::<L>();
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let bytes = encode_application_program_description(&old, limits).unwrap();
    let description = decode_application_program_description(&bytes, limits).unwrap();
    assert_eq!(old.identity(), new.identity());
    assert_eq!(
        description.compatibility_with_validated(&old),
        WorthQueryApplicationProgramArchiveCompatibility::ExactRevision
    );
    assert_eq!(
        description.compatibility_with_validated(&new),
        WorthQueryApplicationProgramArchiveCompatibility::ChangedRevision
    );
    let new_bytes = encode_application_program_description(&new, limits).unwrap();
    let new_description = decode_application_program_description(&new_bytes, limits).unwrap();
    assert_eq!(
        new_description.compatibility_with_validated(&new),
        WorthQueryApplicationProgramArchiveCompatibility::ExactRevision
    );
    let facts: Vec<_> = new_description
        .facts()
        .iter()
        .filter(|fact| fact.family() == ApplicationSemanticFamily::RelationLaws)
        .collect();
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].subject(), "Precedes");
    assert!(!description
        .facts()
        .iter()
        .any(|fact| fact.family() == ApplicationSemanticFamily::RelationLaws));
    let unbound = ApplicationProgramAuthoring::<GraphSchema<Old>, GraphProgram<Old>>::begin()
        .validated_program()
        .unwrap();
    assert_eq!(
        old.revision(),
        unbound.revision(),
        "empty law collection preserves old revisions"
    );
}

#[test]
fn omitting_completed_schema_cannot_bypass_law_binding_at_installation() {
    unbound_refused::<Acyclic>();
    unbound_refused::<Symmetric>();
    unbound_refused::<DirectedUnique>();
    unbound_refused::<NormalizedUnique>();
    unbound_refused::<SymmetricDirectedUnique>();
    unbound_refused::<Isolated>();
}
fn unbound_refused<L: LawPolicy>() {
    let unbound = ApplicationProgramAuthoring::<GraphSchema<L>, GraphProgram<L>>::begin()
        .validated_program()
        .unwrap();
    assert!(!unbound.matches_relation_laws(GraphSchema::<L>::declaration().unwrap().erased()));
    let refusal = match install_program(unbound, 2, &[]) {
        Err(refusal) => refusal,
        Ok(_) => panic!("an unbound program cannot govern a law-bearing schema"),
    };
    let WorthQueryInMemoryApplicationDenial::Program(refusal) = refusal else {
        panic!("{refusal:?}")
    };
    assert!(matches!(refusal.support_denial(), Some(worth_query_host::facade::domain::WorthQueryProgramSupportDenial::RelationLawMeaningMismatch { .. })), "{refusal:?}");
}

fn capture<L: LawPolicy>(world: &World<L>) -> WorthQueryApplicationCheckpoint {
    let runtime = world.application.runtime();
    runtime
        .with_host_advancement(|phase| {
            runtime
                .capture_application_checkpoint(
                    &phase,
                    WorthQueryCheckpointCapturePolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap()
}

fn restore<L: LawPolicy>(
    checkpoint: WorthQueryApplicationCheckpoint,
) -> Result<
    WorthQueryProgramApplicationRuntime<GraphSchema<L>, GraphProgram<L>>,
    WorthQueryInMemoryApplicationDenial,
> {
    in_memory_program_from_checkpoint(
        program::<L>(),
        GraphSchema::<L>::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        host_limits(),
        checkpoint,
    )
}

#[test]
fn an_installed_checkpoint_cannot_silently_acquire_a_new_relation_law() {
    checkpoint_laws::<Unrestricted, Acyclic>();
    checkpoint_laws::<Unrestricted, Symmetric>();
    checkpoint_laws::<Unrestricted, DirectedUnique>();
    checkpoint_laws::<Unrestricted, NormalizedUnique>();
    checkpoint_laws::<Unrestricted, SymmetricDirectedUnique>();
    checkpoint_laws::<CrossPartition, Isolated>();
}
fn checkpoint_laws<Old: LawPolicy, L: LawPolicy>() {
    // This graph is lawful under either policy: refusal must concern installed meaning.
    let old = install::<Old>(3, &[]);
    let unchanged_checkpoint = capture(&old);
    let changed_checkpoint = capture(&old);
    old.close();
    let mut unchanged = restore::<Old>(unchanged_checkpoint).unwrap();
    assert!(unchanged.close_live_delivery().owner_terminal());
    unchanged.close_conditional_runtime().unwrap();
    let refusal = match restore::<L>(changed_checkpoint) {
        Err(refusal) => refusal,
        Ok(_) => panic!("old installed meaning cannot become a different schema/program"),
    };
    let WorthQueryInMemoryApplicationDenial::Graph(refusal) = refusal else {
        panic!("{refusal:?}")
    };
    assert_eq!(refusal.kind(), worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected);

    let strengthened = install::<L>(3, &[]);
    let same_meaning = capture(&strengthened);
    let removed = capture(&strengthened);
    strengthened.close();
    let mut unchanged_law = restore::<L>(same_meaning).unwrap();
    assert!(unchanged_law.close_live_delivery().owner_terminal());
    unchanged_law.close_conditional_runtime().unwrap();
    assert!(
        matches!(
            restore::<Old>(removed),
            Err(WorthQueryInMemoryApplicationDenial::Graph(_))
        ),
        "removing a law is also changed installed meaning"
    );
}
