//! Actual checkpoint-source rereads retain their responsible failure boundary.

use std::{
    fs,
    path::{Path, PathBuf},
};

use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest, PersistedRecordIdentity,
    PhysicalCheckpointSource, PhysicalRootRoutingBlock, RecordArtifactFile,
    CHECKPOINT_STREAM_HEADER_RECORD_BYTES,
};
use worth_store_recovery_runtime::{
    PhysicalRecoveryBlockKind, PhysicalRecoveryOutcome, PhysicalRecoveryPlanningDenial,
    PhysicalRecoveryReleaseHeadControlDenial as Control,
    PhysicalRecoveryReleaseHeadWalkDenial as Walk,
    PhysicalRecoverySelectedReleaseHeadDenial as Denial, WorthStoreRecovery,
};

use super::*;

#[test]
fn source_root_head_and_control_failures_remain_distinct_and_deny_serving() {
    let (world, first, _) = release_reopen::released_world(1);
    assert!(first.remaining_payload_records() > 0);
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0xea; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("actual partial-drop checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);
    std::thread::Builder::new()
        .name("selected-head-diagnostics".to_owned())
        .stack_size(16 << 20)
        .spawn(move || hostile_rereads(&root))
        .unwrap()
        .join()
        .unwrap();
}

fn hostile_rereads(root: &Path) {
    let checkpoint = fs::read(root.join("families/checkpoint.current")).unwrap();
    let source = PhysicalCheckpointSource::decode_stream_header_record(
        &checkpoint[..CHECKPOINT_STREAM_HEADER_RECORD_BYTES],
    )
    .unwrap();
    let root_artifact = RecordArtifactFile::RootManifest {
        generation: source.root().generation(),
    };
    let root_path = artifact(root, root_artifact);
    let root_bytes = fs::read(&root_path).unwrap();
    let (manifest, _) = DurablePhysicalRootManifest::decode(&root_bytes, u16::MAX).unwrap();
    let (_, accumulator) = release_reopen::selected_release_certificates_from_bytes(&checkpoint);
    let heads = release_reopen::selected_head_oracle::selected_heads(root, accumulator);
    let [head] = heads.as_slice() else {
        panic!("one genuine keyed head")
    };
    let reference = manifest.release_custody_head_root().unwrap();
    let head_path = artifact(
        root,
        RecordArtifactFile::ReleaseCustodyHeadBlock {
            generation: reference.generation(),
            block: reference.block(),
        },
    );
    let reservation = route(root, &manifest, head.reservation_record());
    let CurrentPhysicalRecordPlacement::Extent(extent) = reservation else {
        panic!("bounded reservation extent")
    };
    let arena_path = root.join("families/records/arenas").join(
        RecordArtifactFile::ExtentArena {
            arena: extent.arena_range().arena().get(),
        }
        .file_name(),
    );
    let current = fs::read(
        root.join("families/records")
            .join(RecordArtifactFile::CurrentRootSelector.file_name()),
    )
    .unwrap();
    let previous_path = root
        .join("families/records")
        .join(RecordArtifactFile::PreviousRootSelector.file_name());
    let previous = fs::read(&previous_path).ok();

    for case in [
        Failure::MissingRoot,
        Failure::MissingHead,
        Failure::CorruptHead,
        Failure::MismatchedControl,
    ] {
        // Use the public phased production API. The mutation races the actual
        // planning reread, not source selection or a synthetic proof factory.
        let selected = super::request(root)
            .admit()
            .unwrap()
            .discover()
            .unwrap()
            .select()
            .unwrap();
        let path = match case {
            Failure::MissingRoot => &root_path,
            Failure::MissingHead | Failure::CorruptHead => &head_path,
            Failure::MismatchedControl => &arena_path,
        };
        let original = fs::read(path).unwrap();
        match case {
            Failure::MissingRoot | Failure::MissingHead => fs::remove_file(path).unwrap(),
            Failure::CorruptHead => {
                let mut changed = original.clone();
                changed[0] ^= 1;
                fs::write(path, changed).unwrap();
            }
            Failure::MismatchedControl => {
                super::tamper::substitute_selected_reservation(root, reservation)
            }
        }
        let outcome = match selected.plan() {
            Err(outcome) => outcome,
            Ok(_) => panic!("hostile source reread authorized a plan"),
        };
        fs::write(path, original).unwrap();
        assert_failure(
            outcome,
            case,
            source.root().generation(),
            reference,
            head.reservation_record(),
        );
        assert_eq!(
            fs::read(root.join("families/checkpoint.current")).unwrap(),
            checkpoint
        );
        assert_eq!(
            fs::read(
                root.join("families/records")
                    .join(RecordArtifactFile::CurrentRootSelector.file_name())
            )
            .unwrap(),
            current
        );
        assert_eq!(fs::read(&previous_path).ok(), previous);
    }
    let outcome = WorthStoreRecovery::recover(super::request(root));
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("restored actual-media twin must recover: {outcome:?}")
    };
    assert_eq!(handoff.core().recovery_effect_count(), 0);
    super::open_serving_with_seal(root, handoff.into_core().into_checkpoint_custody().unwrap());
}

#[derive(Clone, Copy)]
enum Failure {
    MissingRoot,
    MissingHead,
    CorruptHead,
    MismatchedControl,
}

fn assert_failure(
    outcome: PhysicalRecoveryOutcome,
    case: Failure,
    source_generation: u64,
    head_reference: worth_store_physical_format::ReleaseCustodyHeadBlockReferenceV1,
    reservation: PersistedRecordIdentity,
) {
    let PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
        panic!("hostile reread must block before effects: {outcome:?}")
    };
    assert_eq!(
        blocked.cause().damage(),
        Some(PhysicalRecoveryBlockKind::SelectedCustody)
    );
    assert_eq!(blocked.recovery_effects(), 0);
    assert_eq!(
        blocked.evidence().artifact.as_deref(),
        Some("checkpoint-source-release-head-v2")
    );
    let Some(PhysicalRecoveryPlanningDenial::SelectedReleaseHead(denial)) =
        &blocked.evidence().planning_denial
    else {
        panic!("typed head admission denial: {:?}", blocked.evidence())
    };
    match case {
        Failure::MissingRoot => assert!(
            matches!(denial, Denial::SourceRootIntegrity { generation, denial: worth_store_recovery_runtime::PhysicalRecoveryIntegrityRejection::Absent } if *generation == source_generation),
            "{denial:?}"
        ),
        Failure::CorruptHead => assert!(
            matches!(
                denial,
                Denial::HeadWalk(Walk::Format(
                    worth_store_physical_format::ReleaseCustodyHeadDenial::Frame(
                        worth_store_physical_format::DurableFrameDenial::WrongMagic
                    )
                ))
            ),
            "{denial:?}"
        ),
        Failure::MissingHead => assert!(
            matches!(denial, Denial::HeadWalk(Walk::Read(worth_store_recovery_runtime::PhysicalRecoveryReleaseHeadReadDenial::MissingBytes { reference })) if *reference == head_reference),
            "{denial:?}"
        ),
        Failure::MismatchedControl => assert!(
            matches!(denial, Denial::Control(Control::FrameDigestMismatch { record }) if *record == reservation),
            "{denial:?}"
        ),
    }
}

fn artifact(root: &Path, artifact: RecordArtifactFile) -> PathBuf {
    root.join("families/records/roots")
        .join(artifact.file_name())
}

fn route(
    root: &Path,
    manifest: &DurablePhysicalRootManifest,
    record: PersistedRecordIdentity,
) -> CurrentPhysicalRecordPlacement {
    let mut pending = manifest.routing_root().into_iter().collect::<Vec<_>>();
    let mut reads = 0;
    while let Some(reference) = pending.pop() {
        reads += 1;
        assert!(reads <= 64, "fixture route traversal must remain bounded");
        let bytes = fs::read(artifact(
            root,
            RecordArtifactFile::RootRoutingBlock {
                generation: reference.generation(),
                block: reference.block(),
            },
        ))
        .unwrap();
        let (block, _) =
            PhysicalRootRoutingBlock::decode(&bytes, manifest.node_capacity()).unwrap();
        if let Some(entries) = block.entries() {
            if let Some(route) = entries.iter().find(|route| route.record() == record) {
                return *route;
            }
        } else {
            pending.extend_from_slice(block.children().unwrap());
        }
    }
    panic!("genuine selected reservation must have a routed placement")
}
