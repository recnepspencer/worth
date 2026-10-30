use worth_foundational::{PhysicalArtifactFamily, PhysicalIntegrityPosture};
use worth_store_offline_integrity_observer::{
    encode_offline_integrity_report, observe_store, OfflineIntegrityObservationLimits,
    OfflineIntegrityObservationLimitsDenial, OFFLINE_INTEGRITY_ROOT_PROTOCOL_DECLARATIONS,
    PHYSICAL_INTEGRITY_OBSERVATION_PROTOCOL_IDENTITY,
    PHYSICAL_INTEGRITY_OBSERVATION_PROTOCOL_VERSION,
};
use worth_store_physical_format::integrity_declarations::families::NAMESPACE_IDENTITY_INTEGRITY_DECLARATION;
use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::integrity_declarations::{
    PhysicalIntegrityAlgorithm, PhysicalIntegrityCoverageBoundary,
};

use crate::{
    store_offline_integrity_observer_support::request, support::fixtures::clean_store,
    support::fixtures::current_selector_bytes, support::fixtures::namespace_identity_bytes,
    support::fixtures::previous_selector_bytes, support::fixtures::root_manifest_bytes,
};

#[test]
fn literal_root_protocol_goldens_are_fixed() {
    let namespace = namespace_identity_bytes();
    let current = current_selector_bytes();
    let previous = previous_selector_bytes();
    let root = root_manifest_bytes();
    assert_eq!(namespace.len(), 72);
    assert_eq!(
        &namespace[40..72],
        &[
            0xd7, 0xbf, 0x45, 0x2a, 0x49, 0x16, 0xb2, 0x9c, 0xdc, 0x46, 0x0a, 0x8a, 0x54, 0xa3,
            0xe8, 0x6f, 0xbe, 0x47, 0xfa, 0x87, 0xb5, 0xda, 0x5f, 0x1c, 0x93, 0x3f, 0x26, 0xfd,
            0x6a, 0x66, 0x7e, 0x93,
        ]
    );
    assert_eq!(current.len(), 107);
    assert_eq!(previous.len(), 107);
    assert_eq!(root.len(), 384);
    assert_eq!(&current[44..48], &[0x84, 0x20, 0x6a, 0x5d]);
    assert_eq!(&previous[44..48], &[0xf3, 0xde, 0xfa, 0xa8]);
    assert_eq!(&root[44..48], &[0x6e, 0xd4, 0x20, 0xc3]);
}

#[test]
fn clean_root_protocol_observation_and_counters_are_exact() {
    let fixture = clean_store("literal-clean");
    let report = observe_store(&request(&fixture)).unwrap();
    assert_eq!(
        report.store_identity(),
        Some("0102030405060708090a0b0c0d0e0f10")
    );
    assert_eq!(report.artifacts().len(), 7);
    let root_artifacts: Vec<_> = report
        .artifacts()
        .iter()
        .filter(|artifact| {
            matches!(
                artifact.family().declared(),
                Some(
                    worth_foundational::PhysicalArtifactFamily::NamespaceIdentity
                        | worth_foundational::PhysicalArtifactFamily::CurrentRootSelector
                        | worth_foundational::PhysicalArtifactFamily::PreviousRootSelector
                        | worth_foundational::PhysicalArtifactFamily::RootManifest
                )
            )
        })
        .collect();
    assert_eq!(root_artifacts.len(), 4);
    assert!(root_artifacts
        .iter()
        .all(|artifact| artifact.outcome().posture()
            == worth_foundational::PhysicalIntegrityPosture::Intact));
    assert_eq!(
        root_artifacts[0].family(),
        worth_foundational::PhysicalArtifactFamily::CurrentRootSelector
    );
    assert_eq!(
        root_artifacts[1].family(),
        worth_foundational::PhysicalArtifactFamily::PreviousRootSelector
    );
    assert_eq!(
        root_artifacts[2].family(),
        worth_foundational::PhysicalArtifactFamily::RootManifest
    );
    let counters = report.counters();
    assert_eq!(counters.entries_visited(), 8);
    assert_eq!(counters.missing_artifacts(), 3);
    assert_eq!(counters.bytes_read(), 670);
    assert_eq!(counters.files_opened(), if cfg!(windows) { 28 } else { 16 });
    assert_eq!(
        counters.open_file_high_water(),
        if cfg!(windows) { 6 } else { 5 }
    );
    assert_eq!(counters.maximum_depth_reached(), 4);
    assert_eq!(counters.checksum_calculations(), 4);
    assert_eq!(counters.namespace_identity_payload_decoder_entries(), 1);
    assert_eq!(counters.checksum_validated_durable_frames(), 3);
    assert_eq!(counters.selector_payload_decoder_entries(), 2);
    assert_eq!(counters.root_manifest_payload_decoder_entries(), 1);
    assert_eq!(counters.unsupported_versions(), 0);
    assert_eq!(counters.exhausted_bounds(), 0);
    let wire = encode_offline_integrity_report(&report).unwrap();
    assert_eq!(counters.report_bytes(), wire.len() as u64);
    assert!(
        wire.starts_with("{\"protocol\":\"store.physical.integrity-observation\",\"version\":1")
    );
    assert!(wire.contains("\"role\":\"offline-root-observer\""));
    for version_one_counter in [
        "namespace_identity_decoders",
        "durable_frame_decoders",
        "selector_decoders",
        "root_manifest_decoders",
    ] {
        assert!(wire.contains(&format!("\"{version_one_counter}\":")));
    }
    assert!(!wire.contains("namespace_identity_payload_decoder_entries"));
    assert!(!wire.contains("checksum_validated_durable_frames"));
    assert!(!wire.contains("selector_payload_decoder_entries"));
    assert!(!wire.contains("root_manifest_payload_decoder_entries"));
    assert!(!wire.contains("admission"));
    assert!(!wire.contains("recovery_option"));
}

#[test]
fn declared_limits_and_protocol_identity_are_typed_and_bounded() {
    let cases = [
        (
            worth_store_offline_integrity_observer::OfflineIntegrityObservationLimits::new(0, 1, 1, 1, 0, 1, 1),
            worth_store_offline_integrity_observer::OfflineIntegrityObservationLimitsDenial::ZeroEntries,
        ),
        (
            worth_store_offline_integrity_observer::OfflineIntegrityObservationLimits::new(1, 0, 1, 1, 0, 1, 1),
            worth_store_offline_integrity_observer::OfflineIntegrityObservationLimitsDenial::ZeroBytes,
        ),
        (
            worth_store_offline_integrity_observer::OfflineIntegrityObservationLimits::new(1, 1, 0, 1, 0, 1, 1),
            worth_store_offline_integrity_observer::OfflineIntegrityObservationLimitsDenial::ZeroOpenFiles,
        ),
        (
            worth_store_offline_integrity_observer::OfflineIntegrityObservationLimits::new(1, 1, 1, 0, 0, 1, 1),
            worth_store_offline_integrity_observer::OfflineIntegrityObservationLimitsDenial::ZeroDepth,
        ),
        (
            worth_store_offline_integrity_observer::OfflineIntegrityObservationLimits::new(1, 1, 1, 1, 0, 0, 1),
            worth_store_offline_integrity_observer::OfflineIntegrityObservationLimitsDenial::ZeroElapsedMilliseconds,
        ),
        (
            worth_store_offline_integrity_observer::OfflineIntegrityObservationLimits::new(1, 1, 1, 1, 0, 1, 0),
            worth_store_offline_integrity_observer::OfflineIntegrityObservationLimitsDenial::ZeroReportBytes,
        ),
    ];
    for (result, expected) in cases {
        assert_eq!(result.unwrap_err(), expected);
    }
    assert_eq!(
        PHYSICAL_INTEGRITY_OBSERVATION_PROTOCOL_IDENTITY.as_str(),
        "store.physical.integrity-observation"
    );
    assert_eq!(PHYSICAL_INTEGRITY_OBSERVATION_PROTOCOL_VERSION.get(), 1);
}

#[test]
fn root_slice_imports_only_declaration_facade_facts() {
    let declarations = OFFLINE_INTEGRITY_ROOT_PROTOCOL_DECLARATIONS;
    for (declaration, family) in [
        (
            declarations.current_selector(),
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CurrentRootSelector,
        ),
        (
            declarations.previous_selector(),
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::PreviousRootSelector,
        ),
        (
            declarations.root_manifest(),
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::RootManifest,
        ),
    ] {
        assert_eq!(declaration.family(), family);
        assert_eq!(declaration.version().format_version(), 2);
        assert_eq!(declaration.version().envelope_schema(), Some(2));
        assert_eq!(declaration.checksums().len(), 1);
        let checksum = declaration.checksums()[0];
        assert_eq!(checksum.algorithm(), worth_store_physical_format::integrity_declarations::PhysicalIntegrityAlgorithm::Crc32c);
        assert_eq!(checksum.covered_ranges().len(), 2);
        assert_eq!(
            (
                checksum.covered_ranges()[0].start(),
                checksum.covered_ranges()[0].end(),
                checksum.covered_ranges()[1].start(),
                checksum.covered_ranges()[1].end(),
            ),
            (
                worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary::Fixed(0),
                worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary::Fixed(44),
                worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary::Fixed(48),
                worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary::ArtifactEnd,
            )
        );
        assert_eq!(
            (checksum.field().start(), checksum.field().end()),
            (
                worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary::Fixed(44),
                worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary::Fixed(48),
            )
        );
    }
}

#[test]
fn namespace_literal_reader_matches_declaration_facts() {
    let declaration = NAMESPACE_IDENTITY_INTEGRITY_DECLARATION;
    assert_eq!(
        declaration.family(),
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::NamespaceIdentity
    );
    assert_eq!(declaration.version().format_version(), 1);
    assert_eq!(declaration.checksums().len(), 1);
    let checksum = declaration.checksums()[0];
    assert_eq!(
        checksum.algorithm(),
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityAlgorithm::Sha256
    );
    assert_eq!(checksum.covered_ranges().len(), 1);
    assert_eq!(
        (
            checksum.covered_ranges()[0].start(),
            checksum.covered_ranges()[0].end()
        ),
        (
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary::Fixed(0),
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary::Fixed(40)
        )
    );
    assert_eq!(
        (checksum.field().start(), checksum.field().end()),
        (
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary::Fixed(40),
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary::Fixed(72)
        )
    );
}

#[path = "comparison_protocol/mod.rs"]
pub(crate) mod comparison_protocol;
#[path = "journal_coverage/mod.rs"]
pub(crate) mod journal_coverage;
#[path = "journal_traversal/mod.rs"]
pub(crate) mod journal_traversal;
#[path = "report_boundary_and_binary/mod.rs"]
pub(crate) mod report_boundary_and_binary;
#[path = "resource_bounds/mod.rs"]
pub(crate) mod resource_bounds;
#[path = "root_protocol_adversarial/mod.rs"]
pub(crate) mod root_protocol_adversarial;
#[path = "root_protocol_corruption_matrix/mod.rs"]
pub(crate) mod root_protocol_corruption_matrix;
#[path = "selected_comparison_protocol/mod.rs"]
pub(crate) mod selected_comparison_protocol;
#[path = "support/mod.rs"]
pub(crate) mod support;
