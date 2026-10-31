//! A physical component fixture, not an admitted application home.
pub(super) fn initialize(
    root: &std::path::Path,
) -> worth_store::physical_runtime::ServingPhysicalRuntime {
    use worth_store::physical_runtime::{
        OrderedJournalRegistry, PhysicalRecordInitialization, RELATIONAL_BRANCH_TRANSITIONS,
        RELATIONAL_CANONICAL_COMMITS,
    };
    use worth_store_testkit_store::physical_record_journeys as fixture;
    let (format, placement, access) = fixture::configuration();
    let media = fixture::media(root);
    let durability = fixture::durability(&media);
    let registry = OrderedJournalRegistry::declare(
        3,
        &[RELATIONAL_CANONICAL_COMMITS, RELATIONAL_BRANCH_TRANSITIONS],
        &["home.checkpoint"],
    )
    .expect("the component fixture declares only its native journals and member anchor");
    fixture::success(
        media.initialize_record_store(
            PhysicalRecordInitialization::new(format, placement, access, durability)
                .with_journal_registry(registry),
        ),
    )
}
