//! Stable paged symbol IDs remain equal in selected-state uniqueness checks.
use super::super::cold_symbols::view_levels::{admit_population, publish_text, TextMutation};
use super::*;
use crate::transactions::data::{
    AspectFieldPatch, CommitConflict, ConflictClass, CreateIntent, EntitySpec, MutationIntent,
};
use crate::validation::data::InvariantViolationFields;
use worth_execution::ExecutionImmutableBytes;
#[test]
fn cold_head_warm_and_full_warm_uniqueness_reuse_the_original_paged_field_identity() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for warmth in 0..3 {
        let producer = runtime(TestDurableMedium::new().attachment());
        admit_population(execution, &producer);
        let (entity, _) = publish_text(
            execution,
            &producer,
            TextMutation::Create,
            "paged-view-value-0600",
        );
        let symbol = producer
            .services
            .symbols
            .interner_snapshot()
            .symbol("paged-view-value-0600")
            .unwrap();
        let (recovery, source, _) = fixture_from_runtime(execution, producer, false, 0);
        source.damaged.store(false, Ordering::Relaxed);
        let medium = TestDurableMedium::new().attachment();
        for bytes in source.commits.values() {
            let artifact = crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone()).unwrap();
            assert!(matches!(
                medium.publish(&artifact),
                crate::durability::medium::RelationalDurablePublication::Committed
            ));
        }
        let mut receiving = runtime(medium);
        let plan = receiving
            .durability()
            .recovery_plan_from_source(
                execution,
                recovery,
                RecoveryVerificationMode::NormalRecoveryVerification,
            )
            .unwrap();
        receiving
            .durability_recovery()
            .recover(execution, plan)
            .unwrap();
        assert_eq!(source.reads.load(Ordering::Relaxed), 0);
        assert!(receiving
            .services
            .symbols
            .interner_snapshot()
            .resolve(symbol)
            .is_none());
        if warmth == 1 {
            let mut read = test_owner_begin_transaction_for_main(&receiving);
            assert!(read
                .read_entity(execution, entity)
                .unwrap()
                .effective()
                .is_some());
        } else if warmth == 2 {
            let read = test_owner_begin_transaction_for_main(&receiving);
            receiving
                .selected_branch_state(&read.basis)
                .unwrap()
                .state()
                .root()
                .demand_all_full(execution, &receiving.services.symbols)
                .unwrap();
        }
        let frontier = receiving.history().latest_patch_stream_position();
        let offers = attached_test_medium(&receiving).offered().len();
        let reservations = receiving.patch_position_reservation_counters();
        let (_, basis) = receiving
            .observe_branch(&receiving.main_branch_identity())
            .unwrap();
        let input = ExecutionImmutableBytes::from_external_bytes(Arc::new(
            b"paged-view-value-0600".as_slice().into(),
        ));
        let text = receiving
            .admit_field_text(execution, &basis, &input)
            .unwrap();
        let value = text.field_value_for(&basis).unwrap();
        assert_eq!(
            value,
            worth_foundational::facade::AspectValue::String(
                worth_foundational::facade::InternedString::Symbol(
                    worth_foundational::facade::Symbol(symbol.0)
                )
            )
        );
        let mut transaction = test_owner_begin_transaction_for_main(&receiving);
        transaction
            .push_batch(
                WorkerIntentBatch::new("duplicate-paged-field").push(MutationIntent::Create(
                    CreateIntent::Entity(EntitySpec {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(1),
                        client_key: ClientKey::raw("duplicate-paged-field"),
                        fields: AspectFieldPatch::from_locator(
                            aspect_field_locator(aspect_key("name"), field_key("name")),
                            value.clone(),
                        ),
                    }),
                )),
                execution.allocation_policy(),
            )
            .unwrap();
        let TransactionCommitError::Conflict {
            error:
                CommitConflict {
                    class:
                        ConflictClass::InvariantViolation {
                            fields:
                                InvariantViolationFields::UniqueEntityField {
                                    value: duplicate, ..
                                },
                            ..
                        },
                    ..
                },
            ..
        } = transaction.commit(execution, &receiving).unwrap_err()
        else {
            panic!("the duplicate must be a typed uniqueness conflict");
        };
        assert_eq!(duplicate, value);
        assert_eq!(receiving.history().latest_patch_stream_position(), frontier);
        assert_eq!(attached_test_medium(&receiving).offered().len(), offers);
        assert_eq!(
            receiving.patch_position_reservation_counters(),
            reservations
        );
    }
}
