//! Neither ordinary deletion nor producer Retire is a governed codec disposition.
use super::*;

#[test]
fn ordinary_deletion_and_producer_retire_never_reopen_a_birth_route_after_restore() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    for producer_retire in [false, true] {
        let mut runtime = runtime_with_test_schema();
        let receipt = runtime
            .prepare_initial_schema_installation()
            .unwrap()
            .install(crate::schema::data::RelationalSchemaRegistry::new())
            .unwrap();
        let (owner, installed) = receipt.issue_application_publication_owner();
        let (batch, targets) = creates(["original-left", "original-right"]);
        let born = publish(
            &runtime,
            execution,
            &installed,
            batch,
            native_birth_fixture(
                owner.admit_required_output("delete-does-not-vacate", [1; 32]),
                targets.to_vec(),
            ),
        )
        .unwrap();
        let ids = targets.map(|target| born.created_entity(&target).unwrap());
        let predecessor = born.required_output_publication(&installed).unwrap();
        release_test_commit_snapshot(&runtime, &born);
        let mut deletion = WorkerIntentBatch::new("actual-deletions-with-different-provenance");
        for id in ids {
            deletion = deletion.push(MutationIntent::Entity(EntityMutationIntent::Delete(
                DeleteEntityIntent { entity_id: id },
            )));
        }
        let committed = if producer_retire {
            let roles = ids
                .into_iter()
                .enumerate()
                .map(|(index, id)| RequiredOutputRoleTarget {
                    role: format!("retired-{index}"),
                    entity_name: "native-unit-output".into(),
                    kind: KindId(1),
                    posture: RequiredOutputRolePosture::Retire,
                    target: EntityReference::Existing(id),
                })
                .collect();
            publish(
                &runtime,
                execution,
                &installed,
                deletion,
                completion(&owner, predecessor, roles, 2),
            )
            .unwrap()
        } else {
            let mut transaction = test_owner_begin_transaction_for_main(&runtime);
            transaction
                .push_batch(deletion, execution.allocation_policy())
                .unwrap();
            transaction.commit(execution, &runtime).unwrap()
        };
        release_test_commit_snapshot(&runtime, &committed);
        let saved = runtime
            .durability_authority()
            .native_checkpoint(execution.allocation_policy())
            .unwrap();
        let mut cold = runtime_with_test_schema();
        let receipt = cold
            .prepare_initial_schema_installation()
            .unwrap()
            .install(crate::schema::data::RelationalSchemaRegistry::new())
            .unwrap();
        cold.durability_recovery()
            .restore_native_checkpoint(execution, &saved)
            .unwrap();
        let receipt = cold
            .readmit_recovered_initial_schema_installation(receipt)
            .unwrap();
        let (_, installed) = receipt.issue_application_publication_owner();
        let before = cold
            .durability_authority()
            .native_checkpoint(execution.allocation_policy())
            .unwrap();
        let basis = test_owner_main_basis(&cold).unwrap();
        let absence = cold
            .readmit_required_output_route_absence(
                execution,
                &installed,
                &basis,
                &crate::mvcc::native_meaning_fixture("delete-does-not-vacate"),
            )
            .unwrap();
        assert!(
            absence.is_none(),
            "ordinary deletion cannot authorize a new birth"
        );
        assert_eq!(
            cold.durability_authority()
                .native_checkpoint(execution.allocation_policy())
                .unwrap(),
            before
        );
    }
}
