//! C.9 admits a record-less terminal head retirement member, charges it, and
//! refuses to plan it.

use worth_store_physical_format::{
    PersistedPhysicalRecoveryOperation, PhysicalRecoveryProjectionDecodeLimits,
};

use super::*;
use crate::redo_replay::terminal_head_retirement_fixture as retirement;
use crate::RecoveryOperationFate::{AcknowledgedDurable, Indeterminate};

fn limits(recovery_memory_bytes: u64, total_entries: u64) -> PhysicalRedoAdmissionLimits {
    PhysicalRedoAdmissionLimits {
        recovery_memory_bytes,
        targets: 0,
        distinct_targets: 0,
        projection: PhysicalRecoveryProjectionDecodeLimits {
            frames: 0,
            record_identities: 0,
            placements: 0,
            segment_updates: 0,
            manifests: 0,
            total_entries,
            inline_allocations: 0,
        },
    }
}

fn claimed() -> PersistedPhysicalRecoveryProjection {
    let heads = vec![retirement::terminal_head(), retirement::survivor()];
    retirement::projection(retirement::selected_terminal_head(heads).retirement)
}

fn admit(
    members: Vec<PhysicalRedoMemberInput>,
    store: StableStoreIdentity,
    limits: PhysicalRedoAdmissionLimits,
) -> Result<AdmittedPhysicalRedoMembers, PhysicalRedoPlanningDenial> {
    admit_physical_redo_members(members, store, retirement::format(), limits)
}

#[test]
fn c9_admits_a_record_less_terminal_head_retirement_member_with_no_page_target() {
    let claimed = claimed();
    let member = retirement::member(&claimed, 12, 1, Indeterminate);
    assert_eq!(
        physical_redo_observation_target_identities(
            std::slice::from_ref(&member),
            1,
            retirement::format()
        )
        .as_deref(),
        Ok([].as_slice())
    );
    let admitted = admit(vec![member], retirement::store(), limits(u64::MAX, 2)).unwrap();
    let mut members = admitted.admitted_root_step_members();
    let view = members.next().unwrap();
    assert!(members.next().is_none());
    assert_eq!(view.materialization(), &claimed);
    assert!(matches!(
        view.materialization().operation(),
        PersistedPhysicalRecoveryOperation::TerminalReleaseHeadRetired(_)
    ));
    assert_eq!(view.record_bytes(0), None);
    assert!(admitted.target_identities().is_empty());
    assert!(admitted.observation_targets().is_empty());
    assert_eq!(admitted.admitted_drop_members().count(), 0);
}

#[test]
fn c9_denies_a_terminal_head_retirement_of_another_store() {
    let member = retirement::member(&claimed(), 12, 1, Indeterminate);
    assert_eq!(
        admit(
            vec![member],
            retirement::store_of([8; 16]),
            limits(u64::MAX, 2)
        )
        .err(),
        Some(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)
    );
}

#[test]
fn c9_charges_every_terminal_head_retirement_against_the_shared_allowances() {
    let claimed = claimed();
    let members = || {
        vec![
            retirement::member(&claimed, 12, 1, Indeterminate),
            retirement::member(&claimed, 13, 2, Indeterminate),
        ]
    };
    // Each member carries one path frame and one rewritten leaf.
    let admitted = admit(members(), retirement::store(), limits(u64::MAX, 4)).unwrap();
    let charged = admitted.scratch_bytes;
    assert_eq!(
        admit(members(), retirement::store(), limits(u64::MAX, 3)).err(),
        Some(PhysicalRedoPlanningDenial::InvalidRecoveryProjection),
        "the second member's tree frames exceed what the first left"
    );
    assert_eq!(
        admit(members(), retirement::store(), limits(charged - 1, 4)).err(),
        Some(PhysicalRedoPlanningDenial::RecoveryMemoryLimit {
            observed: charged,
            admitted: charged - 1,
        })
    );
}

/// Room for one ordinary one-target member and one retirement's tree frames.
fn mixed_limits() -> PhysicalRedoAdmissionLimits {
    PhysicalRedoAdmissionLimits {
        recovery_memory_bytes: u64::MAX,
        targets: 1,
        distinct_targets: 1,
        projection: PhysicalRecoveryProjectionDecodeLimits {
            frames: 1,
            record_identities: 1,
            placements: 1,
            segment_updates: 1,
            manifests: 1,
            total_entries: 5,
            inline_allocations: 1,
        },
    }
}

#[test]
fn no_plan_contains_a_terminal_head_retirement_behind_an_ordinary_member() {
    let ordinary =
        || PhysicalRedoMemberInput::new(range(), [1; 32], Indeterminate, &encoded_redo());
    let observations = || vec![observation(1, 9, [0; 32])];
    let alone = admit(vec![ordinary()], retirement::store(), mixed_limits()).unwrap();
    assert!(
        alone.plan(observations()).is_ok(),
        "the ordinary member plans on its own"
    );
    let members = vec![
        ordinary(),
        retirement::member(&claimed(), 11, 2, Indeterminate),
    ];
    let admitted = admit(members, retirement::store(), mixed_limits()).unwrap();
    assert_eq!(
        admitted.plan(observations()).err(),
        Some(PhysicalRedoPlanningDenial::TerminalHeadRetirementUnsupported)
    );
}

#[test]
fn no_plan_contains_a_terminal_head_retirement() {
    let claimed = claimed();
    for fate in [Indeterminate, AcknowledgedDurable] {
        let member = retirement::member(&claimed, 12, 1, fate);
        let admitted = admit(vec![member], retirement::store(), limits(u64::MAX, 2)).unwrap();
        assert_eq!(
            admitted.plan(Vec::new()).err(),
            Some(PhysicalRedoPlanningDenial::TerminalHeadRetirementUnsupported)
        );
    }
}
