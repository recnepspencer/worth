//! Pure DATA vectors and identity dimensions; these tests admit no correction.
use super::*;
use worth_foundational::facade::{HomeIdentity, WorldOwnerSlotIdentity};
fn home(namespace: u8, slot: u64) -> DurableWorldOwnerIdentity {
    DurableWorldOwnerIdentity::from_recorded(
        HomeIdentity::from_observed_bytes([namespace; 16]).unwrap(),
        WorldOwnerSlotIdentity::from_recorded(slot),
    )
}
fn fields(owner: WorldOwnerCoordinate) -> CorrectionIntentFields<'static> {
    CorrectionIntentFields {
        bindings: ["correction", "source", "action"],
        role: WorthQueryAftermathCausalRole::RecordedReverse,
        source_owner: owner,
        source_commit: 12,
        source_member: WorthQueryCorrectionSourceMember::Primary,
        target_owner: owner,
        target_branch: "main",
        target_lifecycle: 2,
        original_intent: &[9; 32],
        lineage: [
            Some(Coordinate {
                owner,
                world_commit: 12,
                native_commit: CommitId(15),
                native_version: VersionId(16),
                native_branch: "main",
                member: WorthQueryCorrectionSourceMember::Primary,
            }),
            None,
            None,
            None,
        ],
    }
}
#[test]
fn home_correction_identity_excludes_new_live_receiver_observations() {
    let owner = home(1, 7);
    assert_eq!(
        encode(fields(scope(Some(owner), 11))),
        encode(fields(scope(Some(owner), 9001)))
    );
    assert_ne!(
        encode(fields(scope(None, 11))),
        encode(fields(scope(None, 9001)))
    );
    assert_ne!(
        encode(fields(scope(Some(owner), 11))),
        encode(fields(scope(None, 7)))
    );
}
#[test]
fn correction_identity_binds_home_namespace_slot_and_target_branch_occurrence() {
    let base = fields(scope(Some(home(1, 7)), 11));
    for different in [
        fields(scope(Some(home(2, 7)), 11)),
        fields(scope(Some(home(1, 8)), 11)),
    ] {
        assert_ne!(encode(base), encode(different));
    }
    let mut changed = base;
    changed.source_commit += 1;
    assert_ne!(encode(base), encode(changed));
    let mut changed = base;
    changed.target_branch = "sibling";
    assert_ne!(encode(base), encode(changed));
    let mut changed = base;
    changed.target_lifecycle += 1;
    assert_ne!(encode(base), encode(changed));
    let mut changed = base;
    changed.source_owner = scope(Some(home(2, 7)), 11);
    assert_ne!(encode(base), encode(changed));
    let mut changed = base;
    changed.target_owner = scope(Some(home(2, 7)), 11);
    assert_ne!(encode(base), encode(changed));
}
#[test]
fn correction_identity_binds_each_semantic_binding_role_and_original_input_identity() {
    let base = fields(scope(Some(home(1, 7)), 11));
    for index in 0..3 {
        let mut changed = base;
        changed.bindings[index] = "different";
        assert_ne!(encode(base), encode(changed));
    }
    let mut changed = base;
    changed.role = WorthQueryAftermathCausalRole::RecordedReapply;
    assert_ne!(encode(base), encode(changed));
    let mut changed = base;
    changed.original_intent = &[10; 32];
    assert_ne!(encode(base), encode(changed));
}
#[test]
fn correction_identity_binds_the_source_member_of_a_shared_commit_and_input() {
    // Public #368: two stages of one commit may share binding and input.
    let base = fields(scope(Some(home(1, 7)), 11));
    let stage = |ordinal| WorthQueryCorrectionSourceMember::AuthoredStage {
        ordinal,
        identity: [ordinal as u8; 32],
    };
    let mut first = base;
    first.source_member = stage(1);
    let mut second = base;
    second.source_member = stage(2);
    assert_ne!(encode(base), encode(first));
    assert_ne!(encode(first), encode(second));
}
#[test]
fn correction_identity_binds_every_lineage_membership_and_native_receipt_dimension() {
    let mut base = fields(scope(Some(home(1, 7)), 11));
    base.lineage = [base.lineage[0]; 4];
    for index in 0..4 {
        let mut changed = base;
        changed.lineage[index] = None;
        assert_ne!(encode(base), encode(changed));
        for dimension in 0..6 {
            let mut changed = base;
            let coordinate = changed.lineage[index].as_mut().unwrap();
            match dimension {
                0 => coordinate.owner = scope(Some(home(2, 7)), 11),
                1 => coordinate.world_commit += 1,
                2 => coordinate.native_commit = CommitId(16),
                3 => coordinate.native_version = VersionId(17),
                4 => coordinate.native_branch = "sibling",
                5 => {
                    coordinate.member = WorthQueryCorrectionSourceMember::AuthoredStage {
                        ordinal: 1,
                        identity: [1; 32],
                    }
                }
                _ => unreachable!("six declared value dimensions"),
            }
            assert_ne!(encode(base), encode(changed));
        }
    }
}
#[test]
fn home_encoding_keeps_owner_scope_in_every_lineage_coordinate() {
    let mut base = fields(scope(Some(home(1, 7)), 11));
    base.lineage = [base.lineage[0]; 4];
    for index in 0..4 {
        let mut changed = base;
        changed.lineage[index].as_mut().unwrap().owner = scope(None, 7);
        assert_ne!(encode(base), encode(changed));
    }
}
#[test]
fn memory_correction_identity_preserves_its_active_process_local_vector() {
    // Literal SHA-256 from the v3 protocol's explicit length-prefixed fields,
    // not calculated through the encoder under test.
    assert_eq!(
        encode(fields(scope(None, 11))),
        [
            135, 91, 139, 169, 216, 218, 59, 111, 21, 109, 99, 8, 153, 202, 190, 208, 156, 1, 207,
            31, 214, 227, 15, 98, 139, 109, 242, 215, 236, 18, 184, 192
        ]
    );
}
