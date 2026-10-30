pub(crate) use {
    crate::leased_preparation::ordinary_context::authority,
    crate::leased_preparation::ordinary_context::lease_request,
    crate::leased_preparation::ordinary_context::serial,
    worth_relational::facade::identity::KindId,
    worth_relational::facade::transactions::BulkEntityCreateIntent,
};

#[test]
fn leased_bulk_creation_matches_serial_canonical_patch() {
    if worth_testkit_relational::test_process::reexecute_isolated_test() {
        return;
    }

    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let _serial = serial();
    fn commit_bulk(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> worth_relational::facade::transactions::CommitResult {
        let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
            .schema_registry(
                crate::leased_preparation::ordinary_context::support::demo_schema_registry(),
            )
            .build()
            .expect("valid native runtime configuration");
        let identity = runtime.main_branch_identity();
        let (_, basis) = runtime.observe_branch(&identity).expect("main basis");
        let mut transaction = runtime
            .begin_branch_transaction(
                &basis,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .expect("admitted basis");
        let keys = (0..64)
            .map(|index| worth_relational::facade::symbols::ClientKey::raw(format!("bulk-{index}")))
            .collect();
        let fields = (0..64)
            .map(|index| {
                worth_relational::facade::transactions::AspectFieldPatch::from_locator(
                    crate::leased_preparation::ordinary_context::support::aspect_field_locator(
                        "name",
                    ),
                    worth_foundational::facade::AspectValue::String(format!("bulk-{index}").into()),
                )
            })
            .collect();
        transaction
            .push_batch(
                worth_relational::facade::transactions::WorkerIntentBatch::new("leased-bulk").push(
                    worth_relational::facade::transactions::MutationIntent::Create(
                        worth_relational::facade::transactions::CreateIntent::BulkEntities(
                            BulkEntityCreateIntent {
                                partition_id: worth_relational::facade::identity::PartitionId::main(
                                ),
                                kind_id: KindId(1),
                                client_keys: keys,
                                field_patches: fields,
                            },
                        ),
                    ),
                ),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect("bulk transaction stages");
        runtime
            .commit_branch_transaction(execution, transaction)
            .expect("bulk commit succeeds")
    }

    let serial = commit_bulk(execution);
    let lease = authority()
        .request_lease(lease_request(
            32 * 1024 * 1024,
            worth_execution::CancellationToken::new(),
        ))
        .expect("bulk lease admitted");
    let leased = commit_bulk(&worth_execution::ExecutionRequest::leased(&lease));
    assert_eq!(serial.changed_records.len(), 64);
    assert_eq!(serial.changed_records, leased.changed_records);
    assert_eq!(serial.patch(), leased.patch());
}
