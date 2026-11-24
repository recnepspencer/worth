use std::hint::black_box;
use worth_relational_durability::internal_port::durability::data::checkpoint_images::DurableCheckpointFieldsFixturePort as _;

use sha2::{Digest, Sha256};

use super::*;
use crate::facade::transactions::{BulkEntityCreateIntent, CreateIntent, MutationIntent};
use crate::identity::data::PartitionId;

/// Bounded synthetic native checkpoint restore and streamed-vs-buffered image
/// digest parity court using counted work and encoded byte sizes.
#[test]
#[ignore = "counted native checkpoint restore and digest campaign; run with -- --ignored --nocapture"]
fn synthetic_native_restore_and_image_digest_benchmark() {
    crate::branch::root::capture::with_reconstruction_fixture_for_test(|fixture_lane| {
        let host_request = crate::test_execution_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host_request);

        const RECORDS: usize = 8_192;
        const DIGEST_SAMPLES: usize = 16;

        let source = runtime_with_test_schema();
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
                            .map(crate::symbols::data::ClientKey::raw)
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
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
            let committed = txn.commit(execution, &source).unwrap();
            release_test_commit_snapshot(&source, &committed);
        }

        let (image, checkpoint_report) = execution
            .run(worth_execution::ExecutionWorkCeiling::new(None), |_| {
                source.durability_authority().checkpoint(execution).unwrap()
            })
            .unwrap();
        let roots = &image.branch_roots_for_test()[0].partition_images;
        let encoded_bytes = rmp_serde::to_vec(roots).unwrap().len();
        assert!(encoded_bytes > 0, "partition checkpoint image is nonempty");

        let lane = fixture_lane;
        let mut expected_digest = None;
        for _ in 0..DIGEST_SAMPLES {
            let buffered = black_box(buffered_digest(roots));
            let streamed = black_box(
                crate::durability::data::branch_root_partition_image_digest(lane, roots).unwrap(),
            );
            assert_eq!(streamed, buffered);
            if let Some(previous) = expected_digest {
                assert_eq!(streamed, previous);
            } else {
                expected_digest = Some(streamed);
            }
        }

        let (native, native_capture_report) = execution
            .run(worth_execution::ExecutionWorkCeiling::new(None), |_| {
                source
                    .durability_authority()
                    .native_checkpoint(
                        execution,
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .unwrap()
            })
            .unwrap();
        assert!(
            native_capture_report.charged_work() > 0,
            "native checkpoint capture must charge counted work"
        );

        let mut recovered = runtime_with_test_schema();
        let ((), restore_report) = execution
            .run(worth_execution::ExecutionWorkCeiling::new(None), |_| {
                recovered
                    .durability_recovery()
                    .restore_native_checkpoint(execution, &native)
                    .unwrap();
            })
            .unwrap();
        assert!(
            restore_report.charged_work() > 0,
            "native checkpoint restore must charge counted work"
        );
        let snapshot = snapshot_for_owner_branch(&recovered, &BranchId("main".into()));
        assert_eq!(
            recovered
                .read_truth()
                .read_snapshot(&snapshot, execution)
                .expect("read request remains live")
                .unwrap()
                .entities()
                .len(),
            RECORDS
        );
        eprintln!(
        "synthetic Relational native restore: records={RECORDS} image_bytes={encoded_bytes} checkpoint_bytes={} checkpoint_work={} native_capture_work={} restore_work={} digest_samples={DIGEST_SAMPLES}",
        native.bytes().len(),
        checkpoint_report.charged_work(),
        native_capture_report.charged_work(),
        restore_report.charged_work(),
    );
    })
}

fn buffered_digest(images: &[crate::durability::data::PartitionCheckpointImage]) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth.relational.branch-root-images.v1\0");
    digest.update((images.len() as u64).to_be_bytes());
    digest.update(rmp_serde::to_vec(images).unwrap());
    digest.finalize().into()
}
