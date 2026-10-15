use super::*;

#[test]
fn leased_bulk_creation_matches_serial_canonical_patch() {
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let _serial = serial();
    fn commit_bulk(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> worth_relational::facade::transactions::CommitResult {
        let runtime = RelationalRuntimeApi::builder()
            .schema_registry(support::demo_schema_registry())
            .build();
        let identity = runtime.main_branch_identity();
        let (_, basis) = runtime.observe_branch(&identity).expect("main basis");
        let mut transaction = runtime
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .expect("admitted basis");
        let keys = (0..64)
            .map(|index| ClientKey::raw(format!("bulk-{index}")))
            .collect();
        let fields = (0..64)
            .map(|index| {
                AspectFieldPatch::from_locator(
                    support::aspect_field_locator("name"),
                    AspectValue::String(format!("bulk-{index}").into()),
                )
            })
            .collect();
        transaction
            .push_batch(
                WorkerIntentBatch::new("leased-bulk").push(MutationIntent::Create(
                    CreateIntent::BulkEntities(BulkEntityCreateIntent {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(1),
                        client_keys: keys,
                        field_patches: fields,
                    }),
                )),
                AllocationPolicy::SystemAllocation,
            )
            .expect("bulk transaction stages");
        runtime
            .commit_branch_transaction(execution, transaction)
            .expect("bulk commit succeeds")
    }

    let serial = commit_bulk(execution);
    let lease = authority()
        .request_lease(lease_request(32 * 1024 * 1024, CancellationToken::new()))
        .expect("bulk lease admitted");
    let leased = commit_bulk(&worth_execution::ExecutionRequest::leased(&lease));
    assert_eq!(serial.changed_records.len(), 64);
    assert_eq!(serial.changed_records, leased.changed_records);
    assert_eq!(serial.patch(), leased.patch());
}
