#[allow(unused_imports)]
use crate::bootstrap_catalog::*;
use {
    worth_store_contracts::AcceptedHandoffReadiness,
    worth_store_contracts::DurableArtifactFamilyId,
    worth_store_contracts::HandoffEvidenceDigestSet, worth_store_contracts::StableDigest,
    worth_store_contracts::ROADMAP_2_S1_SCOPE,
};
use {
    worth_store_layout_indexes::bootstrap_catalog,
    worth_store_layout_indexes::declarations::layout_declarations,
    worth_store_layout_indexes::BootstrapOnlyAccessDenied,
    worth_store_layout_indexes::BootstrapOnlyAccessPath,
};
use {
    worth_store_physical_certification::FixtureCapabilityDeclaration,
    worth_store_physical_certification::FixtureMutationBoundary,
    worth_store_physical_certification::LargeStoreFixtureProfile,
    worth_store_physical_certification::PhysicalFixtureBuilder,
    worth_store_physical_certification::ProductionBackedFixtureMaterialization,
};
use {
    worth_store_physical_format::physical_bootstrap_catalog,
    worth_store_physical_format::InMemoryPhysicalFormatModel,
    worth_store_physical_format::InMemoryPhysicalFormatModelRequest,
    worth_store_physical_format::PhysicalExtentId, worth_store_physical_format::PhysicalGeneration,
    worth_store_physical_format::PhysicalGenerationAuthority,
    worth_store_physical_format::PhysicalPageId, worth_store_physical_format::PhysicalRecordSlot,
    worth_store_physical_format::PhysicalRootReference,
    worth_store_physical_format::PhysicalSegmentId,
    worth_store_physical_format::PlatformPhysicalAppendRequest,
};

// store-proof-identity[bootstrap_catalog_admits_minimal_root_discovery_and_typed_read_access]: worth-store-layout-indexes::layout/bootstrap::bootstrap_catalog::bootstrap_catalog_admits_minimal_root_discovery_and_typed_read_access
#[test]
fn bootstrap_catalog_admits_minimal_root_discovery_and_typed_read_access() {
    let published = published_layout();
    let open = published
        .admit_bootstrap_open_witness()
        .expect("published layout should admit bootstrap open witness");
    let physical_catalog = physical_bootstrap_catalog()
        .discover_catalog(&open)
        .expect("physical bootstrap catalog should derive");

    let outcome = bootstrap_catalog().read_catalog(
        worth_store_layout_indexes::BootstrapOnlyAccessPath::fixed_bootstrap_access_path(),
        physical_catalog.clone(),
        physical_catalog.current_root(),
    );
    let counters = outcome.counters();
    let (catalog, admission) = outcome.expect("bootstrap facade should admit catalog");

    assert_eq!(
        catalog.root_reference(),
        worth_store_physical_format::PhysicalRootReference::from_raw(1).unwrap()
    );
    assert_eq!(
        catalog.discovery_layout().physical_format_version(),
        worth_store_physical_format::PhysicalFormatVersion::initial_format_version()
    );
    assert_eq!(
        catalog.discovery_layout().checksum_bytes_checked(),
        physical_catalog.checksum().bytes_checked()
    );
    assert_eq!(catalog.root_entry_count(), 4);
    assert_eq!(catalog.segment_count(), 1);
    assert_eq!(catalog.page_slot_count(), 1);
    assert_eq!(catalog.extent_count(), 1);
    assert_eq!(catalog.allocation_class_count(), 2);
    assert_eq!(catalog.free_space_count(), 0);
    assert_eq!(catalog.layout_entry_count(), 5);
    assert_eq!(counters.catalog_candidates_read(), 1);
    assert_eq!(
        counters.checksum_bytes_verified(),
        physical_catalog.checksum().bytes_checked()
    );
    assert_eq!(counters.root_entries_read(), 4);
    assert_eq!(counters.layout_entries_read(), 5);
    assert_eq!(counters.admitted_catalogs(), 1);

    assert_eq!(
        admission.physical_format_version(),
        catalog.physical_format_version()
    );
    assert_eq!(
        admission.root_owner(),
        physical_catalog.current_root().root_owner()
    );
}

// store-proof-identity[bootstrap_only_lane_denies_ordinary_family_access]: worth-store-layout-indexes::layout/bootstrap::bootstrap_catalog::bootstrap_only_lane_denies_ordinary_family_access
#[test]
fn bootstrap_only_lane_denies_ordinary_family_access() {
    let denial = bootstrap_catalog().deny_ordinary_family_access(
        layout_declarations()
            .declaration(worth_store_contracts::DurableArtifactFamilyId::PhysicalPage)
            .expect("physical page family must remain declared")
            .family(),
    );

    assert!(matches!(
        denial,
        worth_store_layout_indexes::BootstrapOnlyAccessDenied::OrdinaryFamilyAccessForbidden { .. }
    ));
}

// store-proof-identity[mismatched_current_root_readmission_is_rejected]: worth-store-layout-indexes::layout/bootstrap::bootstrap_catalog::mismatched_current_root_readmission_is_rejected
#[test]
fn mismatched_current_root_readmission_is_rejected() {
    let published = published_layout();
    let open = published
        .admit_bootstrap_open_witness()
        .expect("published layout should admit bootstrap open witness");
    let physical_catalog = physical_bootstrap_catalog()
        .discover_catalog(&open)
        .expect("physical bootstrap catalog should derive");
    let other_published = republished_layout();
    let other_open = other_published
        .admit_bootstrap_open_witness()
        .expect("other published layout should admit bootstrap open witness");
    let other_catalog = physical_bootstrap_catalog()
        .discover_catalog(&other_open)
        .expect("other physical bootstrap catalog should derive");

    let denial = bootstrap_catalog()
        .read_catalog(
            worth_store_layout_indexes::BootstrapOnlyAccessPath::fixed_bootstrap_access_path(),
            physical_catalog,
            other_catalog.current_root(),
        )
        .expect_err("stale or imported root admission cannot unlock ordinary bootstrap access");

    assert!(matches!(
        denial,
        worth_store_layout_indexes::BootstrapOnlyAccessDenied::CurrentRootReadmissionRequired { .. }
    ));
}

// store-proof-identity[bootstrap_admission_cannot_unlock_a_different_same_version_catalog]: worth-store-layout-indexes::layout/bootstrap::bootstrap_catalog::bootstrap_admission_cannot_unlock_a_different_same_version_catalog
#[test]
fn bootstrap_admission_cannot_unlock_a_different_same_version_catalog() {
    let published = published_layout();
    let open = published
        .admit_bootstrap_open_witness()
        .expect("first published layout should admit bootstrap open witness");
    let first_catalog = physical_bootstrap_catalog()
        .discover_catalog(&open)
        .expect("first physical bootstrap catalog should derive");
    let (_, first_admission) = bootstrap_catalog()
        .read_catalog(
            worth_store_layout_indexes::BootstrapOnlyAccessPath::fixed_bootstrap_access_path(),
            first_catalog,
            physical_bootstrap_catalog()
                .discover_catalog(&open)
                .expect("first bootstrap replay should derive")
                .current_root(),
        )
        .expect("first bootstrap facade should admit catalog");

    let other_published = republished_layout();
    let other_open = other_published
        .admit_bootstrap_open_witness()
        .expect("second published layout should admit bootstrap open witness");
    let other_catalog = physical_bootstrap_catalog()
        .discover_catalog(&other_open)
        .expect("second physical bootstrap catalog should derive");
    let (_, other_admission) = bootstrap_catalog()
        .read_catalog(
            worth_store_layout_indexes::BootstrapOnlyAccessPath::fixed_bootstrap_access_path(),
            other_catalog,
            physical_bootstrap_catalog()
                .discover_catalog(&other_open)
                .expect("second bootstrap replay should derive")
                .current_root(),
        )
        .expect("second bootstrap facade should admit catalog");
    assert_ne!(first_admission, other_admission);
}

// store-proof-identity[bootstrap_catalog_replays_stably_across_certification_replay]: worth-store-layout-indexes::layout/bootstrap::bootstrap_catalog::bootstrap_catalog_replays_stably_across_certification_replay
#[test]
fn bootstrap_catalog_replays_stably_across_certification_replay() {
    let published = published_layout();
    let published_physical_catalog = physical_bootstrap_catalog()
        .discover_catalog(
            &published
                .admit_bootstrap_open_witness()
                .expect("published layout should admit bootstrap open witness"),
        )
        .expect("physical bootstrap catalog should derive");
    let (published_catalog, published_admission) = bootstrap_catalog()
        .read_catalog(
            worth_store_layout_indexes::BootstrapOnlyAccessPath::fixed_bootstrap_access_path(),
            published_physical_catalog.clone(),
            published_physical_catalog.current_root(),
        )
        .expect("published bootstrap catalog should admit");
    let certification_materialization =
        worth_store_physical_certification::ProductionBackedFixtureMaterialization::from_replay_artifact(
            worth_store_physical_certification::LargeStoreFixtureProfile::CheckpointHeavy,
            published_physical_catalog.root_reference().get(),
            published.replay_artifact(),
        )
        .expect("published layout should materialize for certification replay");
    let fixture = worth_store_physical_certification::PhysicalFixtureBuilder::production_backed(
        "bootstrap-certification",
    )
    .materialize_with(certification_materialization)
    .capability(
        worth_store_physical_certification::FixtureCapabilityDeclaration::for_mutation_boundary(
            worth_store_physical_certification::FixtureMutationBoundary::Manifest,
        ),
    )
    .and_reopen_through_physical_authority()
    .expect("certification fixture should replay through physical authority");
    let certification_replay_artifact = fixture
        .replay_artifact()
        .expect("production-backed certification replay should preserve the native replay artifact")
        .clone();
    let reopened = worth_store_physical_format::InMemoryPhysicalFormatModel::restore(
        readiness(),
        worth_store_physical_format::InMemoryPhysicalFormatModelRequest::physical_format_canonical(
        ),
        certification_replay_artifact.clone(),
    )
    .expect("certification replay layout should reopen through physical authority");
    let certification_physical_catalog = physical_bootstrap_catalog()
        .discover_catalog(
            &certification_replay_artifact
                .admit_bootstrap_open_witness()
                .expect("certification replay should admit bootstrap witness"),
        )
        .expect("certification replay should derive canonical bootstrap catalog");
    let (certification_catalog, certification_admission) = bootstrap_catalog()
        .read_catalog(
            worth_store_layout_indexes::BootstrapOnlyAccessPath::fixed_bootstrap_access_path(),
            certification_physical_catalog.clone(),
            certification_physical_catalog.current_root(),
        )
        .expect("certification replay bootstrap catalog should admit");

    assert!(fixture
        .authority_receipt()
        .reopened_through_physical_authority());
    assert_eq!(fixture.manifest().source().root_reference(), 1);
    assert_eq!(
        published_physical_catalog.identity(),
        certification_physical_catalog.identity()
    );
    assert_eq!(published_catalog, certification_catalog);
    assert_eq!(published_admission, certification_admission);
    assert_eq!(reopened.counters().opens(), 1);
    assert_eq!(reopened.counters().restores(), 1);
}
