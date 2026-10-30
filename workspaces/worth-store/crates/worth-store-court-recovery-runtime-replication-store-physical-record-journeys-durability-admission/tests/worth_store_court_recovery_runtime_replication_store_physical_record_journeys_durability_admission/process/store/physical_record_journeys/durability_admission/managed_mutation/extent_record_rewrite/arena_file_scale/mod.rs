#[allow(unused_imports)] use crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::arena_file_scale::*;
use {worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordReadLimits, worth_store::physical_runtime::ServingPhysicalRuntime, worth_signal::facade::TemporalDuration, worth_proof::TransitionOutcome, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::completed, worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare_records, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::read_record, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::request};
use {std::collections::HashSet};
use {std::num::NonZeroU32, std::num::NonZeroU64};
use {std::path::Path};
use {std::process::Command};
use {worth_store::physical_runtime::CheckpointMemoryLimit, worth_store::physical_runtime::ExtentArenaCapacity, worth_store::physical_runtime::GroupCommitDelay, worth_store::physical_runtime::GroupCommitLimit, worth_store::physical_runtime::IdempotencyRetentionGenerations, worth_store::physical_runtime::LiveIdempotencyBindingLimit, worth_store::physical_runtime::PendingUnresolvedMutationLimit, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointPolicy, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalDurabilityDeclaration, worth_store::physical_runtime::PhysicalIdempotencyPolicy, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::PhysicalRecordPlacementPolicy, worth_store::physical_runtime::PhysicalWalPolicy, worth_store::physical_runtime::RecordCountLimit, worth_store::physical_runtime::RecordScanOutcome, worth_store::physical_runtime::RecordScanRequest, worth_store::physical_runtime::RetainedWalTailLimit, worth_store::physical_runtime::WalSegmentByteLimit, worth_store::physical_runtime::WalSegmentInventoryLimit};

/// Exact C11 file-count population: 32,768 records and over 4 GiB of logical
/// payload, requiring multiple GiB of disk and a long certification window.
#[test]
#[ignore = "32,768 real records and >4 GiB payload; run in the C11 capacity lane"]
fn thirty_two_thousand_extents_use_arena_files_proportional_to_bytes() {
    run_scale_case(16_384, 1 << 30);
}

#[test]
fn small_population_reopens_in_another_process_and_agrees_with_offline_media() {
    run_scale_case(33, 64 << 20);
}

#[test]
#[ignore = "the parent starts this writer in a distinct process"]
fn scale_writer_child() {
    let (root, count, capacity) = case_from_env();
    let (format, _, access) = configuration();
    let placement = worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .arena_capacity(
            worth_store::physical_runtime::ExtentArenaCapacity::bytes(capacity).unwrap(),
        )
        // Each batch reserves its arena ranges before root publication. The
        // default 64 KiB index has 15 slots shared by free ranges and claims,
        // with admission headroom reserved.
        .arena_index_bytes(worth_store::physical_runtime::RecordByteLimit::new(256 * 1024).unwrap())
        .admit(format)
        .unwrap();
    let media = worth_store_testkit_store::physical_record_journeys::media(&root);
    let durability = large_population_durability(&media);
    let serving = worth_store_testkit_store::physical_record_journeys::success(
        media.initialize_record_store(
            worth_store::physical_runtime::PhysicalRecordInitialization::new(
                format, placement, access, durability,
            ),
        ),
    );
    let mut published_records = 0_u64;
    let mut batch_number = 0_u64;
    let mut checkpoints = 0_u64;
    let mut identities = std::collections::HashSet::new();
    for (phase, bytes, batch_width) in [(0_u64, LARGE, LARGE_BATCH), (1, SMALL, SMALL_BATCH)] {
        let payload = vec![phase as u8 + 47; bytes];
        for start in (0..count).step_by(batch_width as usize) {
            let batch_size = (count - start).min(batch_width);
            batch_number += 1;
            let mut key = [43; 32];
            key[..8].copy_from_slice(&batch_number.to_le_bytes());
            let records = vec![payload.as_slice(); batch_size as usize];
            let completed =
                completed(prepare_records(&serving, placement, key, &records).execute());
            assert_eq!(completed.persisted_records().len(), batch_size as usize);
            for record in completed.persisted_records() {
                assert!(
                    identities.insert(*record),
                    "batch reused a physical record identity"
                );
            }
            let previous = published_records;
            published_records += batch_size;
            for checkpoint in (previous / 256 + 1)..=(published_records / 256) {
                checkpoint_and_bound_wal(&serving, checkpoint);
                checkpoints += 1;
            }
        }
    }
    assert_eq!(published_records, count * 2);
    assert_eq!(identities.len() as u64, published_records);
    assert_eq!(checkpoints, published_records / 256);
    serving.close();
}

#[test]
#[ignore = "the parent starts this reopener in a distinct process"]
fn scale_reopener_child() {
    let (root, count, _) = case_from_env();
    let (format, _, access) = configuration();
    let media = worth_store_testkit_store::physical_record_journeys::media(&root);
    let durability = large_population_durability(&media);
    let serving =
        worth_store_testkit_store::physical_record_journeys::success(media.open_record_store(
            worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability),
        ));
    let mut scan = serving
        .records()
        .unwrap()
        .scan(
            worth_store::physical_runtime::RecordScanRequest::from_start().with_batch_limit(
                worth_store::physical_runtime::RecordCountLimit::new(32).unwrap(),
            ),
        )
        .unwrap();
    let mut scratch = vec![0_u8; 64 * 1024];
    let mut large = 0_u64;
    let mut small = 0_u64;
    let mut identities = std::collections::HashSet::new();
    while let worth_store::physical_runtime::RecordScanOutcome::Batch(batch) =
        scan.read_next_into(&mut scratch).unwrap()
    {
        for record in batch.records() {
            assert!(
                identities.insert(record.record_id()),
                "fresh-process scan repeated a physical record identity"
            );
            let (seen, expected, value) = match record.declared_payload_bytes() {
                n if n == LARGE as u64 => (&mut large, LARGE, 47),
                n if n == SMALL as u64 => (&mut small, SMALL, 48),
                n => panic!("unexpected reopened record length {n}"),
            };
            *seen += 1;
            if *seen == 1 || *seen == count / 2 || *seen == count {
                let reader = serving.records().unwrap();
                let session = reader
                    .open(
                        record.record_id(),
                        worth_store::physical_runtime::RecordReadLimits::new(
                            worth_store::physical_runtime::RecordByteLimit::new(expected as u32)
                                .unwrap(),
                        ),
                    )
                    .unwrap();
                assert_eq!(
                    worth_store_testkit_store::physical_record_journeys::read_record(
                        session, expected
                    )
                    .0,
                    vec![value; expected]
                );
            }
        }
    }
    assert_eq!((large, small), (count, count));
    assert_eq!(identities.len() as u64, count * 2);
    drop(scan);
    serving.close();
}
