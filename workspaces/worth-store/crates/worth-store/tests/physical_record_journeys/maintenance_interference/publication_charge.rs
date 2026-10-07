use std::num::{NonZeroU32, NonZeroU64};

use worth_proof::{NonEmpty, TransitionOutcome};
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, ManifestEntryCapacity, PhysicalDataDispatchOutcome,
    PhysicalManifestCapacityTransition, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationPreparationSuccess, PhysicalRecordAccessPolicy,
    PhysicalRecordFormatDeclaration, PhysicalRecordInitialization, PhysicalRecordPlacementPolicy,
    PhysicalWalGroupAppendOutcome, PhysicalWalGroupBarrierOutcome, PhysicalWalPolicy,
    RecordAppendBatch, WalSegmentByteLimit, WalSegmentInventoryLimit,
};
use worth_store_physical_format::{
    BOOTSTRAP_CATALOG_BYTES, DURABLE_FRAME_HEADER_BYTES, ROOT_SELECTOR_BYTES,
};

use super::reopen::open_interference;
use super::{append, residency, RESIDENT_BYTES};

#[path = "publication_charge/cutover.rs"]
mod cutover;
use crate::publication_metadata_oracle as oracle;

static EXTENT_SEED: [u8; 20_000] = [0x51; 20_000];

#[test]
fn branched_group_settles_one_actual_metadata_charge_and_reopens_exactly() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let placement = PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(2).unwrap())
        .admit(format)
        .unwrap();
    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    let media = super::super::media(&root);
    let durability = super::super::durability_with_wal_policy(
        &media,
        PhysicalWalPolicy::segmented(
            WalSegmentByteLimit::new(NonZeroU64::new(512 * 1024).unwrap()),
            WalSegmentInventoryLimit::new(NonZeroU32::new(64).unwrap()),
        ),
    );
    let serving = super::super::success(
        media.initialize_record_store(
            PhysicalRecordInitialization::new(format, placement, access, durability)
                .with_residency_policy(residency(format, RESIDENT_BYTES)),
        ),
    );
    let seeds = (0..8).map(|_| EXTENT_SEED.as_slice()).collect::<Vec<_>>();
    let seeded = super::super::durable_publication::publish_single(
        &serving,
        placement,
        PhysicalMutationIdempotencyMaterial::new([201; 32]),
        RecordAppendBatch::try_from_iter(seeds).unwrap(),
    );
    assert_eq!(seeded.current_root().record_count(), 8);
    assert!(seeded.current_root().routing_root().unwrap().level() >= 2);
    let before_charge = serving.certification_charged_growth_bytes();
    let before_wal = oracle::wal_file_bytes(&root);
    let submission = serving.certification_record_submission();
    let left = prepared(&submission, placement, [202; 32], b"left");
    let right = prepared(&submission, placement, [203; 32], b"right");
    let appended = match submission.append_prepared_wal_group(NonEmpty::new(left, vec![right])) {
        PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
        _ => panic!("two-member group did not append"),
    };
    assert_eq!(appended.members().len(), 2);
    let targets = appended
        .members()
        .iter()
        .map(|member| {
            member
                .mutation()
                .reserved()
                .redo()
                .records()
                .iter()
                .flat_map(|record| record.targets())
                .map(|claim| claim.target())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    for left in &targets[0] {
        for right in &targets[1] {
            assert_ne!(
                left, right,
                "group members must address disjoint data frames"
            );
        }
    }
    let basis = appended.basis();
    let mut durable = match submission.synchronize_appended_wal_group(appended) {
        PhysicalWalGroupBarrierOutcome::Durable(group) => group.into_members().into_vec(),
        _ => panic!("two-member group was not durable"),
    };
    let right = durable.pop().unwrap();
    let left = durable.pop().unwrap();
    let dispatched = [left, right]
        .map(|member| require_dispatched(submission.dispatch_wal_durable_data(member)));
    let [left, right] = dispatched;
    let completed =
        serving.certification_complete_dispatched_group(basis, NonEmpty::new(left, vec![right]));
    let generation = completed.current_root().generation();
    assert_eq!(completed.settled_members().len(), 2);
    assert_eq!(completed.current_root().record_count(), 10);
    let actual_metadata =
        oracle::actual_publication_metadata(&root, generation, format.declaration());
    let wal_growth = oracle::wal_file_bytes(&root) - before_wal;
    let charged = serving.certification_charged_growth_bytes();
    assert_eq!(charged - before_charge, wal_growth + actual_metadata);
    assert!(actual_metadata < 2 * full_member_metadata_ceiling(9));
    serving.close();

    let reopened = open_interference(&root);
    assert_eq!(reopened.certification_charged_growth_bytes(), charged);
    let second_ceiling = full_member_metadata_ceiling(11);
    // One next append needs its routing ceiling, a WAL member (bounded by
    // the two-member WAL measured above), and candidate/displaced page space.
    // Exact charge/parity above, not the size of this headroom, detects leaks.
    let page_bytes = u64::from(format.declaration().page_size().bytes());
    let finite_limit = charged + second_ceiling + wal_growth + 2 * page_bytes;
    reopened.certification_limit_candidate_growth_bytes(finite_limit);
    append(&reopened, placement, 204, b"next-after-settled-group");
    assert_eq!(
        reopened
            .observer()
            .acquisition_snapshot()
            .unwrap()
            .root_generation(),
        generation + 1
    );
    reopened.close();
}

fn require_dispatched(
    outcome: PhysicalDataDispatchOutcome,
) -> worth_store::physical_runtime::DataDispatchedPhysicalMutation {
    match outcome {
        PhysicalDataDispatchOutcome::Dispatched(dispatched) => dispatched,
        PhysicalDataDispatchOutcome::Suspended(retry) => {
            panic!("group data suspended: {:?}", retry.cause())
        }
        PhysicalDataDispatchOutcome::NotStarted { cause, .. } => {
            panic!("group data not started: {cause:?}")
        }
        PhysicalDataDispatchOutcome::Indeterminate(failure) => {
            panic!("group data indeterminate: {:?}", failure.cause())
        }
    }
}

fn prepared(
    submission: &worth_store::physical_runtime::PhysicalRecordSubmission,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    identity: [u8; 32],
    bytes: &[u8],
) -> worth_store::physical_runtime::PreparedPhysicalMutation {
    let outcome = super::super::durable_publication::prepare_single(
        submission,
        placement,
        PhysicalManifestCapacityTransition::PreserveCurrent,
        PhysicalMutationIdempotencyMaterial::new(identity),
        RecordAppendBatch::try_from_iter([bytes]).unwrap(),
    );
    match outcome.into_raw() {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(value)) => value,
        _ => panic!("group member not prepared"),
    }
}

/// This is only the old conservative reservation oracle for choosing a finite
/// test budget. Actual metadata comes from literal emitted file lengths above.
fn full_member_metadata_ceiling(entries: u64) -> u64 {
    let capacity = 2_u64;
    let leaves = entries.div_ceil(capacity);
    let mut branches = 0;
    let mut nodes = leaves;
    while nodes > 1 {
        nodes = nodes.div_ceil(capacity);
        branches += nodes;
    }
    let header = DURABLE_FRAME_HEADER_BYTES as u64 + 40;
    let tree = |leaf_width, branch_width| {
        leaves * (header + capacity * leaf_width) + branches * (header + capacity * branch_width)
    };
    let fixed = (DURABLE_FRAME_HEADER_BYTES as u64 + 336)
        + (DURABLE_FRAME_HEADER_BYTES as u64 + 168)
        + (2 * ROOT_SELECTOR_BYTES + BOOTSTRAP_CATALOG_BYTES) as u64;
    fixed + tree(88, 72) + tree(40, 56) + tree(40, 72)
}
