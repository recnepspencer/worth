use std::hint::black_box;
use std::time::Instant;

use sha2::{Digest, Sha256};

use super::*;
use crate::facade::transactions::{BulkEntityCreateIntent, CreateIntent, MutationIntent};
use crate::identity::data::PartitionId;

/// Manual cold-lane measurement; real House restore remains the production court.
#[test]
#[ignore = "run explicitly to measure a bounded synthetic native checkpoint restore"]
fn synthetic_native_restore_and_image_digest_benchmark() {
    const RECORDS: usize = 8_192;
    const DIGEST_SAMPLES: usize = 16;

    let source = persisted_runtime_with_test_schema();
    let names = (0..RECORDS)
        .map(|index| format!("restore-bench-{index:04}"))
        .collect::<Vec<_>>();
    for chunk in names.chunks(4_096) {
        let mut txn = test_owner_begin_transaction_for_main(&source);
        txn.push_batch(
            WorkerIntentBatch::new("restore-bench").push(MutationIntent::Create(
                CreateIntent::BulkEntities(BulkEntityCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_keys: chunk
                        .iter()
                        .map(|name| crate::symbols::data::ClientKey::raw(name))
                        .collect(),
                    field_patches: chunk
                        .iter()
                        .map(|name| {
                            single_string_aspect_field_patch(
                                aspect_key("name"),
                                field_key("name"),
                                name,
                            )
                        })
                        .collect(),
                }),
            )),
        )
        .unwrap();
        let committed = txn.commit(&source).unwrap();
        release_test_commit_snapshot(&source, &committed);
    }

    let image = source.durability_authority().checkpoint().unwrap();
    let roots = &image.branch_roots[0].partition_images;
    let encoded_bytes = rmp_serde::to_vec(roots).unwrap().len();
    let buffered_started = Instant::now();
    let buffered = (0..DIGEST_SAMPLES)
        .map(|_| black_box(buffered_digest(roots)))
        .last()
        .unwrap();
    let buffered_elapsed = buffered_started.elapsed();
    let streamed_started = Instant::now();
    let streamed = (0..DIGEST_SAMPLES)
        .map(|_| {
            black_box(crate::durability::data::branch_root_partition_image_digest(roots).unwrap())
        })
        .last()
        .unwrap();
    let streamed_elapsed = streamed_started.elapsed();
    assert_eq!(streamed, buffered);

    let native = source.durability_authority().native_checkpoint().unwrap();
    let mut recovered = persisted_runtime_with_test_schema();
    let restore_started = Instant::now();
    recovered
        .durability_recovery()
        .restore_native_checkpoint(&native)
        .unwrap();
    let restore_elapsed = restore_started.elapsed();
    let snapshot = snapshot_for_owner_branch(&recovered, &BranchId("main".into()));
    assert_eq!(
        recovered
            .read_truth()
            .read_snapshot(&snapshot)
            .unwrap()
            .entities()
            .len(),
        RECORDS
    );
    eprintln!(
        "synthetic Relational native restore: records={RECORDS} image_bytes={encoded_bytes} checkpoint_bytes={} buffered_digest_{}={buffered_elapsed:?} streamed_digest_{}={streamed_elapsed:?} reopen={restore_elapsed:?}",
        native.bytes().len(),
        DIGEST_SAMPLES,
        DIGEST_SAMPLES,
    );
}

fn buffered_digest(images: &[crate::durability::data::PartitionCheckpointImage]) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth.relational.branch-root-images.v1\0");
    digest.update((images.len() as u64).to_be_bytes());
    digest.update(rmp_serde::to_vec(images).unwrap());
    digest.finalize().into()
}
