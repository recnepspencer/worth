use worth_store_physical_format::{
    PhysicalRecordFormatDeclaration, PhysicalRecoveryProjectionDecodeLimits, PhysicalRewriteRedo,
};
use worth_store_wal::{LogSequenceNumber, WalLsnRange};

use super::{
    encoded_redo, observation, plan_physical_redo, range, test_store, PhysicalRedoMemberInput,
    PhysicalRedoPlanningDenial, RecoveryOperationFate,
};
use crate::{admit_physical_redo_members, PhysicalRedoAdmissionLimits};

fn rewrite(length: u32) -> PhysicalRewriteRedo {
    PhysicalRewriteRedo::new(
        [4; 32], [5; 32], 3, 7, 64, length, [9; 32], 8, 128, 11, [3; 32], 64, 128, 4,
    )
    .unwrap()
}

fn member(start: u64, bytes: &[u8]) -> PhysicalRedoMemberInput {
    PhysicalRedoMemberInput::new(
        WalLsnRange::new(
            LogSequenceNumber::new(start),
            LogSequenceNumber::new(start + 1),
        )
        .unwrap(),
        [start as u8; 32],
        RecoveryOperationFate::Indeterminate,
        bytes,
    )
}

#[test]
fn rewrite_redo_is_admitted_beside_canonical_redo_and_unknown_domains_stay_rejected() {
    let payload = rewrite(32);
    let plan = plan_physical_redo(
        vec![
            PhysicalRedoMemberInput::new(
                range(),
                [1; 32],
                RecoveryOperationFate::Indeterminate,
                &encoded_redo(),
            ),
            member(11, &payload.encode()),
        ],
        vec![observation(1, 9, [0; 32])],
        64,
    )
    .unwrap();
    assert_eq!(plan.decisions().len(), 1);
    assert_eq!(plan.rewrites(), &[payload]);

    let unknown_domain = b"store.physical.unknown.v9";
    let mut unknown = (unknown_domain.len() as u64).to_le_bytes().to_vec();
    unknown.extend_from_slice(unknown_domain);
    assert_eq!(
        plan_physical_redo(vec![member(10, &unknown)], Vec::new(), 64),
        Err(PhysicalRedoPlanningDenial::WrongDomain)
    );
    let mut truncated = payload.encode();
    truncated.pop();
    assert_eq!(
        plan_physical_redo(vec![member(10, &truncated)], Vec::new(), 64),
        Err(PhysicalRedoPlanningDenial::MalformedMember)
    );

    let denied = admit_physical_redo_members(
        vec![member(10, &payload.encode())],
        test_store(),
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
        PhysicalRedoAdmissionLimits {
            recovery_memory_bytes: 16,
            targets: 4,
            distinct_targets: 4,
            projection: PhysicalRecoveryProjectionDecodeLimits {
                frames: 4,
                record_identities: 4,
                placements: 4,
                segment_updates: 4,
                manifests: 4,
                total_entries: 12,
                inline_allocations: 4,
            },
        },
    );
    assert_eq!(
        denied,
        Err(PhysicalRedoPlanningDenial::RecoveryMemoryLimit {
            observed: 32,
            admitted: 16,
        })
    );
}
