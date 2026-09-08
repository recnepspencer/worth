mod comparison_protocol;
mod documented_commands;
mod journal_coverage;
mod journal_traversal;
mod phase_4_literal_vectors;
mod report_boundary_and_binary;
mod resource_bounds;
mod root_protocol_adversarial;
mod root_protocol_corruption_matrix;
mod root_protocol_expected_counters;
mod support;

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

use support::{
    clean_store, current_selector_bytes, namespace_identity_bytes, previous_selector_bytes,
    request, root_manifest_bytes,
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
    assert_eq!(root.len(), 368);
    assert_eq!(&current[44..48], &[0x4b, 0x44, 0xe9, 0x52]);
    assert_eq!(&previous[44..48], &[0x3c, 0xba, 0x79, 0xa7]);
    assert_eq!(&root[44..48], &[0xca, 0xbf, 0xb3, 0x37]);
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
                    PhysicalArtifactFamily::NamespaceIdentity
                        | PhysicalArtifactFamily::CurrentRootSelector
                        | PhysicalArtifactFamily::PreviousRootSelector
                        | PhysicalArtifactFamily::RootManifest
                )
            )
        })
        .collect();
    assert_eq!(root_artifacts.len(), 4);
    assert!(root_artifacts
        .iter()
        .all(|artifact| artifact.outcome().posture() == PhysicalIntegrityPosture::Intact));
    assert_eq!(
        root_artifacts[0].family(),
        PhysicalArtifactFamily::CurrentRootSelector
    );
    assert_eq!(
        root_artifacts[1].family(),
        PhysicalArtifactFamily::PreviousRootSelector
    );
    assert_eq!(
        root_artifacts[2].family(),
        PhysicalArtifactFamily::RootManifest
    );
    let counters = report.counters();
    assert_eq!(counters.entries_visited(), 8);
    assert_eq!(counters.missing_artifacts(), 3);
    assert_eq!(counters.bytes_read(), 654);
    assert_eq!(counters.files_opened(), 16);
    assert_eq!(counters.open_file_high_water(), 5);
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
            OfflineIntegrityObservationLimits::new(0, 1, 1, 1, 0, 1, 1),
            OfflineIntegrityObservationLimitsDenial::ZeroEntries,
        ),
        (
            OfflineIntegrityObservationLimits::new(1, 0, 1, 1, 0, 1, 1),
            OfflineIntegrityObservationLimitsDenial::ZeroBytes,
        ),
        (
            OfflineIntegrityObservationLimits::new(1, 1, 0, 1, 0, 1, 1),
            OfflineIntegrityObservationLimitsDenial::ZeroOpenFiles,
        ),
        (
            OfflineIntegrityObservationLimits::new(1, 1, 1, 0, 0, 1, 1),
            OfflineIntegrityObservationLimitsDenial::ZeroDepth,
        ),
        (
            OfflineIntegrityObservationLimits::new(1, 1, 1, 1, 0, 0, 1),
            OfflineIntegrityObservationLimitsDenial::ZeroElapsedMilliseconds,
        ),
        (
            OfflineIntegrityObservationLimits::new(1, 1, 1, 1, 0, 1, 0),
            OfflineIntegrityObservationLimitsDenial::ZeroReportBytes,
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
            PhysicalIntegrityArtifactFamily::CurrentRootSelector,
        ),
        (
            declarations.previous_selector(),
            PhysicalIntegrityArtifactFamily::PreviousRootSelector,
        ),
        (
            declarations.root_manifest(),
            PhysicalIntegrityArtifactFamily::RootManifest,
        ),
    ] {
        assert_eq!(declaration.family(), family);
        assert_eq!(declaration.version().format_version(), 1);
        assert_eq!(declaration.version().envelope_schema(), Some(2));
        assert_eq!(declaration.checksums().len(), 1);
        let checksum = declaration.checksums()[0];
        assert_eq!(checksum.algorithm(), PhysicalIntegrityAlgorithm::Crc32c);
        assert_eq!(checksum.covered_ranges().len(), 2);
        assert_eq!(
            (
                checksum.covered_ranges()[0].start(),
                checksum.covered_ranges()[0].end(),
                checksum.covered_ranges()[1].start(),
                checksum.covered_ranges()[1].end(),
            ),
            (
                PhysicalIntegrityCoverageBoundary::Fixed(0),
                PhysicalIntegrityCoverageBoundary::Fixed(44),
                PhysicalIntegrityCoverageBoundary::Fixed(48),
                PhysicalIntegrityCoverageBoundary::ArtifactEnd,
            )
        );
        assert_eq!(
            (checksum.field().start(), checksum.field().end()),
            (
                PhysicalIntegrityCoverageBoundary::Fixed(44),
                PhysicalIntegrityCoverageBoundary::Fixed(48),
            )
        );
    }
}

#[test]
fn namespace_literal_reader_matches_declaration_facts() {
    let declaration = NAMESPACE_IDENTITY_INTEGRITY_DECLARATION;
    assert_eq!(
        declaration.family(),
        PhysicalIntegrityArtifactFamily::NamespaceIdentity
    );
    assert_eq!(declaration.version().format_version(), 1);
    assert_eq!(declaration.checksums().len(), 1);
    let checksum = declaration.checksums()[0];
    assert_eq!(checksum.algorithm(), PhysicalIntegrityAlgorithm::Sha256);
    assert_eq!(checksum.covered_ranges().len(), 1);
    assert_eq!(
        (
            checksum.covered_ranges()[0].start(),
            checksum.covered_ranges()[0].end()
        ),
        (
            PhysicalIntegrityCoverageBoundary::Fixed(0),
            PhysicalIntegrityCoverageBoundary::Fixed(40)
        )
    );
    assert_eq!(
        (checksum.field().start(), checksum.field().end()),
        (
            PhysicalIntegrityCoverageBoundary::Fixed(40),
            PhysicalIntegrityCoverageBoundary::Fixed(72)
        )
    );
}
