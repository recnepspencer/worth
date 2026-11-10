//! Exact original field identity survives cold, Head and Full uniqueness checks.
use super::{
    candidate_source::runtime, current_publications::create_entity,
    immutable_recovery_input::copy_input, symbol_text_publications::*,
};
use std::sync::{atomic::Ordering, Arc};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::{durability::*, identity::*, symbols::ClientKey, transactions::*};
use worth_testkit_relational::{
    aspect_storage::{
        aspect_field_locator, aspect_key, field_key, publications::begin_main_transaction,
    },
    cold_recovery::capture_source,
    durable_medium::TestDurableMedium,
};
#[test]
fn cold_head_warm_and_full_warm_uniqueness_reuse_the_original_paged_field_identity() {
    let host=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = runtime(&medium);
    admit_population(execution, &producer);
    let (entity, _, symbol) = publish_text(
        execution,
        &producer,
        TextMutation::Create,
        "paged-view-value-0600",
    );
    let extra = create_entity(execution, &producer, "member-before");
    let original = capture_source(execution, producer, medium, extra, false);
    for warmth in 0..3 {
        let (input, source) = copy_input(&original);
        let medium = TestDurableMedium::new();
        let attachment = medium.attachment();
        for bytes in source.commits.values() {
            let a = RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone()).unwrap();
            assert!(matches!(
                attachment.publish(&a),
                DurablePublicationFate::Committed
            ));
        }
        let mut receiving = runtime(&medium);
        let plan = receiving
            .durability()
            .recovery_plan_from_source(
                execution,
                input,
                RecoveryVerificationMode::NormalRecoveryVerification,
            )
            .unwrap();
        receiving
            .durability_recovery()
            .recover(execution, plan)
            .unwrap();
        assert_eq!(source.reads.load(Ordering::Relaxed), 0);
        if warmth == 1 {
            let mut read = begin_main_transaction(&receiving);
            assert!(read
                .read_entity(execution, entity)
                .unwrap()
                .effective()
                .is_some());
        } else if warmth == 2 {
            let (_, basis) = receiving
                .observe_branch(&receiving.main_branch_identity())
                .unwrap();
            let snapshot = receiving
                .snapshots()
                .snapshot_for_observation(&basis.observation())
                .unwrap();
            assert!(receiving
                .read_truth()
                .read_snapshot(&snapshot, execution)
                .unwrap()
                .is_some());
            receiving.snapshots().release_snapshot(&snapshot).unwrap();
        }
        let frontier = receiving.history().latest_patch_stream_position();
        let offers = medium.offered().len();
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
            AspectValue::String(InternedString::Symbol(worth_foundational::facade::Symbol(
                symbol.0
            )))
        );
        let mut transaction = begin_main_transaction(&receiving);
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
            panic!("duplicate remains typed uniqueness conflict")
        };
        assert_eq!(duplicate, value);
        assert_eq!(receiving.history().latest_patch_stream_position(), frontier);
        assert_eq!(medium.offered().len(), offers);
        assert_eq!(
            receiving.patch_position_reservation_counters(),
            reservations
        );
    }
}
