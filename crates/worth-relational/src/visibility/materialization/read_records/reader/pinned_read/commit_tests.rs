use super::*;
use crate::config::data::CascadeDeletePolicy;
use crate::identity::data::PartitionId;
use crate::query::data::*;
use crate::tests::support::*;
use worth_execution::ExecutionRequest;

#[test]
fn pinned_traversal_keeps_its_frontier_across_a_commit() {
    let host = crate::execution::test_policy::serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let a = create_entity_in_partition(&request, &runtime, "a", PartitionId(7));
    let b = create_entity_in_partition(&request, &runtime, "b", PartitionId(7));
    let ab = create_relation_in_partition(&request, &runtime, a, b, "ab", PartitionId(7));
    let snapshot = runtime
        .visibility_authority()
        .snapshot()
        .expect("current snapshot fixture is admitted");
    let pinned = runtime
        .read_truth()
        .pin_read_capability(&snapshot, &[], &request)
        .unwrap();
    let plan = runtime
        .read_truth()
        .plan_query_packet(
            &snapshot,
            PlannedQueryPacket {
                label: "held-frontier".into(),
                context_id: pinned.query_context().clone(),
                scope: QueryScope::ConnectivityTraversal {
                    seeds: Arc::from([a]),
                    relation_kind_scope: None,
                    max_depth: None,
                },
                locality: QueryLocalityClass::CrossPartitionTraversal,
                ordering: QueryOrderingContract::CanonicalTraversalOrder,
                access_contract: QueryAccessContract::AuthoritativeStorageOnly,
                execution_shape: QueryExecutionShape::BulkPacketized,
                reduction: ReductionDiscipline::DeterministicMerge,
                plan_key: DeterministicQueryPlanKey(917652),
                target_count_hint: 1,
            },
        )
        .unwrap();
    let c = create_entity_in_partition(&request, &runtime, "c", PartitionId(7));
    create_relation_in_partition(&request, &runtime, b, c, "bc", PartitionId(7));
    let answer = pinned.execute_query_plan(plan, &request).unwrap().unwrap();
    assert_eq!(
        answer
            .outcome
            .result
            .entities
            .iter()
            .map(|row| row.entity_id)
            .collect::<Vec<_>>(),
        vec![a, b]
    );
    assert_eq!(
        answer
            .outcome
            .result
            .relations
            .iter()
            .map(|row| row.relation_id)
            .collect::<Vec<_>>(),
        vec![ab]
    );
    assert_eq!(
        answer.outcome.plan.packet.context_id.version_id,
        crate::identity::data::VersionId(3)
    );
}

#[test]
fn exact_view_pin_keeps_original_root_after_commit_snapshot_release_and_rejects_foreign_runtime() {
    let host = crate::execution::test_policy::serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let committed = create_entity_outcome(&request, &runtime, "original-view");
    let original_entity = changed_entities(&committed)[0];
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    assert_eq!(
        basis.observation().commit_receipt(),
        Some(&committed.commit)
    );
    let view = runtime
        .read_truth()
        .project_observation(&basis.observation())
        .unwrap();
    release_test_commit_snapshot(&runtime, &committed);
    assert_eq!(
        runtime.snapshots().retained_snapshot_for_commit(runtime.runtime_instance_id(), &committed.commit)
            .unwrap_err().kind(),
        crate::visibility::exact_commit_snapshot::RelationalRetainedCommitSnapshotDenialKind::SnapshotNotRetained,
    );
    let foreign = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    assert!(matches!(
        foreign
            .read_truth()
            .pin_view_read_capability(&view, &[], &request),
        Err(RelationalPinnedReadAdmissionDenial::SnapshotUnavailable),
    ));
    let pin = runtime
        .read_truth()
        .pin_view_read_capability(&view, &[], &request)
        .unwrap();
    assert_eq!(pin.snapshot, ());
    assert_eq!(pin.context.version_id, committed.commit.version_id);
    assert!(Arc::ptr_eq(pin.basis.root(), view.selected_root().unwrap()));
    let later = create_entity_outcome(&request, &runtime, "later-view");
    let later_entity = changed_entities(&later)[0];
    release_test_commit_snapshot(&runtime, &later);
    drop(later);
    assert_eq!(pin.context.version_id, committed.commit.version_id);
    drop(view);
    drop(basis);
    drop(committed);
    drop(runtime);
    drop(foreign);
    use crate::visibility::materialization::read_records::projection::ProjectionRecordSource;
    use crate::visibility::materialization::read_records::{read_stop::ReadStop, read_work};
    use worth_foundational::facade::{
        AspectValue, ContractValidatedAspectValueView, InternedString,
    };
    for (entity, expected_present) in [(original_entity, true), (later_entity, false)] {
        let seen = read_work::metered_with_memory(
            &request,
            |context, memory| {
                pin.with_entity_record(
                    &request,
                    entity,
                    |work, bytes| {
                        context
                            .checkpoint(work)
                            .map_err(crate::execution::denial::kernel_stop)?;
                        memory.resize(bytes).map_err(|cause| {
                            crate::execution::denial::lease_denial(
                                worth_execution::LeaseDenial::MemoryExhausted(cause),
                            )
                        })
                    },
                    |record| {
                        assert_eq!(record.entity_id, entity);
                        let state = record
                            .authoritative_aspect_state
                            .as_ref()
                            .expect("original payload");
                        // The generic birth uses a Raw string in its scalar EntityField contract.
                        let ContractValidatedAspectValueView::Scalar(AspectValue::String(
                            InternedString::Raw(text),
                        )) = state
                            .get(&aspect_key("name"))
                            .expect("original name aspect")
                            .view()
                        else {
                            panic!("the original scalar payload must retain its plain string");
                        };
                        assert_eq!(text, "original-view");
                    },
                )
                .map_err(|stop| match stop {
                    ReadStop::Request(cause) | ReadStop::Admission(cause) => cause,
                })
            },
            |_| 0, // Only an optional unit escapes; native temporary record custody is settled above.
        )
        .expect("original request funds the pinned read")
        .expect("the pinned record read succeeds");
        assert_eq!(
            seen.is_some(),
            expected_present,
            "the original pin excludes the later entity"
        );
    }
}
