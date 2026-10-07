//! A real recovered seal cannot authorize externally changed selected head bytes.

use super::*;
use worth_store::physical_runtime::RecordBootstrapDenial;
use worth_store_physical_format::{
    DurablePhysicalRootManifest, PhysicalCheckpointSource, RecordArtifactFile,
    CHECKPOINT_STREAM_HEADER_RECORD_BYTES,
};

#[test]
fn sealed_serving_rereads_funded_head_and_denies_post_seal_corruption() {
    std::thread::Builder::new()
        .name("sealed-head-freshness".into())
        .stack_size(16 << 20)
        .spawn(|| {
            let (world, receipt, _) = release_reopen::released_world(1);
            assert!(receipt.remaining_payload_records() > 0);
            checkpoint(world.serving(), [0xd1; 32]);
            let retained = world.retained_root();
            let root = retained.path();
            drop(world);
            let checkpoint_bytes = fs::read(root.join("families/checkpoint.current")).unwrap();
            let source = PhysicalCheckpointSource::decode_stream_header_record(
                &checkpoint_bytes[..CHECKPOINT_STREAM_HEADER_RECORD_BYTES],
            )
            .unwrap();
            let roots = root.join("families/records/roots");
            let manifest_bytes = fs::read(
                roots.join(
                    RecordArtifactFile::RootManifest {
                        generation: source.root().generation(),
                    }
                    .file_name(),
                ),
            )
            .unwrap();
            let (manifest, _) =
                DurablePhysicalRootManifest::decode(&manifest_bytes, u16::MAX).unwrap();
            let head = manifest
                .release_custody_head_root()
                .expect("genuine selected head tree");
            let path = roots.join(
                RecordArtifactFile::ReleaseCustodyHeadBlock {
                    generation: head.generation(),
                    block: head.block(),
                }
                .file_name(),
            );
            let original = fs::read(&path).unwrap();
            let core = recover_core(root);
            let policy = core.residency_policy();
            let format = AdmittedPhysicalRecordFormat::admit(policy.record_format());
            let observer = core.certification_residency_allocations();
            assert!(core.selected_head_walk_owned_heap_bytes().unwrap() > 0);
            let seal = core
                .into_checkpoint_custody()
                .expect("real independent head rejoin");
            let mut changed = original.clone();
            let index = changed.len() / 2;
            changed[index] ^= 0x40;
            fs::write(&path, &changed).unwrap();
            let altered = snapshot_family(root);
            let TransitionOutcome::Denied(denial) =
                open_with_policy(root, seal, format, policy).into_raw()
            else {
                panic!("post-seal head mutation must deny Serving")
            };
            assert_eq!(
                denial.reason(),
                RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch
            );
            let runtime = denial.into_runtime();
            assert_eq!(runtime.media_counters().replacements(), 0);
            assert_eq!(runtime.media_counters().deletions(), 0);
            assert_eq!(snapshot_family(root), altered);
            runtime.close();
            backing_census::assert_disposed("post-seal head mismatch", |dimension| {
                let counters = observer.snapshot().for_dimension(dimension);
                (
                    counters.active_units(),
                    counters.admitted_units(),
                    counters.released_units(),
                )
            });
            fs::write(&path, &original).unwrap();
            let fresh = recover_core(root);
            let seal = fresh.into_checkpoint_custody().unwrap();
            let TransitionOutcome::Success(serving) =
                open_with_policy(root, seal, format, policy).into_raw()
            else {
                panic!("fresh real seal must open after exact media restoration")
            };
            checkpoint(&serving, [0xd2; 32]);
            serving.close();
            drop(recover_core(root));
        })
        .unwrap()
        .join()
        .expect("real head freshness worker");
}
