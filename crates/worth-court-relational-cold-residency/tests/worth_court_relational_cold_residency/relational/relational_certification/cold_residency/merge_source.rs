//! Real cold merge sources and independently executed warm-reference meaning.
use super::{
    checkpoint_receiver::reopen,
    current_publications::create_entity,
    merge_publications::{self, request},
    symbol_text_publications::*,
};
use std::{collections::BTreeMap, sync::Arc};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    history::BranchId,
    identity::*,
    indexes::*,
    runtime::{EntityReadRecord, RelationalRuntime},
    storage::AuthoritativeFieldComparisonKey,
};
use worth_testkit_relational::{
    aspect_storage::{aspect_field_locator, aspect_key, field_key},
    cold_recovery::{capture_source, update_entity, ColdRecoverySourceFixture},
    durable_medium::TestDurableMedium,
    durable_publication::durable_runtime,
};
pub(super) struct MergeRecipe {
    original: ColdRecoverySourceFixture,
    index: DerivedIndexId,
    expected_index_keys: Vec<AuthoritativeFieldComparisonKey>,
    symbol: Option<worth_relational::facade::symbols::Symbol>,
    expected: Arc<BTreeMap<EntityId, EntityReadRecord>>,
}
pub(super) struct MergeFixture{pub runtime:RelationalRuntime,pub source:Arc<worth_testkit_relational::durable_medium::immutable_members::ImmutableCheckpointMemberSource>,pub medium:Arc<TestDurableMedium>,pub directory:super::selected_directory::Directory,pub index:DerivedIndexId,pub expected_index_keys:Vec<AuthoritativeFieldComparisonKey>,pub symbol:Option<worth_relational::facade::symbols::Symbol>,pub expected:Arc<BTreeMap<EntityId,EntityReadRecord>>}
impl MergeRecipe {
    pub fn new(execution: &ExecutionRequest<'_, '_>, symbols: bool) -> Self {
        let medium = TestDurableMedium::new();
        let producer = durable_runtime(&medium);
        let symbol = if symbols {
            admit_population(execution, &producer);
            let (entity, _before, _) = publish_text(
                execution,
                &producer,
                TextMutation::Create,
                "paged-view-value-0300",
            );
            let (_, _current, symbol) = publish_text(
                execution,
                &producer,
                TextMutation::Update(entity),
                "paged-view-value-0600",
            );
            Some(symbol)
        } else {
            let entity = create_entity(execution, &producer, "merge-anchor");
            update_entity(execution, &producer, entity, "merge-anchor-current");
            None
        };
        let feature = merge_publications::branch(execution, &producer, "cold-merge-feature");
        merge_publications::create(
            execution,
            &producer,
            "merge-feature-only",
            PartitionId::main(),
            &feature,
        );
        let member = create_entity(execution, &producer, "member-before");
        let index = producer
            .index_authority()
            .register(DerivedIndexDefinition {
                index_id: DerivedIndexId(0),
                name: "merge-matrix-name".into(),
                branch_scoped: true,
                kind: DerivedIndexKind::EntityField {
                    field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
                },
            })
            .unwrap()
            .index_id;
        let branch = BranchId("main".into());
        let (_, basis) = producer
            .observe_branch(&producer.main_branch_identity())
            .unwrap();
        let built = producer.index_authority().build_for_commit(
            execution,
            DerivedIndexBuildRequest {
                source_commit_id: basis.observation().commit_id().unwrap(),
                branch_id: branch.clone(),
                index_ids: vec![index],
            },
        );
        assert!(built.failed_indexes.is_empty());
        drop(basis);
        let reference = producer.fork().unwrap();
        let candidate = reference
            .prepare_merge_execution(execution, request(&reference))
            .unwrap();
        let merged = reference
            .execute_prepared_merge(execution, candidate)
            .unwrap();
        let view = reference
            .read_truth()
            .read_snapshot(&merged.commit.snapshot, execution)
            .unwrap()
            .unwrap();
        let expected = Arc::new(
            view.entities()
                .iter()
                .map(|r| (r.entity_id, r.clone()))
                .collect(),
        );
        let rebuilt = reference.index_authority().build_for_commit(
            execution,
            DerivedIndexBuildRequest {
                source_commit_id: merged.commit.commit.commit_id,
                branch_id: branch.clone(),
                index_ids: vec![index],
            },
        );
        assert!(rebuilt.failed_indexes.is_empty());
        let generation = reference
            .index_access()
            .read_latest_generation(execution, index, &branch)
            .unwrap()
            .unwrap();
        let DerivedIndexEntries::EntityField(entries) = &generation.entries else {
            panic!("actual entity index")
        };
        assert!(!entries.is_empty());
        let expected_index_keys = entries.keys().cloned().collect();
        let original = capture_source(execution, producer, medium, member, false);
        Self {
            original,
            index,
            expected_index_keys,
            symbol,
            expected,
        }
    }
    pub fn reopen(&self, execution: &ExecutionRequest<'_, '_>) -> MergeFixture {
        {
            let reader = reopen(execution, &self.original);
            MergeFixture {
                runtime: reader.runtime,
                source: reader.source,
                medium: reader.medium,
                directory: reader.directory,
                index: self.index,
                expected_index_keys: self.expected_index_keys.clone(),
                symbol: self.symbol,
                expected: self.expected.clone(),
            }
        }
    }
}
pub(super) fn assert_merge_records(
    actual: &[EntityReadRecord],
    expected: &BTreeMap<EntityId, EntityReadRecord>,
) {
    assert_eq!(actual.len(), expected.len());
    let ids = actual
        .iter()
        .map(|row| row.entity_id)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), actual.len());
    for row in actual {
        let counterpart = expected
            .values()
            .find(|prior| prior.authoritative_aspect_state == row.authoritative_aspect_state)
            .expect("each record has the independent memory reference value");
        assert_eq!(row.kind, counterpart.kind);
        assert_eq!(row.lifecycle, counterpart.lifecycle);
        assert_eq!(row.created_at_version, counterpart.created_at_version);
        assert_eq!(row.retired_at_version, counterpart.retired_at_version);
        assert_eq!(row.lineage_id.is_some(), counterpart.lineage_id.is_some());
    }
}
