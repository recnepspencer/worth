use super::*;
use contracts::identity::data::primitives::PartitionId;
use contracts::identity::data::record_ids::EntityId;
use contracts::transactions::data::primitives::RecordRef;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::tests::envelope_with_patch_and_update_for_test as envelope_with_patch_and_update;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeFixturePort as _;
use worth_relational_contracts::internal_port as contracts;

#[test]
fn corrupt_replay_keeps_its_durability_failure() {
    let entity = EntityId::new(PartitionId::main(), 1, 1);
    let mut envelope = envelope_with_patch_and_update(RecordRef::Entity(entity), entity);
    // A merge without its required execution authority is corrupt evidence.
    let branch_context = envelope.branch_context.clone();
    envelope
        .body_mut_for_test()
        .merge_parent_branches
        .push(branch_context);
    let error = require_merge_source(&envelope).unwrap_err();
    match error {
        RelationalRecoveryStop::AdmissionImage(cause) => {
            panic!("corrupt original evidence became image refusal: {cause:?}")
        }
        RelationalRecoveryStop::Durability(error) => {
            assert_eq!(error.class, RecoveryFailureClass::ReplayFailure)
        }
        RelationalRecoveryStop::Request(cause) => {
            panic!("corrupt evidence became request control: {cause:?}")
        }
        RelationalRecoveryStop::Content(cause) => panic!("unexpected content refusal: {cause:?}"),
        RelationalRecoveryStop::Allocation(cause) => {
            panic!("corrupt evidence became allocation failure: {cause:?}")
        }
    }
}
