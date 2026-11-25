//! An ordinary publication port cannot bypass recovered codec target admission.
use super::{fixture::*, recovered, serial_request, RelationalOperationControl};

#[test]
fn native_codec_candidate_requires_exact_recovered_owner_before_any_movement() {
    let request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let world = world(execution, true);
    let (mut target, mut recovered, receipt) = recovered(execution, &world);
    let selected = basis(&target);
    let source = protected_payload(execution, &target, world.authored);
    let count = target.history().immutable_commit_count();
    let (owner, installed) = target
        .durability_recovery()
        .issue_recovered_codec_source_publication_owner(execution, &recovered)
        .unwrap();
    for ordinary in [true, false] {
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
        // Exercise the real validated-proposal carrier as well as detached
        // native candidate preparation; custody must survive both phases.
        let proposal = target
            .validate_branch_transaction(execution, transaction)
            .unwrap();
        let candidate = target
            .prepare_validated_proposal(execution, proposal)
            .unwrap();
        if ordinary {
            let denied = target
                .publication_port()
                .compare_and_publish(execution, candidate);
            assert!(
                matches!(
                    denied,
                    worth_relational::facade::mvcc::RelationalPublicationOutcome::Denied(
                        worth_relational::facade::mvcc::RelationalPublicationDenial::RecoveredCodecTransitionRequired
                    )
                ),
                "{denied:?}"
            );
            assert!(target
                .owner_component_services()
                .basis_port()
                .compare_current_exact(&selected)
                .is_ok());
            assert_eq!(target.history().immutable_commit_count(), count);
            assert_eq!(
                protected_payload(execution, &target, world.authored),
                source
            );
            assert!(target
                .readmit_recovered_initial_schema_installation(receipt.clone())
                .is_err());
        } else {
            let acknowledged = target
                .durability_recovery()
                .commit_checkpoint_transition(execution, recovered, candidate)
                .unwrap();
            let (_, successor) = acknowledged.into_parts();
            assert!(successor.codec_transition_retires(world.output));
            target
                .readmit_recovered_initial_schema_installation(receipt)
                .unwrap();
            assert_eq!(
                protected_payload(execution, &target, world.authored),
                source
            );
            return;
        }
    }
    unreachable!("the exact native recovered owner must complete the second attempt")
}
