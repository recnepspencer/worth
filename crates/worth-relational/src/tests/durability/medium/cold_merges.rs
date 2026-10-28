//! Independent merge preparation crosses real recovered branch and journal ports.
use super::*;
use crate::execution::RelationalExecutionDenialCause;
use crate::history::data::BranchId;
use crate::merge::data::{
    MergeExecutionPreparationError, MergeExecutionRequest, MergeIntent, MergePlanningError,
};

use crate::indexes::data::{
    DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexEntries, DerivedIndexId,
    DerivedIndexKind,
};
struct Fixture {
    index: DerivedIndexId,
    expected_index_keys: Vec<crate::storage::data::AuthoritativeFieldComparisonKey>,
    runtime: crate::runtime::RelationalRuntime,
    source: Arc<MemberSource>,
    manifest: crate::durability::data::RelationalCheckpointManifest,
    symbol: Option<crate::symbols::data::Symbol>,
    expected: std::collections::BTreeMap<
        crate::identity::data::EntityId,
        crate::storage::data::EntityReadRecord,
    >,
}
fn recovered(execution: &worth_execution::ExecutionRequest<'_, '_>) -> Fixture {
    recovered_shape(execution, false)
}
fn recovered_shape(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    symbols: bool,
) -> Fixture {
    let producer = durable_runtime_with_test_schema();
    let symbol = if symbols {
        use super::cold_symbols::view_levels::{admit_population, publish_text, TextMutation};
        admit_population(execution, &producer);
        let (entity, _before) = publish_text(
            execution,
            &producer,
            TextMutation::Create,
            "paged-view-value-0300",
        );
        let (_, _current) = publish_text(
            execution,
            &producer,
            TextMutation::Update(entity),
            "paged-view-value-0600",
        );
        Some(
            producer
                .services
                .symbols
                .interner_snapshot()
                .symbol("paged-view-value-0600")
                .unwrap(),
        )
    } else {
        let entity = create_entity(execution, &producer, "merge-anchor");
        update_entity_and_release_snapshot(execution, &producer, entity, "merge-anchor-current");
        None
    };
    let feature = create_branch_from_main(&producer, "cold-merge-feature");
    let created = create_entity_outcome_on_branch(
        execution,
        &producer,
        "merge-feature-only",
        feature.clone(),
    );
    release_test_commit_snapshot(&producer, &created);
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
        .expect("index fixture has identity capacity")
        .index_id;
    let branch = BranchId("main".into());
    let built = producer.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: producer.history().branch_head(&branch).unwrap().commit_id,
            branch_id: branch.clone(),
            index_ids: vec![index],
        },
    );
    assert!(built.failed_indexes.is_empty());
    let reference = producer.fork().unwrap();
    let candidate = reference
        .prepare_merge_execution(execution, request())
        .unwrap();
    let merged = reference
        .execute_prepared_merge(execution, candidate)
        .unwrap();
    let view = reference
        .read_truth()
        .read_snapshot(&merged.commit.snapshot, execution)
        .unwrap()
        .unwrap();
    let expected = view
        .entities()
        .iter()
        .map(|record| (record.entity_id, record.clone()))
        .collect();
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
        panic!("actual entity index");
    };
    assert!(!entries.is_empty());
    let expected_index_keys = entries.keys().cloned().collect();
    let (recovery, source, _) = checkpoint_fixture(execution, producer, member, false);
    #[derive(serde::Deserialize)]
    struct Input {
        manifest: crate::durability::data::RelationalCheckpointManifest,
    }
    let manifest = rmp_serde::from_slice::<Input>(
        recovery
            .checkpoint()
            .unwrap()
            .checkpoint()
            .bytes()
            .strip_prefix(crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC)
            .unwrap(),
    )
    .unwrap()
    .manifest;
    source.damaged.store(false, Ordering::Relaxed);
    let medium = TestDurableMedium::new().attachment();
    for bytes in source.commits.values() {
        let artifact = crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone()).unwrap();
        assert!(matches!(
            medium.publish(&artifact),
            crate::durability::medium::RelationalDurablePublication::Committed
        ));
    }
    let mut runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(test_schema_registry())
        .build_durable(medium)
        .expect("valid native runtime configuration");
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert_eq!(
        source.reads.load(Ordering::Relaxed),
        0,
        "reopen leaves branch regions cold"
    );
    if let Some(symbol) = symbol {
        assert!(
            runtime
                .services
                .symbols
                .interner_snapshot()
                .resolve(symbol)
                .is_none(),
            "the unchanged ancestor's current field text is cold"
        );
    }
    Fixture {
        index,
        expected_index_keys,
        runtime,
        source,
        manifest,
        symbol,
        expected,
    }
}

fn request() -> MergeExecutionRequest {
    MergeExecutionRequest::new(
        BranchId("main".into()),
        BranchId("cold-merge-feature".into()),
        MergeIntent::ReconcileIntoTarget,
    )
}

#[test]
fn cold_merge_preparation_refuses_region_damage_without_journal_effects() {
    let backing = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    let Fixture {
        runtime, source, ..
    } = recovered(execution);
    let position = runtime.history().latest_patch_stream_position();
    let offered = attached_test_medium(&runtime).offered().len();
    source.damaged.store(true, Ordering::Relaxed);
    let expected = MergeExecutionPreparationError::Planning(MergePlanningError::Execution(
        RelationalExecutionDenialCause::StorageUnavailable(
            crate::storage::data::RelationalStorageUnavailable {
                subject: crate::storage::data::RelationalStorageSubject::Region(PartitionId::main()),
                cause: Cause::IntegrityMismatch,
            },
        ),
    ));
    assert_eq!(
        runtime
            .prepare_merge_execution(execution, request())
            .unwrap_err(),
        expected
    );
    let reads = source.reads.load(Ordering::Relaxed);
    assert!(
        reads > 0,
        "merge preparation must actually cross a cold region"
    );
    source.damaged.store(false, Ordering::Relaxed);
    assert_eq!(
        runtime
            .prepare_merge_execution(execution, request())
            .unwrap_err(),
        expected
    );
    assert_eq!(
        source.reads.load(Ordering::Relaxed),
        reads,
        "damage stays sticky"
    );
    assert_eq!(runtime.history().latest_patch_stream_position(), position);
    assert_eq!(attached_test_medium(&runtime).offered().len(), offered);
}

#[test]
fn independent_merge_executes_from_a_real_cold_checkpoint() {
    let backing = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    let Fixture {
        runtime,
        source,
        expected,
        ..
    } = recovered(execution);
    let prepared = runtime
        .prepare_merge_execution(execution, request())
        .unwrap();
    assert!(source.reads.load(Ordering::Relaxed) > 0);
    let outcome = runtime.execute_prepared_merge(execution, prepared).unwrap();
    assert_eq!(outcome.structural_summary.adopted_source_record_count, 1);
    assert_eq!(changed_entities(&outcome.commit).len(), 1);
    let view = runtime
        .read_truth()
        .read_snapshot(&outcome.commit.snapshot, execution)
        .unwrap()
        .unwrap();
    assert_eq!(
        view.entities().len(),
        3,
        "main retains its own records and the adopted feature record"
    );
    assert_merge_records(view.entities(), &expected);
}

#[path = "cold_merge_member_matrix.rs"]
mod member_matrix;

fn assert_merge_records(
    actual: &[crate::storage::data::EntityReadRecord],
    expected: &std::collections::BTreeMap<
        crate::identity::data::EntityId,
        crate::storage::data::EntityReadRecord,
    >,
) {
    assert_eq!(actual.len(), expected.len());
    // Abandoned preparation rounds may burn identities. Compare immutable
    // meaning with the memory-only merge, and separately preserve uniqueness.
    let ids = actual
        .iter()
        .map(|row| row.entity_id)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), actual.len());
    for row in actual {
        let counterpart = expected
            .values()
            .find(|prior| prior.authoritative_aspect_state == row.authoritative_aspect_state)
            .expect("each record has the memory reference's value");
        assert_eq!(row.kind, counterpart.kind);
        assert_eq!(row.lifecycle, counterpart.lifecycle);
        assert_eq!(row.created_at_version, counterpart.created_at_version);
        assert_eq!(row.retired_at_version, counterpart.retired_at_version);
        assert_eq!(row.lineage_id.is_some(), counterpart.lineage_id.is_some());
    }
}

#[path = "cold_merge_index_matrix.rs"]
mod index_matrix;
