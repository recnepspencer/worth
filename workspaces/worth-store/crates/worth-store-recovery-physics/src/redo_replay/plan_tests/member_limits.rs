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

#[test]
fn pre_decode_redo_memory_budget_refuses_before_allocating_decoded_member() {
    let base_limits = PhysicalRedoAdmissionLimits {
        recovery_memory_bytes: 0,
        targets: 8,
        distinct_targets: 8,
        projection: PhysicalRecoveryProjectionDecodeLimits {
            frames: 8,
            record_identities: 8,
            placements: 8,
            segment_updates: 8,
            manifests: 8,
            total_entries: 24,
            inline_allocations: 8,
        },
    };
    let denied = admit_physical_redo_members_with_memory_budget(
        vec![member(10)],
        test_store(),
        test_format(),
        base_limits,
        0,
    )
    .expect_err("zero memory budget must refuse before decoding member");
    let PhysicalRedoPlanningDenial::RecoveryMemoryLimit { observed, admitted } = denied else {
        panic!("expected RecoveryMemoryLimit, got {denied:?}");
    };
    assert_eq!(admitted, 0);
    assert!(observed > 0);
    assert_eq!(
        admit_physical_redo_members_with_memory_budget(
            vec![member(10)],
            test_store(),
            test_format(),
            PhysicalRedoAdmissionLimits {
                recovery_memory_bytes: observed - 1,
                ..base_limits
            },
            observed - 1,
        ),
        Err(PhysicalRedoPlanningDenial::RecoveryMemoryLimit {
            observed,
            admitted: observed - 1,
        })
    );
    let ok = admit_physical_redo_members_with_memory_budget(
        vec![member(10)],
        test_store(),
        test_format(),
        PhysicalRedoAdmissionLimits {
            recovery_memory_bytes: 64 * 1024,
            ..base_limits
        },
        64 * 1024,
    )
    .expect("healthy budget admits member");
    assert_eq!(ok.target_identity_count(), 1);
    assert_eq!(ok.distinct_target_count(), 1);
    assert!(ok.owned_heap_bytes().expect("owned bytes") > 0);
    assert_eq!(ok.target_identities().len(), 1);
}
