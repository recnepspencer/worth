#[allow(unused_imports)] use crate::physical_record_journeys::maintenance_interference::publication_charge::*;
use {std::num::NonZeroU32, std::num::NonZeroU64};

use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::ManifestEntryCapacity, worth_store::physical_runtime::PhysicalDataDispatchOutcome, worth_store::physical_runtime::PhysicalManifestCapacityTransition, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalRecordAccessPolicy, worth_store::physical_runtime::PhysicalRecordFormatDeclaration, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordPlacementPolicy, worth_store::physical_runtime::PhysicalWalGroupAppendOutcome, worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome, worth_store::physical_runtime::PhysicalWalPolicy, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::WalSegmentByteLimit, worth_store::physical_runtime::WalSegmentInventoryLimit};
use {worth_store_physical_format::BOOTSTRAP_CATALOG_BYTES, worth_store_physical_format::DURABLE_FRAME_HEADER_BYTES, worth_store_physical_format::ROOT_SELECTOR_BYTES};

use {worth_store_testkit_store::physical_record_journeys::maintenance_interference::reopen::open_interference};
use {worth_store_testkit_store::physical_record_journeys::maintenance_interference::append, worth_store_testkit_store::physical_record_journeys::maintenance_interference::residency, worth_store_testkit_store::physical_record_journeys::maintenance_interference::RESIDENT_BYTES};


use {worth_store_testkit_store::publication_metadata_oracle as oracle};

#[test]
fn branched_group_settles_one_actual_metadata_charge_and_reopens_exactly() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let placement = worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(2).unwrap())
        .admit(format)
        .unwrap();
    let access = worth_store::physical_runtime::PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    let media = worth_store_testkit_store::physical_record_journeys::media(&root);
    let durability = worth_store_testkit_store::physical_record_journeys::durability_with_wal_policy(
        &media,
        worth_store::physical_runtime::PhysicalWalPolicy::segmented(
            worth_store::physical_runtime::WalSegmentByteLimit::new(std::num::NonZeroU64::new(512 * 1024).unwrap()),
            worth_store::physical_runtime::WalSegmentInventoryLimit::new(std::num::NonZeroU32::new(64).unwrap()),
        ),
    );
    let serving = worth_store_testkit_store::physical_record_journeys::success(
        media.initialize_record_store(
            worth_store::physical_runtime::PhysicalRecordInitialization::new(format, placement, access, durability)
                .with_residency_policy(residency(format, RESIDENT_BYTES)),
        ),
    );
    let seeds = (0..8).map(|_| EXTENT_SEED.as_slice()).collect::<Vec<_>>();
    let seeded = worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([201; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter(seeds).unwrap(),
    );
    assert_eq!(seeded.current_root().record_count(), 8);
    assert!(seeded.current_root().routing_root().unwrap().level() >= 2);
    let before_charge = serving.certification_charged_growth_bytes();
    let before_wal = worth_store_testkit_store::publication_metadata_oracle::wal_file_bytes(&root);
    let submission = serving.certification_record_submission();
    let left = prepared(&submission, placement, [202; 32], b"left");
    let right = prepared(&submission, placement, [203; 32], b"right");
    let appended = match submission.append_prepared_wal_group(worth_proof::NonEmpty::new(left, vec![right])) {
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
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
        worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome::Durable(group) => group.into_members().into_vec(),
        _ => panic!("two-member group was not durable"),
    };
    let right = durable.pop().unwrap();
    let left = durable.pop().unwrap();
    let dispatched = [left, right]
        .map(|member| require_dispatched(submission.dispatch_wal_durable_data(member)));
    let [left, right] = dispatched;
    let completed =
        serving.certification_complete_dispatched_group(basis, worth_proof::NonEmpty::new(left, vec![right]));
    let generation = completed.current_root().generation();
    assert_eq!(completed.settled_members().len(), 2);
    assert_eq!(completed.current_root().record_count(), 10);
    let actual_metadata =
        worth_store_testkit_store::publication_metadata_oracle::actual_publication_metadata(&root, generation, format.declaration());
    let wal_growth = worth_store_testkit_store::publication_metadata_oracle::wal_file_bytes(&root) - before_wal;
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
#[path = "cutover/mod.rs"]
pub(crate) mod cutover;
