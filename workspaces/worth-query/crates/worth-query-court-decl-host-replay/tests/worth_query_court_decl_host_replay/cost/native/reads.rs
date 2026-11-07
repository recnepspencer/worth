use super::*;
use worth_relational::facade::query::*;

pub fn verify(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    commit: &CommitResult,
    indexes: &[DerivedIndexId],
    edited: EntityId,
    sample: &mut Sample,
) {
    let (snapshot, report) = execution
        .run(ExecutionWorkCeiling::new(None), |_| {
            let (_, basis) = runtime
                .observe_branch(&runtime.main_branch_identity())
                .unwrap();
            runtime
                .snapshots()
                .snapshot_for_observation(&basis.observation())
                .unwrap()
        })
        .unwrap();
    sample.record(Phase::SnapshotOpen, "charged_work", report.charged_work());
    let (record, report) = execution
        .run(ExecutionWorkCeiling::new(None), |_| {
            runtime
                .read_truth()
                .visible_entity_at_version(execution, edited, commit.version_id)
                .unwrap()
                .unwrap()
        })
        .unwrap();
    assert_eq!(record.entity_id, edited);
    sample.record(
        Phase::ReadAdmission,
        "point_read_work",
        report.charged_work(),
    );
    let packet = PlannedQueryPacket {
        label: "scale-fixed-read".into(),
        context_id: runtime.read_truth().query_plan_context(&snapshot).unwrap(),
        scope: QueryScope::EntityFieldEquals {
            field_locator: schema::locator("name"),
            value: worth_foundational::facade::AspectValue::String("edited".into()),
            partition_scope: None,
        },
        locality: QueryLocalityClass::CrossPartitionTraversal,
        ordering: QueryOrderingContract::CanonicalEntityIdOrder,
        access_contract: QueryAccessContract::DerivedIndexWithStorageParity,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(1),
        target_count_hint: 1,
    };
    let plan = runtime
        .read_truth()
        .plan_query_packet(&snapshot, packet)
        .unwrap();
    let lookup = BoundedEntityFieldLookupRequest::new(
        snapshot.clone(),
        indexes[0],
        KindId(1),
        schema::locator("name"),
        worth_foundational::facade::AspectValue::String("edited".into()),
        2,
    )
    .unwrap();
    let (indexed, report) = execution
        .run(ExecutionWorkCeiling::new(None), |_| {
            runtime
                .index_access()
                .execute_bounded_entity_field_lookup(
                    lookup,
                    BoundedIndexParityMode::Production,
                    execution,
                )
                .unwrap()
        })
        .unwrap();
    sample.record(
        Phase::ReadAdmission,
        "indexed_lookup_work",
        report.charged_work(),
    );
    assert_eq!(indexed.candidate_entity_ids(), &[edited]);
    // Independent whole-state oracle is outside all measured intervals.
    let cold = runtime
        .read_truth()
        .execute_query_plan(plan, execution)
        .unwrap()
        .unwrap();
    assert_eq!(cold.outcome.result.entities.len(), 1);
    let warm: Vec<_> = indexes
        .iter()
        .map(|index| {
            runtime
                .index_access()
                .read_published_generation_for_commit(execution, *index, &commit.commit)
                .unwrap()
                .unwrap()
        })
        .collect();
    let rebuilt = runtime
        .index_authority()
        .build_for_commit(execution, build_request(indexes, commit));
    assert!(rebuilt.failed_indexes.is_empty());
    for generation in &rebuilt.generations {
        let maintained = warm
            .iter()
            .find(|g| {
                g.index_id == generation.index_id
                    && g.source_commit_id == generation.source_commit_id
            })
            .unwrap();
        assert_eq!(
            maintained.entries, generation.entries,
            "incremental index differs from cold"
        );
    }
    runtime.snapshots().release_snapshot(&snapshot).unwrap();
}
