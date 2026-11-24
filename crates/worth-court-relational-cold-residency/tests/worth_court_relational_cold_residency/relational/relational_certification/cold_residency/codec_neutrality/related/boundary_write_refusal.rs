//! The original producer cannot reclaim a boundary edge overwritten by an ordinary edit.
use super::{
    super::{
        fixture::{basis, capture},
        publication_fixture, serial_request,
    },
    fixture::*,
    incoming_reference::{installed_references, relation_payload},
};
use worth_relational::facade::{
    identity::KindId,
    mvcc::{RelationalTransactionIntent, RequiredOutputRolePosture},
    transactions::*,
};

#[test]
fn owned_boundary_equal_write_and_rewire_refuse_warm_and_cold_before_effects() {
    let request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    for rewire in [false, true] {
        let (runtime, receipt) = installed_references(1);
        let (world, owned) = world_with_boundary(execution, runtime, receipt);
        let (edge, publisher) = owned.unwrap();
        let ProducerPublication {
            owner,
            installed,
            custody: previous,
        } = publisher;
        let mut transaction = world
            .runtime
            .begin_branch_transaction(
                &basis(&world.runtime),
                RelationalTransactionIntent::ordinary(),
            )
            .unwrap();
        transaction
            .push_batch(
                WorkerIntentBatch::new("ordinary boundary overwrite").push(
                    MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
                        UpdateRelationEndpointsIntent {
                            relation_id: edge,
                            kind_id: KindId(4),
                            source: EntityReference::Existing(world.outputs[usize::from(rewire)]),
                            target: EntityReference::Existing(world.authored),
                        },
                    )),
                ),
                execution.allocation_policy(),
            )
            .unwrap();
        let written = transaction.commit(execution, &world.runtime).unwrap();
        world
            .runtime
            .snapshots()
            .release_snapshot(&written.snapshot)
            .unwrap();
        let selected = basis(&world.runtime);
        let before = capture(execution, &world.runtime);
        let payload = relation_payload(execution, &world.runtime, edge);
        let denied = world
            .runtime
            .readmit_required_output_publication(
                execution,
                &installed,
                &selected,
                "native.closed.producer",
                &[19; 32],
                &publication_fixture::meaning("native.closed.producer"),
                &publication_fixture::roles(
                    &world.outputs.map(|id| (id, KindId(3))),
                    RequiredOutputRolePosture::Create,
                ),
            )
            .unwrap_err();
        assert!(
            format!("{denied:?}").contains("incident relation native position/endpoints"),
            "{denied:?}"
        );
        let admission = owner
            .admit_required_output_succession(
                owner.admit_required_output("native.closed.preserver", [20; 32]),
                previous.authority(),
            )
            .unwrap();
        let meaning = previous.meaning().clone();
        let mut transaction = world
            .runtime
            .begin_branch_transaction(&selected, RelationalTransactionIntent::ordinary())
            .unwrap();
        transaction
            .push_batch(
                WorkerIntentBatch::new("attempt genuine no-data Preserve"),
                execution.allocation_policy(),
            )
            .unwrap();
        let proposal = transaction
            .validate(execution, &world.runtime)
            .unwrap()
            .bind_required_output_publication(
                &installed,
                worth_relational::facade::mvcc::RequiredOutputPublication::Preserve {
                    admission,
                    predecessor: previous,
                    meaning,
                    roles: publication_fixture::roles(
                        &world.outputs.map(|id| (id, KindId(3))),
                        RequiredOutputRolePosture::Preserve,
                    ),
                },
            )
            .unwrap();
        let denied = world
            .runtime
            .prepare_validated_proposal(execution, proposal)
            .unwrap_err();
        assert!(
            format!("{denied:?}").contains("incident relation breaks producer write/endpoints"),
            "{denied:?}"
        );
        assert!(world
            .runtime
            .owner_component_services()
            .basis_port()
            .compare_current_exact(&selected)
            .is_ok());
        assert_eq!(capture(execution, &world.runtime), before);
        assert_eq!(relation_payload(execution, &world.runtime, edge), payload);
    }
}
