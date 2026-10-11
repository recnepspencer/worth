use super::*;

#[test]
fn stale_native_contract_basis_denies_without_mutating_truth() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let entity = create_entity(execution, &runtime, "before");
    let name = contract_for_entity(&runtime, "name");
    let stale = PortableRecordAspectPatch::new([PortableAspectPatchOperation::SetWhole {
        basis: PortableAspectContractBasis::new(
            name.key().clone(),
            name.identity(),
            worth_foundational::facade::AspectContractRevision(name.revision().0 + 1),
        ),
        value: ContractValidationInput::Scalar(text("after")),
    }]);
    let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("stale-native-basis").push(MutationIntent::Entity(
                EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                    entity_id: entity,
                    aspect_patch: stale,
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let error = transaction
        .commit(
            execution,
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect_err("stale basis must deny");
    assert!(matches!(
        error,
        TransactionCommitError::Conflict {
            error: CommitConflict {
                class: ConflictClass::RecordAspectPatchDenied { .. },
                ..
            },
            ..
        }
    ));
    let latest = runtime.visibility_authority().snapshot();
    let read = runtime
        .read_truth()
        .read_snapshot(&latest, execution)
        .unwrap();
    let value = read
        .get_entity(entity)
        .unwrap()
        .authoritative_aspect_state
        .as_ref()
        .unwrap()
        .get(name.key())
        .unwrap();
    assert!(matches!(
        value.view(),
        ContractValidatedAspectValueView::Scalar(AspectValue::String(value))
            if value == &"before".into()
    ));
    drop(read);
    runtime.snapshots().release_snapshot(&latest).unwrap();
}
