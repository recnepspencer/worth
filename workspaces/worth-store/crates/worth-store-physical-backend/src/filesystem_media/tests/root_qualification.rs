use super::super::*;
use worth_proof::{ProofOutcomeKind, TransitionOutcome};

fn request(root: &std::path::Path) -> FilesystemQualificationRequest {
    FilesystemQualificationRequest::production(
        root,
        FilesystemAccessPosture::CoordinatedServiceAccount,
    )
}

fn qualified(root: &std::path::Path) -> QualifiedFilesystemMedia {
    match FilesystemMediaOwner::qualify(request(root)).into_raw() {
        TransitionOutcome::Success(qualified) => qualified,
        TransitionOutcome::Denied(value) => panic!("root qualification denied: {value:?}"),
        TransitionOutcome::Deferred(value) => panic!("root qualification deferred: {value:?}"),
        TransitionOutcome::Stale(value) => panic!("root qualification stale: {value:?}"),
        TransitionOutcome::RebindRequired(value) => panic!("root qualification rebind: {value:?}"),
        TransitionOutcome::Failed(value) => panic!("root qualification failed: {value:?}"),
    }
}

#[test]
fn root_qualification_creates_and_durably_reopens_one_stable_identity() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let first = qualified(&root);
    let identity = first.store_identity();
    let profile_root = first.profile().root_identity();
    let identity_bytes = std::fs::read(root.join("namespace/identity")).unwrap();
    let identity_modified = std::fs::metadata(root.join("namespace/identity"))
        .unwrap()
        .modified()
        .unwrap();
    let (owner, _profile, basis, capabilities, stable) = first.into_runtime_parts();
    assert_eq!(stable, identity);
    assert_eq!(basis.root_identity(), profile_root);
    assert!(capabilities.data_sync().is_some());
    assert!(capabilities.direct_io().is_none());
    owner.close();

    let second = qualified(&root);
    assert_eq!(second.store_identity(), identity);
    assert_eq!(second.profile().root_identity(), profile_root);
    assert_eq!(
        std::fs::read(root.join("namespace/identity")).unwrap(),
        identity_bytes
    );
    assert_eq!(
        std::fs::metadata(root.join("namespace/identity"))
            .unwrap()
            .modified()
            .unwrap(),
        identity_modified
    );
    let (owner, _, _, _, _) = second.into_runtime_parts();
    owner.close();
}

#[test]
fn access_posture_and_cross_root_basis_substitution_fail_closed() {
    let parent = tempfile::tempdir().unwrap();
    let root_a = parent.path().join("a");
    let root_b = parent.path().join("b");
    let qualified_a = qualified(&root_a);
    let report_a = qualified_a.qualification_report();
    assert_eq!(
        report_a.access_contract(),
        FilesystemAccessContract::CoordinatedServiceAccount
    );
    qualified_a.close();

    let unmanaged = FilesystemMediaOwner::qualify(FilesystemQualificationRequest::production(
        &root_b,
        FilesystemAccessPosture::UnmanagedWritersPossible,
    ));
    assert_eq!(unmanaged.kind(), ProofOutcomeKind::Denied);
    assert!(!root_b.exists());

    let qualified_b = qualified(&root_b);
    let identity_b = qualified_b.store_identity();
    qualified_b.close();
    let substituted =
        FilesystemMediaOwner::qualify(request(&root_b).require_current_profile(report_a));
    assert_eq!(substituted.kind(), ProofOutcomeKind::Stale);
    let reopened_b = qualified(&root_b);
    assert_eq!(reopened_b.store_identity(), identity_b);
    reopened_b.close();
}

#[test]
fn optional_capability_handles_carry_observed_granularity() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let media = qualified(&root);
    assert!(media.profile().allocation_granularity().get() > 0);
    assert!(media.capabilities().preallocation().is_none());
    assert!(media.capabilities().memory_map().is_none());
    assert!(media.capabilities().direct_io().is_none());
    let (owner, _, _, _, _) = media.into_runtime_parts();
    owner.close();
}

#[test]
fn every_profile_basis_dimension_invalidates_independently() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let media = qualified(&root);
    let report = media.qualification_report();
    media.close();

    let contract = report
        .clone()
        .with_contract_version_for_certification(report.contract_version() + 1);
    assert!(matches!(
        FilesystemMediaOwner::qualify(request(&root).require_current_profile(contract)).into_raw(),
        TransitionOutcome::RebindRequired(
            MediaQualificationRebindRequired::QualificationContractChanged { .. }
        )
    ));

    let volume = report
        .clone()
        .with_volume_identity_for_certification([0x51; 32]);
    assert!(matches!(
        FilesystemMediaOwner::qualify(request(&root).require_current_profile(volume)).into_raw(),
        TransitionOutcome::RebindRequired(MediaQualificationRebindRequired::VolumeChanged { .. })
    ));

    let profile = report
        .clone()
        .with_profile_digest_for_certification([0xA7; 32]);
    assert!(matches!(
        FilesystemMediaOwner::qualify(request(&root).require_current_profile(profile)).into_raw(),
        TransitionOutcome::RebindRequired(
            MediaQualificationRebindRequired::BackendProfileChanged { .. }
        )
    ));

    let backend_build = report
        .clone()
        .with_backend_build_identity_for_certification([0xB4; 32]);
    assert!(matches!(
        FilesystemMediaOwner::qualify(request(&root).require_current_profile(backend_build))
            .into_raw(),
        TransitionOutcome::RebindRequired(
            MediaQualificationRebindRequired::BackendProfileChanged { .. }
        )
    ));

    let other_root = parent.path().join("other");
    let other = qualified(&other_root);
    other.close();
    assert!(matches!(
        FilesystemMediaOwner::qualify(request(&other_root).require_current_profile(report))
            .into_raw(),
        TransitionOutcome::Stale(_)
    ));
}

#[test]
fn certification_qualification_exercises_real_media_and_removes_all_scratch() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let outcome = FilesystemMediaOwner::qualify(FilesystemQualificationRequest::certification(
        &root,
        FilesystemAccessPosture::CoordinatedServiceAccount,
    ));
    let media = match outcome.into_raw() {
        TransitionOutcome::Success(media) => media,
        _ => panic!("certification qualification must complete"),
    };
    assert_eq!(media.mode(), FilesystemQualificationMode::Certification);
    let namespace_names = std::fs::read_dir(root.join("namespace"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect::<Vec<_>>();
    assert_eq!(namespace_names.len(), 2);
    assert!(namespace_names.iter().any(|name| name == "identity"));
    assert!(namespace_names.iter().any(|name| name == "mutation.lock"));
    assert_eq!(media.counters().cleanup_actions(), 3);
    assert_eq!(media.counters().preserved_residue(), 0);
    assert!(
        media
            .counters()
            .requested_bytes_for(MediaOperationRole::PositionedWrite)
            >= 1_048_576
    );
    assert_eq!(
        media
            .counters()
            .requested_bytes_for(MediaOperationRole::Append),
        257
    );
    assert_eq!(media.counters().peak_request_width_bytes(), 65_536);
    assert!(media.counters().requested_heap_capacity_bytes() <= 65_536);
    let (owner, _, _, _, _) = media.into_runtime_parts();
    owner.close();
}

#[test]
fn classification_resumes_exact_scaffold_but_rejects_foreign_and_damaged_roots() {
    use worth_store_physical_format::store_namespace::{
        ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
    };

    let parent = tempfile::tempdir().unwrap();

    let foreign = parent.path().join("foreign");
    std::fs::create_dir(&foreign).unwrap();
    std::fs::write(foreign.join("customer-data"), b"belongs to caller").unwrap();
    assert!(matches!(
        FilesystemMediaOwner::qualify(request(&foreign)).into_raw(),
        TransitionOutcome::Denied(MediaQualificationDenial::OwnerPreEffect { .. })
    ));
    assert_eq!(
        std::fs::read(foreign.join("customer-data")).unwrap(),
        b"belongs to caller"
    );
    assert!(!foreign.join("namespace").exists());

    let incomplete = parent.path().join("incomplete");
    std::fs::create_dir_all(incomplete.join("namespace")).unwrap();
    let resumed = match FilesystemMediaOwner::qualify(request(&incomplete)).into_raw() {
        TransitionOutcome::Success(media) => media,
        _ => panic!("exact C.4 scaffold must be retryable"),
    };
    assert!(incomplete.join("families").is_dir());
    assert!(incomplete.join("staging").is_dir());
    assert!(incomplete.join("namespace/identity").is_file());
    resumed.into_runtime_parts().0.close();

    let damaged = parent.path().join("damaged");
    std::fs::create_dir_all(damaged.join("namespace")).unwrap();
    std::fs::create_dir(damaged.join("families")).unwrap();
    std::fs::create_dir(damaged.join("staging")).unwrap();
    std::fs::write(damaged.join("namespace/mutation.lock"), []).unwrap();
    let proposed = ProposedStoreIdentity::from_nonzero_bytes([7; 16]).unwrap();
    let record = StoreNamespaceIdentityRecord::new(StoreNamespaceVersion::CURRENT, proposed);
    std::fs::write(damaged.join("namespace/identity"), record.encode()).unwrap();
    std::fs::remove_dir(damaged.join("families")).unwrap();
    assert!(matches!(
        FilesystemMediaOwner::qualify(request(&damaged)).into_raw(),
        TransitionOutcome::Denied(MediaQualificationDenial::OwnerPreEffect { .. })
    ));
    assert!(!damaged.join("families").exists());
}

#[test]
fn initialized_roots_preserve_valid_residue_but_reject_ambiguous_staged_names() {
    use worth_store_physical_format::store_namespace::{
        NamespaceInitializationAttempt, StagedNamespaceName,
    };

    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    qualified(&root).close();
    let attempt = NamespaceInitializationAttempt::from_nonzero_bytes([9; 16]).unwrap();
    let residue = StagedNamespaceName::for_identity(attempt);
    let residue_path = root.join("namespace").join(residue.as_str());
    std::fs::write(&residue_path, b"preserved inspection residue").unwrap();
    qualified(&root).close();
    assert_eq!(
        std::fs::read(&residue_path).unwrap(),
        b"preserved inspection residue"
    );

    let ambiguous = root.join("namespace/identity-not-canonical.staged");
    std::fs::write(&ambiguous, b"unknown staged-looking entry").unwrap();
    assert!(matches!(
        FilesystemMediaOwner::qualify(request(&root)).into_raw(),
        TransitionOutcome::Denied(MediaQualificationDenial::OwnerPreEffect { .. })
    ));
    assert!(ambiguous.exists());
    assert!(residue_path.exists());
}

#[expect(
    clippy::disallowed_methods,
    reason = "category: coordinated storage-owner unit-test harness; purpose: opened root binding survives a real name aba; source: worth-store-physical-backend::filesystem_media::tests::root_qualification::opened_root_binding_survives_a_real_name_aba; lexical counts: std::thread::spawn=1, std::thread::scope=0, std::thread::Scope::spawn=0, available_parallelism=0; barriers, joins and original bounds coordinate the storage-owner claim"
)]
#[test]
fn opened_root_binding_survives_a_real_name_aba() {
    let parent = tempfile::tempdir().unwrap();
    let root_a = parent.path().join("a");
    let root_b = parent.path().join("b");
    let first_a = qualified(&root_a);
    let expected = first_a.qualification_report();
    first_a.close();
    qualified(&root_b).close();

    let gate = MediaPauseGate::for_certification();
    let schedule = MediaFaultSchedule::for_certification(vec![MediaFaultRule::for_certification(
        MediaOperationRole::ObserveRootProfile,
        2,
        MediaFaultDirective::PauseBefore(gate.clone()),
    )])
    .unwrap();
    let admitted_root = root_a.clone();
    let admission = std::thread::spawn(move || {
        FilesystemMediaOwner::qualify(request(&admitted_root).with_fault_schedule(schedule))
            .into_raw()
    });
    gate.wait_until_reached();

    let held_a = parent.path().join("held-a");
    let held_b = parent.path().join("held-b");
    let swapped = std::fs::rename(&root_a, &held_a).is_ok();
    if swapped {
        std::fs::rename(&root_b, &root_a).unwrap();
        std::fs::rename(&root_a, &held_b).unwrap();
        std::fs::rename(&held_a, &root_a).unwrap();
        std::fs::rename(&held_b, &root_b).unwrap();
    } else {
        assert!(
            root_a.is_dir(),
            "OS denial must preserve the opened root name"
        );
    }
    gate.release();

    let media = match admission.join().unwrap() {
        TransitionOutcome::Success(media) => media,
        _ => panic!("name ABA must retain the opened A binding"),
    };
    assert_eq!(media.qualification_report(), expected);
    media.close();
}

#[expect(
    clippy::disallowed_methods,
    reason = "category: coordinated storage-owner unit-test harness; purpose: different root left at the ambient name fails after ownership; source: worth-store-physical-backend::filesystem_media::tests::root_qualification::different_root_left_at_the_ambient_name_fails_after_ownership; lexical counts: std::thread::spawn=1, std::thread::scope=0, std::thread::Scope::spawn=0, available_parallelism=0; barriers, joins and original bounds coordinate the storage-owner claim"
)]
#[test]
fn different_root_left_at_the_ambient_name_fails_after_ownership() {
    let parent = tempfile::tempdir().unwrap();
    let root_a = parent.path().join("a");
    let root_b = parent.path().join("b");
    qualified(&root_a).close();
    qualified(&root_b).close();

    let gate = MediaPauseGate::for_certification();
    let schedule = MediaFaultSchedule::for_certification(vec![MediaFaultRule::for_certification(
        MediaOperationRole::ObserveRootProfile,
        2,
        MediaFaultDirective::PauseBefore(gate.clone()),
    )])
    .unwrap();
    let admitted_root = root_a.clone();
    let admission = std::thread::spawn(move || {
        FilesystemMediaOwner::qualify(request(&admitted_root).with_fault_schedule(schedule))
            .into_raw()
    });
    gate.wait_until_reached();

    let held_a = parent.path().join("held-a");
    let replaced = std::fs::rename(&root_a, &held_a).is_ok();
    if replaced {
        std::fs::rename(&root_b, &root_a).unwrap();
    } else {
        assert!(
            root_a.is_dir(),
            "OS denial must preserve the opened root name"
        );
    }
    gate.release();

    let outcome = admission.join().unwrap();
    if replaced {
        assert!(matches!(
            outcome,
            TransitionOutcome::Failed(MediaQualificationFailure::PostOwnership {
                cause,
                ..
            }) if matches!(*cause, MediaQualificationPostOwnershipCause::RootIdentityChanged(_))
        ));
    } else {
        let TransitionOutcome::Success(media) = outcome else {
            panic!("OS-denied replacement must preserve original admission");
        };
        media.close();
    }
}
