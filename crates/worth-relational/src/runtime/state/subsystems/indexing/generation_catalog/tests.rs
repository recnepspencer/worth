use super::*;
use crate::indexes::data::{
    DerivedIndexApplicability, DerivedIndexEntries, DerivedIndexPublicationStatus,
};
use proptest::prelude::*;

// These are catalog ordering inputs, not forged runtime admission or storage truth.
fn generation(
    id: u64,
    index: u64,
    branch: u8,
    version: u64,
    schema: u32,
    published: bool,
) -> DerivedIndexGeneration {
    DerivedIndexGeneration {
        generation_id: DerivedIndexGenerationId(id),
        index_id: DerivedIndexId(index),
        source_commit_id: CommitId(version + 100),
        source_branch_id: BranchId(format!("branch-{branch}")),
        applicability: DerivedIndexApplicability {
            branch_id: BranchId(format!("branch-{branch}")),
            version_id: VersionId(version),
            schema_version: SchemaVersionId(schema),
        },
        status: if published {
            DerivedIndexPublicationStatus::Published
        } else {
            DerivedIndexPublicationStatus::BuildFailed
        },
        entries: DerivedIndexEntries::EntityField(BTreeMap::new().into()),
    }
}

fn id(generation: Option<Arc<DerivedIndexGeneration>>) -> Option<DerivedIndexGenerationId> {
    generation.map(|generation| generation.generation_id)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]
    #[test]
    fn indexed_selection_matches_independent_inventory_ordering_after_replacements(
        updates in prop::collection::vec((0u64..32, 0u64..3, 0u8..3, 0u64..5, 0u32..3, any::<bool>()), 1..150)
    ) {
        let mut catalog = GenerationCatalog::default();
        let mut truth = BTreeMap::new();
        for (identity, index, branch, version, schema, published) in updates {
            let replacement = generation(identity, index, branch, version, schema, published);
            truth.insert(replacement.generation_id, replacement.clone());
            catalog.insert(replacement);
        }
        for index in 0..3 {
            for branch in [None, Some(BranchId("branch-0".into())), Some(BranchId("absent".into()))] {
                for version in 0..6 {
                    for schema in 0..4 {
                        let candidates = truth.values().filter(|g| g.index_id == DerivedIndexId(index));
                        let is_published = |g: &&DerivedIndexGeneration| g.status == DerivedIndexPublicationStatus::Published;
                        let is_branch = |g: &&DerivedIndexGeneration| branch.as_ref().is_none_or(|b| *b == g.applicability.branch_id);
                        let expected_candidate = candidates.clone().max_by_key(|g| (
                            is_published(g), is_branch(g),
                            g.applicability.version_id == VersionId(version),
                            g.applicability.schema_version == SchemaVersionId(schema),
                            g.applicability.version_id, g.generation_id,
                        )).map(|g| g.generation_id);
                        prop_assert_eq!(id(catalog.candidate(DerivedIndexId(index), branch.as_ref(), VersionId(version), SchemaVersionId(schema))), expected_candidate);
                        let expected_exact = candidates.clone().filter(is_published).filter(is_branch)
                            .filter(|g| g.applicability.version_id == VersionId(version) && g.applicability.schema_version == SchemaVersionId(schema))
                            .map(|g| g.generation_id).max();
                        prop_assert_eq!(id(catalog.exact(DerivedIndexId(index), branch.as_ref(), VersionId(version), SchemaVersionId(schema))), expected_exact);
                        let expected_commit = candidates.clone().filter(is_published).filter(is_branch)
                            .filter(|g| g.source_commit_id == CommitId(version + 100) && g.applicability.version_id == VersionId(version))
                            .map(|g| g.generation_id).max();
                        prop_assert_eq!(id(catalog.published_for_commit(DerivedIndexId(index), branch.as_ref(), CommitId(version + 100), VersionId(version))), expected_commit);
                        let expected_latest = candidates.filter(is_branch).map(|g| g.generation_id).max();
                        prop_assert_eq!(id(catalog.latest(DerivedIndexId(index), branch.as_ref())), expected_latest);
                    }
                }
            }
        }
        let inventory = catalog.all();
        prop_assert_eq!(inventory.len(), truth.len());
        for version in 0..6 {
            let selected = catalog.for_commit(CommitId(version + 100));
            let expected: Vec<_> = truth.values().filter(|g| g.source_commit_id == CommitId(version + 100)).collect();
            prop_assert_eq!(selected.iter().map(AsRef::as_ref).collect::<Vec<_>>(), expected);
            prop_assert_eq!(catalog.any_at_or_before(VersionId(version)), truth.values().any(|g| g.applicability.version_id <= VersionId(version)));
        }
    }
}

#[test]
fn selection_reads_one_payload_independent_of_retained_history() {
    let mut catalog = GenerationCatalog::default();
    for ordinal in 1..=10_000 {
        catalog.insert(generation(ordinal, 1, 0, ordinal, 1, true));
        if [100, 1_000, 10_000].contains(&ordinal) {
            let before = catalog.selection_counters();
            assert_eq!(
                id(catalog.exact(DerivedIndexId(1), None, VersionId(1), SchemaVersionId(1))),
                Some(DerivedIndexGenerationId(1))
            );
            assert_eq!(
                id(catalog.candidate(DerivedIndexId(1), None, VersionId(1), SchemaVersionId(1))),
                Some(DerivedIndexGenerationId(1))
            );
            assert_eq!(catalog.for_commit(CommitId(101)).len(), 1);
            let after = catalog.selection_counters();
            assert_eq!(
                after.generation_payload_reads - before.generation_payload_reads,
                3
            );
            assert_eq!(
                after.history_inventory_entries - before.history_inventory_entries,
                0
            );
        }
    }
    assert_eq!(catalog.all().len(), 10_000);
    assert_eq!(
        catalog.selection_counters().history_inventory_entries,
        10_000
    );
}

#[test]
fn replacement_removes_old_bindings_without_changing_held_readers_or_forks() {
    let mut catalog = GenerationCatalog::default();
    catalog.insert(generation(7, 1, 0, 3, 1, true));
    let held = catalog.generation(DerivedIndexGenerationId(7)).unwrap();
    let fork = catalog.clone();
    catalog.insert(generation(7, 2, 1, 4, 2, false));
    assert!(catalog.latest(DerivedIndexId(1), None).is_none());
    assert!(catalog.for_commit(CommitId(103)).is_empty());
    assert!(!catalog.any_at_or_before(VersionId(3)));
    assert!(catalog
        .exact(DerivedIndexId(2), None, VersionId(4), SchemaVersionId(2))
        .is_none());
    assert_eq!(held.index_id, DerivedIndexId(1));
    assert_eq!(
        id(fork.exact(DerivedIndexId(1), None, VersionId(3), SchemaVersionId(1))),
        Some(DerivedIndexGenerationId(7))
    );
    assert_eq!(fork.selection_counters().generation_payload_reads, 1);
    assert_eq!(fork.selection_counters().history_inventory_entries, 0);
}

#[test]
fn ordinary_publication_rejects_identity_reuse_before_changing_the_catalog() {
    let mut catalog = GenerationCatalog::default();
    catalog.publish(generation(7, 1, 0, 3, 1, true));
    let rejected = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        catalog.publish(generation(7, 2, 1, 4, 2, true));
    }));
    assert!(rejected.is_err());
    assert_eq!(
        id(catalog.latest(DerivedIndexId(1), None)),
        Some(DerivedIndexGenerationId(7))
    );
    assert!(catalog.latest(DerivedIndexId(2), None).is_none());
    assert_eq!(catalog.all().len(), 1);
}
