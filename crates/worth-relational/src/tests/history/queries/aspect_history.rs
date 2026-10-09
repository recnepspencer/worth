use crate::tests::support::*;

#[test]
fn record_local_aspect_history_reads_committed_patch_truth() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let created = create_entity_outcome(&runtime, "before");
    let entity = changed_entities(&created)[0];
    let updated = update_entity(&runtime, entity, "after");
    let history =
        runtime
            .history()
            .entity_aspect_history(&BranchId("main".to_string()), entity, None);
    let traced = runtime.history().entity_aspect_history_with_trace(
        &BranchId("main".to_string()),
        entity,
        None,
    );
    let artifact = traced.trace.diagnostic_artifact();
    let digest = traced.aspect_history_digest();
    let resolved_aspects = ordered_aspect_keys([
        AspectKey::new("lifecycle").unwrap(),
        AspectKey::new("name").unwrap(),
    ]);

    assert_eq!(history.len(), 2);
    assert_eq!(traced.entries, history);
    assert_eq!(
        traced.trace.requested_target,
        HistoryAspectQueryTarget::Entity(entity)
    );
    assert_eq!(traced.trace.returned_entries, 2);
    assert_eq!(traced.trace.resolved_aspects, resolved_aspects);
    assert_eq!(
        traced.trace.searched_commit_span,
        Some(AspectHistoryCommitSpan {
            first_commit_id: created.commit.commit_id,
            last_commit_id: updated.commit.commit_id,
        })
    );
    assert_eq!(traced.trace.searched_lineage_event_span, None);
    assert_eq!(traced.trace.traversed_lineage_events, 0);
    assert_eq!(artifact.scope, DiagnosticsScope::History);
    assert_eq!(artifact.kind, DiagnosticsArtifactKind::DetailedTrace);
    assert_eq!(artifact.entries.len(), 1);
    assert_eq!(
        artifact.entries[0].code,
        DiagnosticCode::AspectHistoryResolved
    );
    assert_eq!(
        digest.requested_target,
        HistoryAspectQueryTarget::Entity(entity)
    );
    assert_eq!(digest.entry_count, 2);
    assert_eq!(digest.resolved_aspects, resolved_aspects);
    assert_direct_history_origin_invariants(&history, RecordRef::Entity(entity));
    assert_eq!(history[0].origin.commit_id, created.commit.commit_id);
    assert_eq!(history[0].origin.target, RecordRef::Entity(entity));
    assert_eq!(history[0].origin.changed_aspects, resolved_aspects);
    assert_eq!(history[1].origin.commit_id, updated.commit.commit_id);

    let filtered = runtime.history().entity_aspect_history(
        &BranchId("main".to_string()),
        entity,
        Some(&ProjectionAspectFilter::whole_aspects(
            ProjectionAspectFilterMode::All,
            [AspectKey::new("name").unwrap()],
        )),
    );
    let any_filtered = runtime.history().entity_aspect_history(
        &BranchId("main".to_string()),
        entity,
        Some(&ProjectionAspectFilter::whole_aspects(
            ProjectionAspectFilterMode::Any,
            [
                AspectKey::new("missing").unwrap(),
                AspectKey::new("name").unwrap(),
            ],
        )),
    );
    assert_eq!(filtered.len(), 2);
    assert_eq!(any_filtered.len(), 2);
}

#[test]
fn aspect_history_projection_filter_matches_field_level_patch_locus() {
    let runtime = AspectSchemaFixture {
        entity_aspects: vec![
            entity_summary_struct_aspect(aspect_key("summary"), field_key("summary")),
            lifecycle_aspect(),
        ],
        ..AspectSchemaFixture::default()
    }
    .build_runtime();
    let mut create_txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    create_txn
        .push_batch(
            WorkerIntentBatch::new("summary-history").push(MutationIntent::Create(
                CreateIntent::Entity(crate::transactions::data::EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: crate::symbols::data::ClientKey::raw("summary-record"),
                    fields: AspectFieldPatch::new(std::collections::BTreeMap::from([(
                        crate::transactions::data::planned_single_field_locator(
                            aspect_key("summary"),
                            field_key("title"),
                        ),
                        string_aspect_value("title v1"),
                    )])),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let created = create_txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let entity = changed_entities(&created)[0];
    let mut update_txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    update_txn
        .push_batch(
            WorkerIntentBatch::new("summary-history-update").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: AspectFieldPatch::from_locator(
                        crate::transactions::data::planned_single_field_locator(
                            aspect_key("summary"),
                            field_key("status"),
                        ),
                        string_aspect_value("ready"),
                    ),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    update_txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();

    let status_filter = ProjectionAspectFilter::new(
        ProjectionAspectFilterMode::All,
        ProjectionAspectScope::fields(aspect_key("summary"), [field_key("status")]),
    );
    let missing_sibling_filter = ProjectionAspectFilter::new(
        ProjectionAspectFilterMode::All,
        ProjectionAspectScope::fields(aspect_key("summary"), [field_key("missing")]),
    );

    let status_history = runtime.history().entity_aspect_history(
        &BranchId("main".to_string()),
        entity,
        Some(&status_filter),
    );
    let missing_sibling_history = runtime.history().entity_aspect_history(
        &BranchId("main".to_string()),
        entity,
        Some(&missing_sibling_filter),
    );

    assert_eq!(status_history.len(), 1);
    assert_eq!(
        status_history[0].origin.changed_aspects,
        ordered_aspect_keys([aspect_key("summary")])
    );
    assert!(missing_sibling_history.is_empty());
}

#[test]
fn bulk_like_aspect_history_filters_and_query_packets_stay_stable_after_recovery() {
    let runtime =
        persisted_runtime_with_declared_aspect_schema(CascadeDeletePolicy::RetainDanglingForAudit);
    let hub = create_entity(&runtime, "hub");
    let leaves = (0..8)
        .map(|index| {
            create_entity_in_partition(
                &runtime,
                &format!("leaf-{index}"),
                PartitionId((index % 3 + 7) as u32),
            )
        })
        .collect::<Vec<_>>();
    let relations = leaves
        .iter()
        .enumerate()
        .map(|(index, leaf)| {
            create_relation_in_partition(
                &runtime,
                hub,
                *leaf,
                &format!("edge-{index}"),
                PartitionId((index % 4 + 20) as u32),
            )
        })
        .collect::<Vec<_>>();
    for (index, leaf) in leaves.iter().enumerate() {
        let _ = update_entity(&runtime, *leaf, &format!("leaf-{index}-updated"));
    }
    let snapshot = runtime.visibility_authority().snapshot();
    let planned_packet = runtime
        .storage_access()
        .plan_read_explicit_query_packet(
            &snapshot,
            &explicit_query_packet(
                &runtime,
                &snapshot,
                "fanout-entities",
                leaves
                    .iter()
                    .copied()
                    .map(RecordRef::Entity)
                    .collect::<Vec<_>>(),
            ),
        )
        .unwrap();
    let before_recovery_reads = execute_explicit_query(
        &runtime,
        &snapshot,
        "fanout-entities",
        leaves
            .iter()
            .copied()
            .map(RecordRef::Entity)
            .collect::<Vec<_>>(),
    )
    .result;
    let name_filter = all_aspect_filter(["name"]);
    let endpoints_filter = all_aspect_filter(["source", "target"]);
    let delete_outcome = delete_entity(&runtime, hub);
    let expected_entity_digests = leaves
        .iter()
        .map(|entity| entity_aspect_history_digest(&runtime, *entity, Some(&name_filter)))
        .collect::<Vec<_>>();
    let expected_relation_digests = relations
        .iter()
        .map(|relation| {
            relation_aspect_history_digest(&runtime, *relation, Some(&endpoints_filter))
        })
        .collect::<Vec<_>>();
    let lifecycle_filter = any_aspect_filter(["lifecycle"]);
    let expected_lifecycle_relation_digests = relations
        .iter()
        .map(|relation| {
            relation_aspect_history_digest(&runtime, *relation, Some(&lifecycle_filter))
        })
        .collect::<Vec<_>>();
    runtime.durability_authority().checkpoint().unwrap();
    let recovery_plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );

    let mut recovered =
        persisted_runtime_with_declared_aspect_schema(CascadeDeletePolicy::RetainDanglingForAudit);
    let recovery = recovered
        .durability_recovery()
        .recover(recovery_plan)
        .unwrap();
    let recovered_snapshot = recovered.visibility_authority().snapshot();
    let recovered_reads = execute_explicit_query(
        &recovered,
        &recovered_snapshot,
        "fanout-entities",
        leaves
            .iter()
            .copied()
            .map(RecordRef::Entity)
            .collect::<Vec<_>>(),
    )
    .result;
    let recovered_entity_digests = leaves
        .iter()
        .map(|entity| entity_aspect_history_digest(&recovered, *entity, Some(&name_filter)))
        .collect::<Vec<_>>();
    let recovered_relation_digests = relations
        .iter()
        .map(|relation| {
            relation_aspect_history_digest(&recovered, *relation, Some(&endpoints_filter))
        })
        .collect::<Vec<_>>();
    let recovered_lifecycle_relation_digests = relations
        .iter()
        .map(|relation| {
            relation_aspect_history_digest(&recovered, *relation, Some(&lifecycle_filter))
        })
        .collect::<Vec<_>>();

    assert_eq!(recovery.latest_commit, Some(delete_outcome.commit.clone()));
    assert_eq!(planned_packet.target_count, leaves.len());
    assert_eq!(before_recovery_reads.entities.len(), leaves.len());
    assert_eq!(before_recovery_reads.entities, recovered_reads.entities);
    assert_eq!(expected_entity_digests, recovered_entity_digests);
    assert_eq!(expected_relation_digests, recovered_relation_digests);
    assert_eq!(
        expected_lifecycle_relation_digests,
        recovered_lifecycle_relation_digests
    );
    assert!(expected_entity_digests
        .iter()
        .all(|digest| digest.entry_count == 2));
    assert!(expected_relation_digests
        .iter()
        .all(|digest| digest.entry_count == 1));
    assert!(expected_lifecycle_relation_digests
        .iter()
        .all(|digest| digest.entry_count == 2));
}
