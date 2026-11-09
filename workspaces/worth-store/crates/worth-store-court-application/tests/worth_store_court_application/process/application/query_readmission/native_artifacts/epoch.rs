//! Real Home operations retain the receiver epoch that admitted their work.
use super::*;
use worth_foundational::facade::{AspectKey, AspectValue, FieldKey};
use worth_relational::facade::{mvcc::*, runtime::*, transactions::*};
mod target_inventory;

fn batch(id: worth_relational::facade::identity::EntityId) -> WorkerIntentBatch {
    WorkerIntentBatch::new("actual epoch control update").push(MutationIntent::Entity(
        EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
            entity_id: id,
            fields: AspectFieldPatch::from_locator(
                planned_single_field_locator(
                    AspectKey::new(
                        FieldSelection::<HomeRecords, Account, AccountIdentity, u64>::at("Balance")
                            .aspect(),
                    )
                    .unwrap(),
                    FieldKey::new("Balance").unwrap(),
                ),
                AspectValue::UInt64(29),
            ),
        }),
    ))
}

#[test]
fn equivalent_receiver_replacement_refuses_retained_proposals_and_candidates_before_movement() {
    let directory = tempfile::tempdir().unwrap();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = ExecutionRequest::leased(&lease);
    fresh_closed(directory.path(), execution);
    with_bootstrap_advancement_from_request(execution, |phase| {
        let (query, mut home) = unshared(directory.path(), &phase);
        let (native, _, _) = home.recovered_query_finalization_parts().unwrap();
        let receipt = native
            .readmit_recovered_initial_schema_installation(
                query.home_initial_schema_receipt().clone(),
            )
            .unwrap();
        let generation = receipt.custom_invariant_generation();
        let runtime = native.runtime_instance_id();
        let (_, basis) = native
            .observe_branch(&native.main_branch_identity())
            .unwrap();
        let rows = administration::rows(native, &execution, "Account");
        let [row] = rows.as_slice() else {
            panic!("actual nonempty Home account");
        };
        let original = administration::field(
            row,
            FieldSelection::<HomeRecords, Account, AccountIdentity, u64>::at("Balance").aspect(),
            "Balance",
        )
        .clone();
        let validate = |native: &RelationalRuntime| {
            let mut transaction = native
                .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
                .unwrap();
            transaction
                .push_batch(batch(row.entity_id), execution.allocation_policy())
                .unwrap();
            transaction.validate(&execution, native).unwrap()
        };
        let proposal = validate(native);
        let candidate = native
            .prepare_validated_proposal(&execution, validate(native))
            .unwrap();
        native
            .replace_equivalent_configuration_epoch_for_test()
            .unwrap();
        assert_eq!(native.runtime_instance_id(), runtime);
        let receipt = native
            .readmit_recovered_initial_schema_installation(
                query.home_initial_schema_receipt().clone(),
            )
            .unwrap();
        assert_eq!(receipt.custom_invariant_generation(), generation);
        let (_, current) = native
            .observe_branch(&native.main_branch_identity())
            .unwrap();
        assert_eq!(current.descriptor(), basis.descriptor());
        assert_eq!(current.admission_identity(), basis.admission_identity());
        let error = native
            .prepare_validated_proposal(&execution, proposal)
            .expect_err("old proposal refuses");
        let TransactionCommitError::Conflict { error, .. } = error else {
            panic!("exact stale validation refusal");
        };
        assert!(matches!(
            error.class,
            ConflictClass::StaleValidationBasis { .. }
        ));
        assert!(
            error.detail.contains("obsolete receiver epoch"),
            "{error:?}"
        );
        assert!(matches!(
            native
                .publication_port()
                .compare_and_publish(&execution, candidate),
            RelationalPublicationOutcome::Denied(
                RelationalPublicationDenial::StaleConfigurationEpoch
            )
        ));
        let (_, after) = native
            .observe_branch(&native.main_branch_identity())
            .unwrap();
        assert_eq!(after.descriptor(), basis.descriptor());
        let after_rows = administration::rows(native, &execution, "Account");
        assert_eq!(
            administration::field(
                &after_rows[0],
                FieldSelection::<HomeRecords, Account, AccountIdentity, u64>::at("Balance")
                    .aspect(),
                "Balance"
            ),
            &original
        );
        // A current operation still validates and prepares at precisely that basis.
        drop(
            native
                .prepare_validated_proposal(&execution, validate(native))
                .unwrap(),
        );
        drop((basis, current, after, query));
        assert!(matches!(
            home.close(&execution).unwrap(),
            ApplicationHomeCloseOutcome::Closed { .. }
        ));
    })
    .unwrap();
}
