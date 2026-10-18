//! A genuine optional survivor cannot disappear from described output membership.
use super::{super::fixture::*, staged};
use crate::{
    identity::data::{EntityId, KindId, PartitionId},
    mvcc::{RelationalPublicationOutcome, RelationalTransactionIntent},
    symbols::data::ClientKey,
    transactions::data::*,
};
use worth_foundational::facade::{AspectValue, InternedString};
fn publish(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
    installed: &crate::mvcc::RelationalInstalledApplicationPublicationAuthority,
    proof: worth_proof::AdmittedRequiredOutputPublication,
) -> [EntityId; 2] {
    let targets =
        [(3, "mixed-changed"), (1, "optional-stable")].map(|(kind, key)| CreatedEntityRef {
            partition_id: PartitionId::main(),
            kind_id: KindId(kind),
            client_key: ClientKey::raw(key),
        });
    let mut batch = WorkerIntentBatch::new("genuine mixed publication");
    for (index, target) in targets.iter().enumerate() {
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: target.partition_id,
            kind_id: target.kind_id,
            client_key: target.client_key.clone(),
            fields: AspectFieldPatch::from_locator(
                locator(if index == 0 { "derived" } else { "source" }),
                AspectValue::String(InternedString::from(if index == 0 {
                    r#"{"old":44}"#
                } else {
                    "optional unchanged"
                })),
            ),
        })));
    }
    let mut transaction = runtime
        .begin_branch_transaction(&basis(runtime), RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(batch, execution.allocation_policy())
        .unwrap();
    let proposal = transaction
        .validate(execution, runtime)
        .unwrap()
        .bind_required_output_publication(
            installed,
            crate::mvcc::native_birth_fixture(proof, targets.to_vec()),
        )
        .unwrap();
    let candidate = runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("actual native producer publication must perform")
    };
    let committed = runtime.settle_performed_publication(performed).unwrap();
    let ids = targets.map(|target| committed.created_entity(&target).unwrap());
    runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    ids
}
#[test]
fn companion_optional_survivor_omission_cannot_disguise_a_genuine_mixed_publication() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let (runtime, receipt) = installed(1);
    let (owner, installed) = receipt.clone().issue_application_publication_owner();
    let world = world_with_installation(execution, runtime, receipt, true);
    let [a, b] = publish(
        execution,
        &world.runtime,
        &installed,
        owner.admit_required_output("native.fixture.mixed", [11; 32]),
    );
    let (mut target, recovered, selected, _transaction) = staged(execution, &world);
    assert!(recovered.codec_transition_retires(a));
    assert!(!recovered.codec_transition_retires(b));
    let before = capture(&target);
    // The complete association is genuine; Query must still refuse its MIXED
    // disposition. A copied subset cannot masquerade as either all or none.
    target
        .durability_recovery()
        .validate_recovered_codec_output_correspondence(
            execution,
            &recovered,
            &selected,
            "native.fixture.mixed",
            &[11; 32],
            &crate::mvcc::native_meaning_fixture("native.fixture.mixed"),
            &crate::mvcc::native_roles_fixture(
                &[(a, KindId(3)), (b, KindId(1))],
                crate::mvcc::RequiredOutputRolePosture::Create,
            ),
        )
        .unwrap();
    let original_roles = crate::mvcc::native_roles_fixture(
        &[(a, KindId(3)), (b, KindId(1))],
        crate::mvcc::RequiredOutputRolePosture::Create,
    );
    for role in &original_roles {
        let denied = target
            .durability_recovery()
            .validate_recovered_codec_output_correspondence(
                execution,
                &recovered,
                &selected,
                "native.fixture.mixed",
                &[11; 32],
                &crate::mvcc::native_meaning_fixture("native.fixture.mixed"),
                std::slice::from_ref(role),
            )
            .unwrap_err();
        assert!(
            format!("{denied:?}").contains("omits or adds native producer output"),
            "{denied:?}"
        );
    }
    assert!(selected.is_current());
    assert_eq!(capture(&target), before);
}
