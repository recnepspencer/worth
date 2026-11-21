use super::*;
use crate::facade::history::BranchId;
use crate::facade::indexes::{DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind};
use crate::facade::merge::{MergeExecutionRequest, MergeIntent};
use crate::facade::runtime::RelationalRuntimeApi;
use crate::facade::transactions::{
    CreateIntent, EntityMutationIntent, MutationIntent, RecordRef, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};
use crate::history::data::RelationalDescriptiveTouch as Touch;
use crate::storage::data::AuthoritativeFieldComparisonKey;
use crate::tests::support::{changed_entities, read_entity_aspect_field};

#[test]
fn native_merge_touches_only_changed_revision_and_both_index_membership_keys() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = runtime_with_aspect_field_merge_policy(
        AspectKey::new("status").unwrap(),
        field_key("status"),
        AspectMergePolicyKind::PreferRicher,
    );
    let entity = create_entity_with_aspect_fields(
        execution,
        &mut runtime,
        "shared",
        crate::tests::support::single_string_aspect_field_patch(
            AspectKey::new("status").unwrap(),
            field_key("status"),
            "off",
        ),
    );
    let index = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "status.merge.touch".to_string(),
            kind: DerivedIndexKind::EntityField {
                field_locator: crate::tests::support::aspect_field_locator(
                    AspectKey::new("status").unwrap(),
                    field_key("status"),
                ),
            },
            branch_scoped: true,
        })
        .expect("index fixture has identity capacity");
    // Branch-scoped indexes are built explicitly for each admitted basis: after
    // registration on main, and for the fork; ordinary maintenance refuses to
    // reconstruct them implicitly.
    rebuild_branch_index(execution, &runtime, "main", index.index_id);
    create_branch_from_main(&runtime, "feature");
    rebuild_branch_index(execution, &runtime, "feature", index.index_id);
    let mut source = crate::tests::support::test_owner_begin_transaction_for_branch(
        &runtime,
        BranchId("feature".to_string()),
    );
    source
        .push_batch(
            WorkerIntentBatch::new("source-status").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: crate::tests::support::single_string_aspect_field_patch(
                        AspectKey::new("status").unwrap(),
                        field_key("status"),
                        "active",
                    ),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    source
        .commit(execution, &runtime)
        .expect("source status commit");

    let prepared = runtime
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            },
        )
        .expect("native merge preparation");
    let merged = runtime
        .execute_prepared_merge(execution, prepared)
        .expect("native merge commit");
    let envelope = runtime
        .replay()
        .canonical_commit_envelope(execution, merged.commit.commit.commit_id)
        .unwrap()
        .expect("sealed canonical merge envelope");
    let touches = envelope
        .descriptive_touches()
        .exact_touches()
        .expect("native materialized merge has exact revisions");
    assert!(touches
        .iter()
        .any(|touch| matches!(touch, Touch::AspectRevision {
        record: RecordRef::Entity(id), aspect,
    } if *id == entity && aspect.as_str() == "status")));
    assert!(touches
        .iter()
        .any(|touch| matches!(touch, Touch::FieldRevision {
        record: RecordRef::Entity(id), aspect, path, ..
    } if *id == entity && aspect.as_str() == "status"
        && path.fields() == [field_key("status")])));
    assert!(
        !touches.iter().any(|touch| matches!(touch,
        Touch::AspectRevision { aspect, .. } | Touch::FieldRevision { aspect, .. }
            if aspect.as_str() == "name")),
        "retained A has no new revision: {touches:?}"
    );
    let old = AuthoritativeFieldComparisonKey::from_aspect_value(
        &crate::tests::support::string_aspect_value("off"),
    );
    let new = AuthoritativeFieldComparisonKey::from_aspect_value(
        &crate::tests::support::string_aspect_value("active"),
    );
    assert!(
        touches
            .iter()
            .any(|touch| matches!(touch, Touch::IndexMembership {
        index: id, old_key: Some(before), new_key: Some(after), ..
    } if *id == index.index_id && *before == old && *after == new)),
        "merge carries the actual old and new index membership: {touches:?}"
    );

    let main = BranchId("main".into());
    let feature = BranchId("feature".into());
    let mut generations = Vec::new();
    let mut original_roots = Vec::new();
    for branch in [&main, &feature] {
        assert_eq!(
            read_entity_aspect_field_display(
                execution,
                &runtime,
                branch,
                entity,
                AspectKey::new("status").unwrap(),
                field_key("status"),
            ),
            "active",
            "the selected target must include the non-first-parent contribution"
        );
        let head = runtime.history().branch_head(branch).unwrap();
        let original = runtime.history.branch_cell(branch).unwrap().root().unwrap();
        assert_eq!(original.commit_id(), Some(head.commit_id));
        original_roots.push((head.commit_id, original.axes(), original));
        let built = runtime.index_authority().build_for_commit(
            execution,
            crate::indexes::data::DerivedIndexBuildRequest {
                source_commit_id: head.commit_id,
                branch_id: branch.clone(),
                index_ids: vec![index.index_id],
            },
        );
        assert!(built.failed_indexes.is_empty(), "{built:?}");
        assert_eq!(built.generations.len(), 1);
        let generation = built.generations.into_iter().next().unwrap();
        let crate::indexes::data::DerivedIndexEntries::EntityField(entries) = &generation.entries
        else {
            panic!("actual status field generation required")
        };
        assert_eq!(
            entries
                .get(&new)
                .unwrap()
                .iter()
                .copied()
                .collect::<Vec<_>>(),
            vec![entity]
        );
        assert!(entries.get(&old).is_none());
        generations.push(generation);
    }

    // This independent resident producer exports the actual original Root columns,
    // including history, revision stamps and peer metadata, before the bare fold.
    let original = runtime
        .durability_authority()
        .native_checkpoint(execution, execution.allocation_policy())
        .unwrap();
    let original = crate::durability::log::native_checkpoint_codec::decode_checkpoint(
        execution,
        original.bytes(),
    )
    .unwrap()
    .checkpoint;
    let complete = runtime
        .complete_checkpoint_service()
        .prepare(
            execution,
            crate::durability::data::RelationalCheckpointArtifactPolicy::Embedded,
        )
        .expect("genuine complete multi-parent reconstruction");
    let complete = crate::durability::log::native_checkpoint_codec::decode_checkpoint(
        execution,
        complete.checkpoint().bytes(),
    )
    .unwrap()
    .checkpoint;
    for (commit, axes, root) in original_roots {
        let expected = original
            .branch_roots
            .iter()
            .find(|image| image.commit_id == commit)
            .expect("original selected Root image");
        let actual = complete
            .branch_roots
            .iter()
            .find(|image| image.commit_id == commit)
            .expect("Complete must preserve both current and feature Roots");
        assert_eq!(actual, expected, "all original Root columns must agree");
        assert_eq!(root.commit_id(), Some(commit));
        assert_eq!(root.axes(), axes);
    }
    // Complete carries its generations as checkpoint deltas beside an empty
    // full-cache slot. An independent build mints its own generation id; the
    // claim is the membership and source axes readmitted for each head.
    assert!(complete.derived_index_artifacts.generations().is_empty());
    let carried =
        crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(|lane| {
            complete
                .derived_index_checkpoint
                .as_ref()
                .expect("Complete carries its index checkpoint")
                .readmit(lane)
        })
        .expect("Complete index checkpoint readmits");
    for generation in generations {
        let actual = carried
            .iter()
            .find(|actual| {
                actual.index_id == generation.index_id
                    && actual.source_commit_id == generation.source_commit_id
                    && actual.source_branch_id == generation.source_branch_id
            })
            .unwrap_or_else(|| {
                let axes = carried
                    .iter()
                    .map(|actual| {
                        (
                            actual.index_id,
                            actual.source_commit_id,
                            actual.source_branch_id.clone(),
                        )
                    })
                    .collect::<Vec<_>>();
                panic!(
                    "Complete carries a generation for each branch head: {:?} {:?} {:?}                      missing from {axes:?}",
                    generation.index_id, generation.source_commit_id, generation.source_branch_id
                )
            });
        assert_eq!(
            (&actual.applicability, &actual.status, &actual.entries),
            (
                &generation.applicability,
                &generation.status,
                &generation.entries
            ),
            "Complete must preserve actual semantic membership and source axes"
        );
    }
}

// Fixture steps from the restore SHA's phase-D module (f9f15bf8c8), kept local
// to this court.
fn runtime_with_aspect_field_merge_policy(
    aspect_key: AspectKey,
    field_key: worth_foundational::facade::FieldKey,
    merge_policy: AspectMergePolicyKind,
) -> crate::facade::runtime::RelationalRuntime {
    let value_aspect = entity_field_aspect(aspect_key.clone(), field_key);
    let registry = register_aspect_field_merge_policy(aspect_key, value_aspect, merge_policy)
        .expect("schema registry");
    RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
        .expect("valid native runtime configuration")
}

fn create_branch_from_main(runtime: &crate::facade::runtime::RelationalRuntime, branch_name: &str) {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    runtime
        .history_authority()
        .fork_branch_from(
            execution,
            BranchId(branch_name.to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();
}

fn create_entity_with_aspect_fields(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &mut crate::facade::runtime::RelationalRuntime,
    client_key: &str,
    fields: crate::transactions::data::AspectFieldPatch,
) -> crate::facade::identity::EntityId {
    let mut targets = fields
        .iter()
        .map(|(target, value)| (target.clone(), value.clone()))
        .collect::<std::collections::BTreeMap<_, _>>();
    targets
        .entry(crate::transactions::data::planned_single_field_locator(
            AspectKey::new("name").expect("valid identity aspect key"),
            worth_foundational::facade::FieldKey::new("name").expect("valid identity field key"),
        ))
        .or_insert_with(|| crate::tests::support::string_aspect_value(client_key));
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_branch(
        runtime,
        BranchId("main".to_string()),
    );
    txn.push_batch(
        WorkerIntentBatch::new(format!("create-{client_key}")).push(MutationIntent::Create(
            CreateIntent::Entity(crate::transactions::data::EntitySpec {
                partition_id: crate::facade::identity::PartitionId::main(),
                kind_id: KindId(1),
                client_key: crate::symbols::data::ClientKey::raw(client_key),
                fields: crate::transactions::data::AspectFieldPatch::new(targets),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    changed_entities(&txn.commit(execution, runtime).unwrap())[0]
}

fn read_entity_aspect_field_display(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::facade::runtime::RelationalRuntime,
    branch: &BranchId,
    entity_id: crate::facade::identity::EntityId,
    aspect_key: AspectKey,
    field: worth_foundational::facade::FieldKey,
) -> String {
    let version_id = runtime
        .history()
        .branch_head(branch)
        .expect("branch head")
        .version_id;
    runtime
        .read_truth()
        .read_version(execution, version_id)
        .expect("read request remains live")
        .get_entity(entity_id)
        .and_then(|entity| read_entity_aspect_field(entity, aspect_key, field))
        .expect("aspect field display value")
}

fn rebuild_branch_index(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::facade::runtime::RelationalRuntime,
    branch: &str,
    index: DerivedIndexId,
) {
    let identity = runtime
        .branch_identity(&BranchId(branch.to_string()))
        .unwrap();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    runtime
        .index_authority()
        .reconstruct_for_basis(
            execution,
            crate::facade::indexes::DerivedIndexBuildRequest {
                source_commit_id: basis.observation().commit_id().unwrap(),
                branch_id: BranchId(branch.to_string()),
                index_ids: vec![index],
            },
            &basis,
            crate::facade::indexes::DerivedIndexMaintenanceBudget {
                maximum_work_units: None,
                maximum_cold_record_slots: None,
                maximum_derived_rows: None,
            },
        )
        .expect("branch index rebuilt for its admitted basis");
}
