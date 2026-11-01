//! Real Directory tails restore Native-owned catalog IDs without a checkpoint.
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    history::BranchId,
    identity::KindId,
    indexes::{
        DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind,
        RelatedEntityEndpoint, RelatedEntityOrderingDirection, RelatedEntityOrderingField,
        RelationJoinDefinition, RelationJoinLeg, RelationJoinSharedEndpoint,
    },
};

fn definitions() -> Vec<DerivedIndexDefinition> {
    let field = |label: &str| {
        worth_relational::facade::transactions::planned_single_field_locator(
            crate::batches::aspect_key(label),
            crate::batches::field_key(label),
        )
    };
    let entity = DerivedIndexKind::EntityField {
        field_locator: field("name"),
    };
    let kinds = [
        entity.clone(),
        DerivedIndexKind::RelationField {
            field_locator: field("label"),
        },
        DerivedIndexKind::RelatedEntityOrdering {
            relation_kind: KindId(2),
            parent_endpoint: RelatedEntityEndpoint::SourceParent,
            child_kind: KindId(1),
            ordering: vec![RelatedEntityOrderingField::new(
                field("name"),
                RelatedEntityOrderingDirection::Descending,
            )],
        },
        DerivedIndexKind::RelationJoin(RelationJoinDefinition::new(
            RelationJoinLeg::new(KindId(2), RelationJoinSharedEndpoint::Source, KindId(1)),
            RelationJoinLeg::new(KindId(2), RelationJoinSharedEndpoint::Target, KindId(1)),
            KindId(1),
        )),
        entity.clone(),
        entity,
    ];
    kinds
        .into_iter()
        .enumerate()
        .map(|(offset, kind)| DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: if offset >= 4 {
                "equal-duplicates".into()
            } else {
                format!("durable-definition-{offset}")
            },
            kind,
            branch_scoped: offset != 1,
        })
        .collect()
}

#[test]
fn tail_only_directory_reopen_preserves_catalog_ids_duplicates_allocator_and_derived_build() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let home = crate::durable_run::open(directory.path(), execution).unwrap();
    let runtime = home.relational().runtime();
    let installed = definitions()
        .into_iter()
        .map(|definition| runtime.index_authority().register(definition).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        installed
            .iter()
            .map(|definition| definition.index_id.0)
            .collect::<Vec<_>>(),
        (1..=6).collect::<Vec<_>>()
    );
    let first = crate::workload::commit(
        runtime,
        execution,
        &BranchId("main".into()),
        crate::batches::create(1),
    )
    .unwrap();
    let record = first.changed_records[0].clone();
    runtime
        .snapshots()
        .release_snapshot(&first.snapshot)
        .unwrap();
    let (_, fork) = runtime
        .observe_fork_source(&BranchId("main".into()))
        .unwrap();
    runtime
        .fork_branch(execution, BranchId("feature".into()), fork)
        .unwrap();
    let transitioned = crate::schema_transition::commit(
        runtime,
        execution,
        &BranchId("feature".into()),
        crate::batches::create(3),
    )
    .unwrap();
    runtime
        .snapshots()
        .release_snapshot(&transitioned.snapshot)
        .unwrap();
    assert_eq!(
        crate::schema_transition::basis_of(runtime, &BranchId("main".into())),
        crate::schema_transition::SOURCE
    );
    assert_eq!(
        crate::schema_transition::basis_of(runtime, &BranchId("feature".into())),
        crate::schema_transition::TARGET
    );
    let second = crate::workload::commit(
        runtime,
        execution,
        &BranchId("main".into()),
        crate::batches::create(2),
    )
    .unwrap();
    let receipt = second.commit.clone();
    runtime
        .snapshots()
        .release_snapshot(&second.snapshot)
        .unwrap();
    let artifacts = crate::workload::artifacts_of(runtime);
    let states = crate::workload::states_of(runtime, std::slice::from_ref(&record));
    // This fresh home has never requested a checkpoint. Independently inspect
    // its actual anchored record, then destroy the original runtime/OS owner.
    assert!(home
        .physical_serving_for_test()
        .anchored_records()
        .unwrap()
        .read(
            worth_store::physical_runtime::AnchoredRecordName::new("home.checkpoint").unwrap(),
            1 << 30
        )
        .unwrap()
        .is_none());
    drop((first, second, transitioned));
    drop(home); // Explicit unbudgeted crash/owner-loss fixture, no close checkpoint.
    let reopened = crate::durable_run::open(directory.path(), execution).unwrap();
    let runtime = reopened.relational().runtime();
    let lookup = runtime.index_access().definition_lookup_snapshot();
    assert_eq!(lookup.definition_count(), installed.len());
    for definition in &installed[..4] {
        assert_eq!(
            lookup.matching_definition(definition),
            Some(definition.clone())
        );
    }
    assert_eq!(lookup.candidate_count_for_name("equal-duplicates"), 2);
    assert_eq!(
        crate::workload::artifacts_of(runtime),
        artifacts,
        "original positioned canonical bytes"
    );
    assert_eq!(
        crate::workload::states_of(runtime, std::slice::from_ref(&record)),
        states,
        "both tails and actual reads survive"
    );
    let ids = vec![
        installed[0].index_id,
        installed[4].index_id,
        installed[5].index_id,
    ];
    let build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: receipt.commit_id,
            branch_id: receipt.branch_id.clone(),
            index_ids: ids.clone(),
        },
    );
    assert!(build.execution_denial.is_none(), "{build:?}");
    assert!(build.failed_indexes.is_empty(), "{build:?}");
    assert_eq!(
        build
            .generations
            .iter()
            .map(|generation| generation.index_id)
            .collect::<Vec<_>>(),
        ids
    );
    for id in ids {
        let generation = runtime
            .index_access()
            .read_latest_generation(execution, id, &receipt.branch_id)
            .unwrap()
            .unwrap();
        assert_eq!(generation.source_commit_id, receipt.commit_id);
    }
    let next = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            name: "allocator-after-source-recovery".into(),
            ..installed[0].clone()
        })
        .unwrap();
    assert_eq!(next.index_id.0, installed.last().unwrap().index_id.0 + 1);
    drop(reopened); // Generations/read caches are separate unbudgeted fixture teardown.
}

#[test]
fn failed_checkpoint_cannot_acknowledge_catalog_before_a_later_actual_journal_commit() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let home = crate::durable_run::open(directory.path(), execution).unwrap();
    let runtime = home.relational().runtime();
    let seed = crate::workload::commit(
        runtime,
        execution,
        &BranchId("main".into()),
        crate::batches::create(10),
    )
    .unwrap();
    runtime
        .snapshots()
        .release_snapshot(&seed.snapshot)
        .unwrap();
    let installed = runtime
        .index_authority()
        .register(definitions().remove(0))
        .unwrap();
    let constrained = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(Some(0)),
        worth_execution::CancellationSource::new().token(),
        None,
    );
    let failure = home
        .checkpoint(&ExecutionRequest::serial(&constrained))
        .unwrap_err();
    assert!(
        matches!(
            &failure,
            worth_store_application::ApplicationHomeCheckpointDenial::Execution(
                worth_execution::WorkCeilingDenial::Admission(
                    worth_execution::LeaseDenial::MemoryExhausted(_)
                )
            )
        ),
        "exact original request allocation refusal: {failure:?}"
    );
    assert!(
        home.physical_serving_for_test()
            .anchored_records()
            .unwrap()
            .read(
                worth_store::physical_runtime::AnchoredRecordName::new("home.checkpoint").unwrap(),
                1 << 30
            )
            .unwrap()
            .is_none(),
        "failed capture cannot select a physical anchor"
    );
    let committed = crate::workload::commit(
        runtime,
        execution,
        &BranchId("main".into()),
        crate::batches::create(11),
    )
    .unwrap();
    runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    let original = crate::workload::artifacts_of(runtime);
    drop((seed, committed, home));
    let reopened = crate::durable_run::open(directory.path(), execution).unwrap();
    assert_eq!(
        reopened
            .relational()
            .runtime()
            .index_access()
            .matching_definition(&installed),
        Some(installed)
    );
    assert_eq!(
        crate::workload::artifacts_of(reopened.relational().runtime()),
        original
    );
    drop(reopened);
}
