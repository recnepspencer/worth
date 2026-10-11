use super::*;

#[test]
fn one_large_index_packet_stops_at_its_declared_ceiling_without_publication() {
    let _serial = serial();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(support::demo_schema_registry())
        .build();
    let identity = runtime.main_branch_identity();
    let (_, basis) = runtime.observe_branch(&identity).expect("main basis");
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .expect("admitted basis");
    let client_keys = (0..128)
        .map(|index| ClientKey::raw(format!("index-heavy-{index}")))
        .collect();
    let field_patches = (0..128)
        .map(|index| {
            AspectFieldPatch::from_locator(
                support::aspect_field_locator("name"),
                AspectValue::String(format!("index-heavy-{index}").into()),
            )
        })
        .collect();
    transaction
        .push_batch(
            WorkerIntentBatch::new("large-index-source").push(MutationIntent::Create(
                CreateIntent::BulkEntities(BulkEntityCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_keys,
                    field_patches,
                }),
            )),
            AllocationPolicy::SystemAllocation,
        )
        .expect("bulk source stages");
    let committed = runtime
        .commit_branch_transaction(transaction, AllocationPolicy::SystemAllocation)
        .expect("source commits");
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "large.index.name".to_owned(),
        kind: DerivedIndexKind::EntityField {
            field_locator: support::aspect_field_locator("name"),
        },
        branch_scoped: true,
    });
    let request = DerivedIndexBuildRequest {
        source_commit_id: committed.commit.commit_id,
        branch_id: BranchId("main".to_owned()),
        index_ids: vec![index.index_id],
    };
    let serial = runtime.index_authority().build_for_commit(request.clone());
    assert!(serial.execution_denial.is_none());
    let published = serial.generations[0].generation_id;
    // One worker's map reservation fits this policy; projection growth
    // reaches the packet's 8 KiB result ceiling: final Arc-backed entries
    // charge more bytes per unique key than their temporary scratch entries.
    let mut request_lease = lease_request(64 * 1024, CancellationToken::new());
    request_lease.policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Automatic,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::MIN, 64 * 1024, 1_000_000),
    );
    let lease = authority()
        .request_lease(request_lease)
        .expect("tight lease is admitted");
    let stopped = runtime
        .index_authority()
        .build_for_commit_with_lease(request.clone(), &lease);
    assert_eq!(
        stopped.execution_denial.map(|denial| denial.kind),
        Some(DerivedIndexExecutionDenialKind::Cause(
            Cause::ResultCapacityExceeded
        )),
    );
    assert!(stopped.generations.is_empty());
    assert_eq!(
        runtime
            .index_access()
            .latest_generation(index.index_id, &request.branch_id)
            .expect("prior index remains")
            .generation_id,
        published
    );
}
