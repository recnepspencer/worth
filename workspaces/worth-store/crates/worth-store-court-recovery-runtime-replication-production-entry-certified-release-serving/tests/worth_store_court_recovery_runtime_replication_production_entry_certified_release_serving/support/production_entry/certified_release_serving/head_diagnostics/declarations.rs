//! Actual checkpoint-source rereads retain their responsible failure boundary.

pub(crate) use {std::fs, std::path::Path, std::path::PathBuf};

pub(crate) use {worth_store_physical_format::CurrentPhysicalRecordPlacement, worth_store_physical_format::DurablePhysicalRootManifest, worth_store_physical_format::PersistedRecordIdentity, worth_store_physical_format::PhysicalCheckpointSource, worth_store_physical_format::PhysicalRootRoutingBlock, worth_store_physical_format::RecordArtifactFile, worth_store_physical_format::CHECKPOINT_STREAM_HEADER_RECORD_BYTES};
pub(crate) use {worth_store_recovery_runtime::PhysicalRecoveryBlockKind, worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial, worth_store_recovery_runtime::PhysicalRecoveryReleaseHeadControlDenial as Control, worth_store_recovery_runtime::PhysicalRecoveryReleaseHeadWalkDenial as Walk, worth_store_recovery_runtime::PhysicalRecoverySelectedReleaseHeadDenial as Denial, worth_store_recovery_runtime::WorthStoreRecovery};

pub(crate) use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::open_serving_with_seal, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery::recover, worth_store_testkit_recovery_runtime::production_entry::release_reopen, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request, worth_store_testkit_recovery_runtime::certified_release_serving::tamper};

pub(crate) fn hostile_rereads(root: &Path) {
    let checkpoint = std::fs::read(root.join("families/checkpoint.current")).unwrap();
    let source = worth_store_physical_format::PhysicalCheckpointSource::decode_stream_header_record(
        &checkpoint[..CHECKPOINT_STREAM_HEADER_RECORD_BYTES],
    )
    .unwrap();
    let root_artifact = worth_store_physical_format::RecordArtifactFile::RootManifest {
        generation: source.root().generation(),
    };
    let root_path = artifact(root, root_artifact);
    let root_bytes = std::fs::read(&root_path).unwrap();
    let (manifest, _) = worth_store_physical_format::DurablePhysicalRootManifest::decode(&root_bytes, u16::MAX).unwrap();
    let (_, accumulator) = worth_store_testkit_recovery_runtime::production_entry::release_reopen::selected_release_certificates_from_bytes(&checkpoint);
    let heads = worth_store_testkit_recovery_runtime::release_reopen::selected_head_oracle::selected_heads(root, accumulator);
    let [head] = heads.as_slice() else {
        panic!("one genuine keyed head")
    };
    let reference = manifest.release_custody_head_root().unwrap();
    let head_path = artifact(
        root,
        worth_store_physical_format::RecordArtifactFile::ReleaseCustodyHeadBlock {
            generation: reference.generation(),
            block: reference.block(),
        },
    );
    let reservation = route(root, &manifest, head.reservation_record());
    let worth_store_physical_format::CurrentPhysicalRecordPlacement::Extent(extent) = reservation else {
        panic!("bounded reservation extent")
    };
    let arena_path = root.join("families/records/arenas").join(
        worth_store_physical_format::RecordArtifactFile::ExtentArena {
            arena: extent.arena_range().arena().get(),
        }
        .file_name(),
    );
    let current = std::fs::read(
        root.join("families/records")
            .join(worth_store_physical_format::RecordArtifactFile::CurrentRootSelector.file_name()),
    )
    .unwrap();
    let previous_path = root
        .join("families/records")
        .join(worth_store_physical_format::RecordArtifactFile::PreviousRootSelector.file_name());
    let previous = std::fs::read(&previous_path).ok();

    for case in [
        crate::production_entry::certified_release_serving::head_diagnostics::Failure::MissingRoot,
        crate::production_entry::certified_release_serving::head_diagnostics::Failure::MissingHead,
        crate::production_entry::certified_release_serving::head_diagnostics::Failure::CorruptHead,
        crate::production_entry::certified_release_serving::head_diagnostics::Failure::MismatchedControl,
    ] {
        // Use the public phased production API. The mutation races the actual
        // planning reread, not source selection or a synthetic proof factory.
        let selected = worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(root)
            .admit()
            .unwrap()
            .discover()
            .unwrap()
            .select()
            .unwrap();
        let path = match case {
            crate::production_entry::certified_release_serving::head_diagnostics::Failure::MissingRoot => &root_path,
            crate::production_entry::certified_release_serving::head_diagnostics::Failure::MissingHead | crate::production_entry::certified_release_serving::head_diagnostics::Failure::CorruptHead => &head_path,
            crate::production_entry::certified_release_serving::head_diagnostics::Failure::MismatchedControl => &arena_path,
        };
        let original = std::fs::read(path).unwrap();
        match case {
            crate::production_entry::certified_release_serving::head_diagnostics::Failure::MissingRoot | crate::production_entry::certified_release_serving::head_diagnostics::Failure::MissingHead => std::fs::remove_file(path).unwrap(),
            crate::production_entry::certified_release_serving::head_diagnostics::Failure::CorruptHead => {
                let mut changed = original.clone();
                changed[0] ^= 1;
                std::fs::write(path, changed).unwrap();
            }
            crate::production_entry::certified_release_serving::head_diagnostics::Failure::MismatchedControl => {
                worth_store_testkit_recovery_runtime::certified_release_serving::tamper::substitute_selected_reservation(root, reservation)
            }
        }
        let outcome = match selected.plan() {
            Err(outcome) => outcome,
            Ok(_) => panic!("hostile source reread authorized a plan"),
        };
        std::fs::write(path, original).unwrap();
        assert_failure(
            outcome,
            case,
            source.root().generation(),
            reference,
            head.reservation_record(),
        );
        assert_eq!(
            std::fs::read(root.join("families/checkpoint.current")).unwrap(),
            checkpoint
        );
        assert_eq!(
            std::fs::read(
                root.join("families/records")
                    .join(worth_store_physical_format::RecordArtifactFile::CurrentRootSelector.file_name())
            )
            .unwrap(),
            current
        );
        assert_eq!(std::fs::read(&previous_path).ok(), previous);
    }
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(root));
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("restored actual-media twin must recover: {outcome:?}")
    };
    assert_eq!(handoff.core().recovery_effect_count(), 0);
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::open_serving_with_seal(
        root,
        handoff
            .into_core()
            .certification_release_media_into_checkpoint_custody()
            .unwrap(),
    );
}

#[derive(Clone, Copy)]
pub(crate) enum Failure {
    MissingRoot,
    MissingHead,
    CorruptHead,
    MismatchedControl,
}

pub(crate) fn assert_failure(
    outcome: PhysicalRecoveryOutcome,
    case: Failure,
    source_generation: u64,
    head_reference: worth_store_physical_format::ReleaseCustodyHeadBlockReferenceV1,
    reservation: PersistedRecordIdentity,
) {
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
        panic!("hostile reread must block before effects: {outcome:?}")
    };
    assert_eq!(
        blocked.cause().damage(),
        Some(worth_store_recovery_runtime::PhysicalRecoveryBlockKind::SelectedCustody)
    );
    assert_eq!(blocked.recovery_effects(), 0);
    assert_eq!(
        blocked.evidence().artifact.as_deref(),
        Some("checkpoint-source-release-head-v2")
    );
    let Some(worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial::SelectedReleaseHead(denial)) =
        &blocked.evidence().planning_denial
    else {
        panic!("typed head admission denial: {:?}", blocked.evidence())
    };
    match case {
        crate::production_entry::certified_release_serving::head_diagnostics::Failure::MissingRoot => assert!(
            matches!(denial, worth_store_recovery_runtime::PhysicalRecoverySelectedReleaseHeadDenial::SourceRootIntegrity { generation, denial: worth_store_recovery_runtime::PhysicalRecoveryIntegrityRejection::Absent } if *generation == source_generation),
            "{denial:?}"
        ),
        crate::production_entry::certified_release_serving::head_diagnostics::Failure::CorruptHead => assert!(
            matches!(
                denial,
                worth_store_recovery_runtime::PhysicalRecoverySelectedReleaseHeadDenial::HeadWalk(worth_store_recovery_runtime::PhysicalRecoveryReleaseHeadWalkDenial::Format(
                    worth_store_physical_format::ReleaseCustodyHeadDenial::Frame(
                        worth_store_physical_format::DurableFrameDenial::WrongMagic
                    )
                ))
            ),
            "{denial:?}"
        ),
        crate::production_entry::certified_release_serving::head_diagnostics::Failure::MissingHead => assert!(
            matches!(denial, worth_store_recovery_runtime::PhysicalRecoverySelectedReleaseHeadDenial::HeadWalk(worth_store_recovery_runtime::PhysicalRecoveryReleaseHeadWalkDenial::Read(worth_store_recovery_runtime::PhysicalRecoveryReleaseHeadReadDenial::MissingBytes { reference })) if *reference == head_reference),
            "{denial:?}"
        ),
        crate::production_entry::certified_release_serving::head_diagnostics::Failure::MismatchedControl => assert!(
            matches!(denial, worth_store_recovery_runtime::PhysicalRecoverySelectedReleaseHeadDenial::Control(worth_store_recovery_runtime::PhysicalRecoveryReleaseHeadControlDenial::FrameDigestMismatch { record }) if *record == reservation),
            "{denial:?}"
        ),
    }
}

pub(crate) fn artifact(root: &Path, artifact: RecordArtifactFile) -> PathBuf {
    root.join("families/records/roots")
        .join(artifact.file_name())
}

pub(crate) fn route(
    root: &Path,
    manifest: &DurablePhysicalRootManifest,
    record: PersistedRecordIdentity,
) -> CurrentPhysicalRecordPlacement {
    let mut pending = manifest.routing_root().into_iter().collect::<Vec<_>>();
    let mut reads = 0;
    while let Some(reference) = pending.pop() {
        reads += 1;
        assert!(reads <= 64, "fixture route traversal must remain bounded");
        let bytes = std::fs::read(artifact(
            root,
            worth_store_physical_format::RecordArtifactFile::RootRoutingBlock {
                generation: reference.generation(),
                block: reference.block(),
            },
        ))
        .unwrap();
        let (block, _) =
            worth_store_physical_format::PhysicalRootRoutingBlock::decode(&bytes, manifest.node_capacity()).unwrap();
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