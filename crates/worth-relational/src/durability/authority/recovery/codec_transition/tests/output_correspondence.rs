//! Actual surviving native publication, exact probes and original request stops.
use super::{fixture::*, recovered, RelationalOperationControl};
use crate::identity::data::{EntityId, KindId};
fn independent(execution: &worth_execution::ExecutionRequest<'_, '_>) -> (World, EntityId) {
    let (runtime, receipt) = installed(1);
    let (owner, installed) = receipt.clone().issue_application_publication_owner();
    let world = world_with_installation(execution, runtime, receipt, true);
    let companion = create(
        execution,
        &world.runtime,
        1,
        "companion",
        "independent stable payload",
        Some((
            &installed,
            owner.admit_required_output("native.fixture.companion", [10; 32]),
        )),
    );
    (world, companion)
}
fn staged(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &World,
) -> (
    crate::runtime::RelationalRuntime,
    super::RecoveredRelationalRuntimeAuthority,
    crate::branch::AdmittedRelationalBranchBasis,
    crate::mvcc::BranchBoundRelationalTransaction,
) {
    let (mut target, mut recovered, _) = recovered(execution, world);
    let selected = basis(&target);
    let (owner, installed) = target
        .durability_recovery()
        .issue_recovered_codec_source_publication_owner(execution, &recovered)
        .unwrap();
    let proof = activation_proof(
        execution,
        &mut target,
        &recovered,
        &selected,
        &owner,
        world.activation,
    );
    let transaction = target
        .durability_recovery()
        .begin_recovered_codec_transition(
            execution,
            &mut recovered,
            &selected,
            RelationalOperationControl::uninterrupted(),
            &installed,
            proof,
            activation(world.activation),
        )
        .unwrap();
    (target, recovered, selected, transaction)
}
#[test]
fn companion_native_association_requires_exact_identity_kind_and_original_payload() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let (world, companion) = independent(execution);
    let (mut target, recovered, selected, _transaction) = staged(execution, &world);
    let before = capture(&target);
    target
        .durability_recovery()
        .validate_recovered_codec_output_correspondence(
            execution,
            &recovered,
            &selected,
            "native.fixture.companion",
            &[10; 32],
            &crate::mvcc::native_meaning_fixture("native.fixture.companion"),
            &crate::mvcc::native_roles_fixture(
                &[(companion, KindId(1))],
                crate::mvcc::RequiredOutputRolePosture::Create,
            ),
        )
        .unwrap();
    for roles in [
        vec![],
        vec![(companion, KindId(3))],
        vec![(
            EntityId::new(companion.partition_id, companion.local_slot_value(), 0),
            KindId(1),
        )],
        vec![(companion, KindId(1)), (world.authored, KindId(1))],
    ] {
        assert!(target
            .durability_recovery()
            .validate_recovered_codec_output_correspondence(
                execution,
                &recovered,
                &selected,
                "native.fixture.companion",
                &[10; 32],
                &crate::mvcc::native_meaning_fixture("native.fixture.companion"),
                &crate::mvcc::native_roles_fixture(
                    &roles,
                    crate::mvcc::RequiredOutputRolePosture::Create
                )
            )
            .is_err());
    }
    assert!(target
        .durability_recovery()
        .validate_recovered_codec_output_correspondence(
            execution,
            &recovered,
            &selected,
            "copied.description",
            &[10; 32],
            &crate::mvcc::native_meaning_fixture("native.fixture.companion"),
            &crate::mvcc::native_roles_fixture(
                &[(companion, KindId(1))],
                crate::mvcc::RequiredOutputRolePosture::Create
            )
        )
        .is_err());
    assert_eq!(capture(&target), before);
    // Equal ordinary writes cannot inherit the original producer association.
    let mut transaction = world
        .runtime
        .begin_branch_transaction(
            &basis(&world.runtime),
            crate::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    transaction
        .push_batch(
            crate::transactions::data::WorkerIntentBatch::new("ordinary companion rewrite").push(
                crate::transactions::data::MutationIntent::Entity(
                    crate::transactions::data::EntityMutationIntent::UpdateFields(
                        crate::transactions::data::UpdateEntityFieldsIntent {
                            entity_id: companion,
                            fields: crate::transactions::data::AspectFieldPatch::from_locator(
                                locator("source"),
                                worth_foundational::facade::AspectValue::String(
                                    worth_foundational::facade::InternedString::from(
                                        "independent stable payload",
                                    ),
                                ),
                            ),
                        },
                    ),
                ),
            ),
            execution.allocation_policy(),
        )
        .unwrap();
    let committed = transaction.commit(execution, &world.runtime).unwrap();
    world
        .runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    let (mut rewritten, recovered, selected, _transaction) = staged(execution, &world);
    assert!(rewritten
        .durability_recovery()
        .validate_recovered_codec_output_correspondence(
            execution,
            &recovered,
            &selected,
            "native.fixture.companion",
            &[10; 32],
            &crate::mvcc::native_meaning_fixture("native.fixture.companion"),
            &crate::mvcc::native_roles_fixture(
                &[(companion, KindId(1))],
                crate::mvcc::RequiredOutputRolePosture::Create
            )
        )
        .is_err());
}
#[test]
fn companion_original_funding_and_cancellation_refuse_without_effects_then_fresh_retry() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let (world, companion) = independent(execution);
    let (mut target, recovered, selected, _transaction) = staged(execution, &world);
    let before = capture(&target);
    let limited = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(Some(0)),
        worth_execution::CancellationSource::new().token(),
        None,
    );
    let denied = target
        .durability_recovery()
        .validate_recovered_codec_output_correspondence(
            &worth_execution::ExecutionRequest::serial(&limited),
            &recovered,
            &selected,
            "native.fixture.companion",
            &[10; 32],
            &crate::mvcc::native_meaning_fixture("native.fixture.companion"),
            &crate::mvcc::native_roles_fixture(
                &[(companion, KindId(1))],
                crate::mvcc::RequiredOutputRolePosture::Create,
            ),
        )
        .unwrap_err();
    assert!(
        denied.request_cause().is_some(),
        "scratch must be funded before allocation: {denied:?}"
    );
    let cancel = worth_execution::CancellationSource::new();
    let stopped = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(Some(64 * 1024 * 1024)),
        cancel.token(),
        None,
    );
    cancel.cancel();
    let denied = target
        .durability_recovery()
        .validate_recovered_codec_output_correspondence(
            &worth_execution::ExecutionRequest::serial(&stopped),
            &recovered,
            &selected,
            "wrong producer",
            &[0; 32],
            &crate::mvcc::native_meaning_fixture("native.fixture.companion"),
            &crate::mvcc::native_roles_fixture(&[], crate::mvcc::RequiredOutputRolePosture::Create),
        )
        .unwrap_err();
    assert_eq!(
        denied.request_cause(),
        Some(&crate::execution::RelationalExecutionDenialCause::Cancelled)
    );
    assert!(selected.is_current());
    assert_eq!(capture(&target), before);
    target
        .durability_recovery()
        .validate_recovered_codec_output_correspondence(
            execution,
            &recovered,
            &selected,
            "native.fixture.companion",
            &[10; 32],
            &crate::mvcc::native_meaning_fixture("native.fixture.companion"),
            &crate::mvcc::native_roles_fixture(
                &[(companion, KindId(1))],
                crate::mvcc::RequiredOutputRolePosture::Create,
            ),
        )
        .unwrap();
}

mod mixed_publication;
