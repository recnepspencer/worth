//! Activation permission cannot be substituted across occurrences or updates.
use super::{fixture::*, recovered};

#[test]
fn activation_proof_rejects_foreign_owner_and_changed_exact_description_before_effects() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let world = world(execution, true);
    for foreign_owner in [false, true] {
        let (mut target, mut recovered, _) = recovered(execution, &world);
        let selected = basis(&target);
        let (owner, installed) = target
            .durability_recovery()
            .issue_recovered_codec_source_publication_owner(execution, &recovered)
            .unwrap();
        let source_before = protected_payload(&target, world.authored);
        let mut update = activation(world.activation);
        let digest = target
            .durability_recovery()
            .checkpoint_program_activation_commitment(execution, &recovered, &selected, &update)
            .unwrap();
        let foreign = worth_proof::InstalledApplicationPublicationOwner::fresh();
        let proof = if foreign_owner {
            foreign.admit_checkpoint_program_activation(digest)
        } else {
            owner.admit_checkpoint_program_activation(digest)
        };
        if !foreign_owner {
            update.replacement = worth_foundational::facade::AspectValue::String(
                worth_foundational::facade::InternedString::from("program.v3"),
            );
        }
        let denied = target
            .durability_recovery()
            .begin_recovered_codec_transition(
                execution,
                &mut recovered,
                &selected,
                crate::mvcc::RelationalOperationControl::uninterrupted(),
                &installed,
                proof,
                update,
            )
            .unwrap_err();
        assert_eq!(denied.kind(),crate::runtime::RelationalSchemaTransitionAdmissionDenialKind::RecoveredAuthorityMismatch);
        assert!(selected.is_current());
        assert_eq!(source_before, protected_payload(&target, world.authored));
        assert!(!recovered.codec_transition_retires(world.output));
    }
}
