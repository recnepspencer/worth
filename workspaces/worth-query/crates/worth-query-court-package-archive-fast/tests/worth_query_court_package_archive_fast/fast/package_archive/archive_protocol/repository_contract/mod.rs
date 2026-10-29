pub(crate) use worth_query_worlds::facade::package_archive::archive_protocol::repository_contract::*;

#[test]
fn exact_repository_is_immutable_idempotent_and_identity_selected() {
    let repository = MemoryPackageArchiveRepository::default();
    let first = record_for_minimal_package(0xa5);
    let first_identity = first.claimed_package_identity().clone();
    let first_bytes = first.exact_envelope_bytes().to_vec();

    assert!(matches!(
        repository.store_exact(&first),
        WorthQueryPackageArchiveStoreOutcome::Stored
    ));
    assert!(matches!(
        repository.store_exact(&first),
        WorthQueryPackageArchiveStoreOutcome::AlreadyStoredExact
    ));

    let conflicting = record_for_minimal_package(0x5a);
    let conflict = match repository.store_exact(&conflicting) {
        WorthQueryPackageArchiveStoreOutcome::IdentityConflict(conflict) => conflict,
        outcome => panic!("different bytes did not conflict: {outcome:?}"),
    };
    assert_eq!(conflict.claimed_package_identity(), &first_identity);
    assert_eq!(
        loaded_bytes(&repository, first_identity.clone()),
        first_bytes
    );

    let second = record_for_all_family_package(0xa5);
    let second_identity = second.claimed_package_identity().clone();
    assert_ne!(second_identity, first_identity);
    assert_eq!(release_name(&first), release_name(&second));
    assert!(matches!(
        repository.load_exact(WorthQueryExactPackageArchiveRequest::new(
            second_identity.clone(),
            WorthQueryPackageEnvelopeLimits::DEFAULT,
        )),
        WorthQueryPackageArchiveLoadOutcome::NotFound
    ));
    assert!(matches!(
        repository.store_exact(&second),
        WorthQueryPackageArchiveStoreOutcome::Stored
    ));
    assert_eq!(repository.records.lock().unwrap().len(), 2);
    assert_eq!(
        loaded_bytes(&repository, second_identity),
        second.exact_envelope_bytes()
    );
}

#[test]
fn exact_load_applies_the_caller_bound_before_returning_bytes() {
    let repository = MemoryPackageArchiveRepository::default();
    let record = record_for_minimal_package(0xa5);
    let identity = record.claimed_package_identity().clone();
    let byte_length = record.exact_envelope_bytes().len() as u64;
    assert!(matches!(
        repository.store_exact(&record),
        WorthQueryPackageArchiveStoreOutcome::Stored
    ));

    let defaults = WorthQueryPackageEnvelopeLimits::DEFAULT;
    let narrow = WorthQueryPackageEnvelopeLimits::new(
        byte_length - 1,
        defaults.maximum_archive_bytes(),
        defaults.maximum_descriptive_text_bytes(),
        defaults.maximum_requirements(),
        defaults.maximum_signature_bytes(),
    );
    let denial =
        match repository.load_exact(WorthQueryExactPackageArchiveRequest::new(identity, narrow)) {
            WorthQueryPackageArchiveLoadOutcome::Denied(denial) => denial,
            outcome => panic!("over-budget load returned the wrong outcome: {outcome:?}"),
        };
    assert_eq!(
        denial.kind(),
        WorthQueryPackageArchiveRepositoryDenialKind::EnvelopeByteBudgetExceeded
    );
}

#[test]
fn repository_record_requires_a_canonical_supported_envelope() {
    let record = record_for_minimal_package(0xa5);
    let mut unsupported = record.into_exact_envelope_bytes();
    unsupported[8..10].copy_from_slice(&2_u16.to_be_bytes());
    let denial = WorthQuerySignedPackageArchiveRecord::from_untrusted_envelope_bytes(
        unsupported,
        WorthQueryPackageEnvelopeLimits::DEFAULT,
    )
    .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryPackageArchiveDenialKind::UnsupportedEnvelopeVersion
    );
    let compatibility = denial.compatibility().unwrap();
    assert_eq!(
        compatibility.layer(),
        WorthQueryPackageArchiveProtocolLayer::ReleaseEnvelope
    );
    assert_eq!(
        compatibility.posture(),
        WorthQueryPackageArchiveCompatibilityPosture::ExceedsWindow
    );

    assert!(
        WorthQuerySignedPackageArchiveRecord::from_untrusted_envelope_bytes(
            vec![0; 8],
            WorthQueryPackageEnvelopeLimits::DEFAULT,
        )
        .is_err()
    );
}

#[test]
fn loaded_repository_bytes_cannot_bypass_fresh_expected_identity_validation() {
    let expected_record = record_for_minimal_package(0xa5);
    let foreign_record = record_for_all_family_package(0xa5);
    let expected_identity = expected_record.claimed_package_identity().clone();
    let request = WorthQueryExactPackageArchiveRequest::new(
        expected_identity,
        WorthQueryPackageEnvelopeLimits::DEFAULT,
    );
    let loaded = WorthQueryUntrustedLoadedPackageArchive::from_untrusted_bytes(
        request,
        foreign_record.into_exact_envelope_bytes(),
    )
    .unwrap();
    let envelope = decode_package_release_envelope(
        loaded.untrusted_envelope_bytes(),
        loaded.envelope_limits(),
    )
    .unwrap();
    let archive =
        decode_package_archive(envelope.archive(), WorthQueryPackageArchiveLimits::DEFAULT)
            .unwrap();
    let (manifest, frames) = archive.into_parts();
    let mut reconstruction = WorthQueryPortablePackageReconstruction::begin(
        manifest,
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap();
    for frame in frames {
        let (index, record) = frame.into_parts();
        reconstruction = reconstruction.push_record(index, record).unwrap();
    }
    let independently_expected = loaded.requested_package_identity().clone();
    assert!(reconstruction
        .close()
        .unwrap()
        .materialize()
        .unwrap()
        .validate_freshly(
            WorthQueryExpectedPortablePackageIdentity::from_untrusted_identity(
                independently_expected,
            ),
        )
        .is_err());
}
