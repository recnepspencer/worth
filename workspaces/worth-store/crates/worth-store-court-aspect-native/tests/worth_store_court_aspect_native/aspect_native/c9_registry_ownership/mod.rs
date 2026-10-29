use std::collections::HashSet;

use worth_store_aspect_native::{
    canonical_basis_source_owner_for_family, certify_canonical_basis_field_role,
    certify_canonical_basis_source, StoreCanonicalBasisFamily as Family,
    StoreCanonicalBasisFieldRole as Field, StoreCanonicalBasisLane as Lane,
    StoreCanonicalBasisSourceDenial as Denial, StoreCanonicalBasisSourceKind as Source,
    STORE_CANONICAL_BASIS_SOURCE_OWNERS,
};

#[test]
fn sources_follow_declaration_observation_and_runtime_owners() {
    let cases = [
        (
            worth_store_aspect_native::StoreCanonicalBasisFamily::PhysicalArtifactIdentity,
            "worth-foundational",
            worth_store_aspect_native::StoreCanonicalBasisSourceKind::FoundationalPhysicalArtifactIdentity,
            worth_store_aspect_native::StoreCanonicalBasisLane::PhysicalIntegrity,
        ),
        (
            worth_store_aspect_native::StoreCanonicalBasisFamily::PhysicalAdapterEvidence,
            "worth-foundational",
            worth_store_aspect_native::StoreCanonicalBasisSourceKind::FoundationalPhysicalAdapterEvidence,
            worth_store_aspect_native::StoreCanonicalBasisLane::PhysicalIntegrity,
        ),
        (
            worth_store_aspect_native::StoreCanonicalBasisFamily::PhysicalIntegrityChecksumCoverage,
            "worth-store-physical-format",
            worth_store_aspect_native::StoreCanonicalBasisSourceKind::StorePhysicalFormatDeclaration,
            worth_store_aspect_native::StoreCanonicalBasisLane::PhysicalRecord,
        ),
        (
            worth_store_aspect_native::StoreCanonicalBasisFamily::PhysicalQuarantineObservation,
            "worth-store",
            worth_store_aspect_native::StoreCanonicalBasisSourceKind::StorePhysicalQuarantineObservation,
            worth_store_aspect_native::StoreCanonicalBasisLane::PhysicalIntegrity,
        ),
        (
            worth_store_aspect_native::StoreCanonicalBasisFamily::PhysicalIntegrityScrubProgress,
            "worth-store",
            worth_store_aspect_native::StoreCanonicalBasisSourceKind::StorePhysicalIntegrityScrubProgress,
            worth_store_aspect_native::StoreCanonicalBasisLane::PhysicalIntegrity,
        ),
    ];
    for &(family, owner_crate, source, lane) in &cases {
        let owner = canonical_basis_source_owner_for_family(family).unwrap();
        assert_eq!(owner.owner_crate(), owner_crate);
        assert_eq!(owner.foundational_lane(), lane);
        assert_eq!(owner.primary_source_kind(), Some(source));
        assert_eq!(certify_canonical_basis_source(family, source), Ok(()));

        // A valid source in another C9 family cannot impersonate this one.
        for &(_, _, other_source, _) in &cases {
            if source != other_source {
                assert_eq!(
                    certify_canonical_basis_source(family, other_source),
                    Err(worth_store_aspect_native::StoreCanonicalBasisSourceDenial::WrongNativeSourceKind {
                        family,
                        source: other_source,
                    })
                );
            }
        }
        assert!(certify_canonical_basis_source(
            family,
            worth_store_aspect_native::StoreCanonicalBasisSourceKind::StoreReadinessHandoff
        )
        .is_err());
        assert!(certify_canonical_basis_source(
            family,
            worth_store_aspect_native::StoreCanonicalBasisSourceKind::FoundationalReceipt
        )
        .is_err());
    }
}

#[test]
fn registry_has_one_owner_per_live_family_and_rejects_projection_inputs() {
    let families: HashSet<_> = worth_store_aspect_native::StoreCanonicalBasisFamily::ALL
        .into_iter()
        .collect();
    let owners: HashSet<_> = STORE_CANONICAL_BASIS_SOURCE_OWNERS
        .iter()
        .map(|owner| owner.family())
        .collect();
    assert_eq!(
        families.len(),
        worth_store_aspect_native::StoreCanonicalBasisFamily::ALL.len()
    );
    assert_eq!(owners.len(), STORE_CANONICAL_BASIS_SOURCE_OWNERS.len());
    assert_eq!(owners, families);
    for family in worth_store_aspect_native::StoreCanonicalBasisFamily::ALL {
        let owner = canonical_basis_source_owner_for_family(family).unwrap();
        assert!(owner.primary_source_kind().is_some());
        assert_ne!(owner.owner_crate(), "worth-store-physical-integrity");
        for field in [
            worth_store_aspect_native::StoreCanonicalBasisFieldRole::TerminalProjection,
            worth_store_aspect_native::StoreCanonicalBasisFieldRole::OperatorDisplay,
            worth_store_aspect_native::StoreCanonicalBasisFieldRole::DocumentChecksum,
            worth_store_aspect_native::StoreCanonicalBasisFieldRole::CompatibilityText,
            worth_store_aspect_native::StoreCanonicalBasisFieldRole::DigestText,
            worth_store_aspect_native::StoreCanonicalBasisFieldRole::RawJsonPayload,
        ] {
            assert!(owner.denies_field(field));
            assert_eq!(
                certify_canonical_basis_field_role(field),
                Err(worth_store_aspect_native::StoreCanonicalBasisSourceDenial::ForbiddenFieldRole { field_role: field })
            );
        }
    }
}

#[test]
fn retired_generic_integrity_routes_have_no_enum_domain_or_registry_alias() {
    let sources = [
        include_str!("../../../../../worth-store-aspect-native/src/canonical_basis.rs"),
        include_str!("../../../../../worth-store-aspect-native/src/canonical_basis/canonical_basis_sources.rs"),
        include_str!("../../../../../worth-store-aspect-native/src/canonical_basis/canonical_basis_domains.rs"),
    ];
    for retired in [
        "IntegrityCloseoutHandoff",
        "PhysicalIntegrityEvidence",
        "PhysicalIntegrityCloseoutEvidence",
        "StorePhysicalIntegrityEvidence",
        "PhysicalIdentityEvidence",
        "PhysicalFoundationEvidence",
        "PhysicalIntegrityQuarantineReceipt",
        "PhysicalIntegrityScrubReceipt",
        "store.new.integrity.closeout.handoff",
        "store.physical.integrity.evidence",
        "store.physical.integrity.closeout.evidence",
    ] {
        for source in sources {
            assert!(
                !source.contains(retired),
                "retired registry route: {retired}"
            );
        }
    }
}
