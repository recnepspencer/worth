use super::*;

// These inline hash/ordering observations are local semantic inputs, not minted
// durability authority or evidence of native allocation admission.
fn members() -> [ObservedGroupMember; 2] {
    let first = ObservedGroupMember {
        store: [1; 16],
        runtime: 2,
        operation: 3,
        member: [4; 32],
        idempotency: [5; 32],
        ordinal: 1,
        count: 2,
        membership: [0; 32],
    };
    let mut members = [
        first,
        ObservedGroupMember {
            operation: 6,
            member: [7; 32],
            idempotency: [8; 32],
            ordinal: 2,
            ..first
        },
    ];
    seal_digest(&mut members);
    members
}

fn seal_digest(members: &mut [ObservedGroupMember]) {
    let digest = crate::physical_runtime::durability::reopened_membership_digest_fields(
        members.len(),
        members.iter().map(|member| {
            (
                member.store,
                member.runtime,
                member.operation,
                member.member,
                member.idempotency,
            )
        }),
    )
    .unwrap();
    for member in members {
        member.membership = digest;
    }
}

#[test]
fn group_validation_reuses_exact_workspace_without_reordering_member_facts() {
    let members = members();
    let mut scratch = Vec::with_capacity(2);
    let pointer = scratch.as_ptr();
    let capacity = scratch.capacity();
    validate_group(members.len(), |index| members[index], &mut scratch).unwrap();
    assert!(scratch.is_empty());
    assert_eq!(scratch.capacity(), capacity);
    assert_eq!(scratch.as_ptr(), pointer);
    validate_group(members.len(), |index| members[index], &mut scratch).unwrap();
    assert_eq!(scratch.as_ptr(), pointer);
    assert_eq!(members[0].ordinal, 1);
    assert_eq!(members[1].ordinal, 2);
}

#[test]
fn duplicate_member_and_idempotency_are_rejected_even_with_matching_digest() {
    for duplicate_member in [false, true] {
        let mut members = members();
        if duplicate_member {
            members[1].member = members[0].member;
        } else {
            members[1].idempotency = members[0].idempotency;
        }
        seal_digest(&mut members);
        let mut scratch = Vec::with_capacity(2);
        assert_eq!(
            validate_group(2, |index| members[index], &mut scratch),
            Err(Denial::InvalidWalMember)
        );
        assert_eq!(scratch.capacity(), 2);
    }
}

#[test]
fn ordinal_count_and_digest_disagreement_are_distinct_rejecting_inputs() {
    for changed in 0..4 {
        let mut members = members();
        match changed {
            0 => members[1].ordinal = 1,
            1 => members[1].count = 3,
            2 => members[1].membership = [9; 32],
            3 => {
                members[0].membership = [9; 32];
                members[1].membership = [9; 32];
            }
            _ => unreachable!(),
        }
        let mut scratch = Vec::with_capacity(2);
        assert_eq!(
            validate_group(2, |index| members[index], &mut scratch),
            Err(Denial::InvalidWalMember)
        );
    }
    let members = members();
    assert_eq!(
        validate_group(1, |index| members[index], &mut Vec::with_capacity(1)),
        Err(Denial::InvalidWalMember)
    );
}

#[test]
fn zero_groups_need_no_scratch_and_short_workspace_never_grows() {
    validate_wal_groups(&mut [], &mut Vec::new()).unwrap();
    let members = members();
    let mut scratch = Vec::with_capacity(1);
    let pointer = scratch.as_ptr();
    assert_eq!(
        validate_group(2, |index| members[index], &mut scratch),
        Err(Denial::RecoveryMemoryLimit)
    );
    assert_eq!(scratch.capacity(), 1);
    assert_eq!(scratch.as_ptr(), pointer);
    assert!(scratch.is_empty());
}
