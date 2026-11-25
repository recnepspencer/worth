//! Authored law meaning changes identity and cannot silently reopen old installed truth.
pub use super::{installation::*, program::GraphProgram, schema::*};
pub use worth_query_host::facade::{
    application_installation::*,
    declaration::{application_program::*, application_schema::*},
};
pub use worth_query_package_archive::facade::{
    decode_application_program_description, encode_application_program_description,
    WorthQueryApplicationProgramArchiveCompatibility, WorthQueryPackageArchiveLimits,
};
pub use worth_query_testkit_document_retention::facade::document_retention_model::host::host_limits;

pub fn program<L: LawPolicy>() -> ValidatedApplicationProgram<GraphSchema<L>, GraphProgram<L>> {
    ApplicationProgramAuthoring::begin()
        .with_schema(&GraphSchema::<L>::declaration().unwrap())
        .validated_program()
        .unwrap()
}

pub fn changed_revision<Old: LawPolicy, L: LawPolicy>() {
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

pub fn unbound_refused<L: LawPolicy>() {
    let unbound = ApplicationProgramAuthoring::<GraphSchema<L>, GraphProgram<L>>::begin()
        .validated_program()
        .unwrap();
    assert!(!unbound.matches_relation_laws(GraphSchema::<L>::declaration().unwrap().erased()));
    let refusal = match install_program(unbound, 2, &[], 0, &[]) {
        Err(refusal) => refusal,
        Ok(_) => panic!("an unbound program cannot govern a law-bearing schema"),
    };
    let WorthQueryApplicationPreparationDenial::Program(refusal) = refusal else {
        panic!("{refusal:?}")
    };
    assert!(matches!(refusal.support_denial(), Some(worth_query_host::facade::domain::WorthQueryProgramSupportDenial::RelationLawMeaningMismatch { .. })), "{refusal:?}");
}

pub fn open_directory<L: LawPolicy>(
    directory: std::path::PathBuf,
    nodes: u64,
    edges: &[(u64, u64)],
) -> Result<
    worth_query_testkit_document_retention::facade::directory_home::DirectoryHomeGuard<
        GraphSchema<L>,
        GraphProgram<L>,
    >,
    worth_query_host::facade::WorthQueryApplicationHomeOpenRefused<GraphSchema<L>, GraphProgram<L>>,
> {
    let home =
        worth_query_testkit_document_retention::facade::directory_home::with_directory_bootstrap(
            |phase| {
                worth_query_host::facade::WorthQueryApplicationHomeRequest::new(
                    worth_query_host::facade::WorthQueryApplicationHomeLocation::Directory(
                        directory.clone(),
                    ),
                    program::<L>(),
                    WorthQueryApplicationProgramRoster::new(),
                    GraphSchema::<L>::declaration().unwrap(),
                    ((),),
                    ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111")
                        .unwrap(),
                    host_limits(
                        worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
                    ),
                    |_, graph, installed| seed_graph::<L>(graph, installed, nodes, edges, 0, &[]),
                )
                .open_in_advancement(phase)
            },
        )?;
    Ok(
        worth_query_testkit_document_retention::facade::directory_home::DirectoryHomeGuard::wrap(
            home, directory,
        ),
    )
}

pub fn checkpoint_laws<Old: LawPolicy, L: LawPolicy>(edges: &[(u64, u64)]) {
    use worth_query_testkit_document_retention::facade::directory_home::{
        unique_directory, ClosedDirectorySnapshot,
    };
    // This graph is lawful under either policy: refusal must concern installed meaning.
    let old = open_directory::<Old>(unique_directory("relation-laws-old"), 3, edges)
        .unwrap_or_else(|refused| panic!("old law directory open refused: {refused:?}"));
    let old_dir = old.checkpoint_and_close_for_reopen();
    let old_snapshot = ClosedDirectorySnapshot::capture(&old_dir, "relation-laws-changed");
    let unchanged = open_directory::<Old>(old_dir.clone(), 3, edges)
        .unwrap_or_else(|refused| panic!("unchanged law directory reopen refused: {refused:?}"));
    let old_dir = unchanged.close_for_reopen();
    old_snapshot.restore_into(&old_dir);
    let refused = open_directory::<L>(old_dir.clone(), 3, edges)
        .err()
        .expect("old installed meaning cannot become a different schema/program");
    assert!(
        !format!("{refused:?}").contains("MemberPayloadRejected"),
        "changed law reopen must fail on installed meaning, not media identity: {refused:?}"
    );
    let _ = std::fs::remove_dir_all(&old_dir);

    let strengthened = open_directory::<L>(unique_directory("relation-laws-new"), 3, edges)
        .unwrap_or_else(|refused| panic!("strengthened law directory open refused: {refused:?}"));
    let strengthened_dir = strengthened.checkpoint_and_close_for_reopen();
    let strengthened_snapshot =
        ClosedDirectorySnapshot::capture(&strengthened_dir, "relation-laws-removed");
    let unchanged_law = open_directory::<L>(strengthened_dir.clone(), 3, edges)
        .unwrap_or_else(|refused| panic!("same law directory reopen refused: {refused:?}"));
    let strengthened_dir = unchanged_law.close_for_reopen();
    strengthened_snapshot.restore_into(&strengthened_dir);
    let removed_refused = open_directory::<Old>(strengthened_dir.clone(), 3, edges)
        .err()
        .expect("removing a law is also changed installed meaning");
    assert!(
        !format!("{removed_refused:?}").contains("MemberPayloadRejected"),
        "removed law reopen must fail on installed meaning, not media identity: {removed_refused:?}"
    );
    let _ = std::fs::remove_dir_all(&strengthened_dir);
}
