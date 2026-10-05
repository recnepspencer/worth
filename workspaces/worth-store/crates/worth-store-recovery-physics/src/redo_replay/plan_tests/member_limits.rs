//! A member is decoded under the targets its predecessors left. The limit
//! it runs past is stated over all the members, as recovery admitted it.

use super::*;
use worth_store_wal::{LogSequenceNumber, WalLsnRange};

fn member(start: u64) -> PhysicalRedoMemberInput {
    PhysicalRedoMemberInput::new(
        WalLsnRange::new(
            LogSequenceNumber::new(start),
            LogSequenceNumber::new(start + 1),
        )
        .unwrap(),
        [1; 32],
        RecoveryOperationFate::Indeterminate,
        &encoded_redo(),
    )
}

#[test]
fn a_member_past_the_targets_left_to_it_names_what_all_members_needed() {
    // Each member carries one target; one is admitted, so the second member
    // is the one refused, with none left to it.
    let members = || vec![member(10), member(11)];
    let past = || {
        Err(PhysicalRedoPlanningDenial::TargetLimit {
            observed: 2,
            admitted: 1,
        })
    };
    assert_eq!(
        plan_physical_redo(members(), Vec::new(), 1).map(|_| ()),
        past()
    );
    assert_eq!(
        physical_redo_target_identities(&members(), 1, 2, test_format()).map(|_| ()),
        past()
    );
    assert_eq!(
        physical_redo_observation_target_identities(&members(), 1, test_format()).map(|_| ()),
        past()
    );
    assert_eq!(
        physical_redo_observation_targets(&members(), 1, test_format()).map(|_| ()),
        past()
    );
}
