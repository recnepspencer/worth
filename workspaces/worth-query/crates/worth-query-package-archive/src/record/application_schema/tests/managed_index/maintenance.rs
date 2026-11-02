//! Explicit maintenance survives transport; absent legacy metadata stays absent.
use super::*;
use ApplicationManagedIndexColdSource as Cold;
use ApplicationManagedIndexMaintenance as Maintenance;

fn parts() -> WorthQueryPortableManagedIndexParts {
    let mut parts = definition().parts().clone();
    parts.semantic_version = 5;
    parts.structure = ApplicationManagedIndexSourceStructure::BinarySourceTree;
    parts.membership = ApplicationManagedIndexMembership::CheckedSummarySourceTree;
    parts
}

#[test]
fn v5_round_trips_explicit_modes_optional_refinement_and_exact_legacy_fences() {
    let refinement =
        ApplicationManagedIndexRefinement::row_local("domain.exact-name.v1", 7).meaning();
    for maintenance in [
        Maintenance::WhenRequired,
        Maintenance::Native { cold: Cold::Refuse },
        Maintenance::Native {
            cold: Cold::AdmitRebuild,
        },
    ] {
        for refinement in [None, Some(refinement.clone())] {
            let definition = ErasedApplicationManagedIndexDefinition::from_maintenance_parts(
                parts(),
                maintenance,
                refinement.clone(),
            );
            assert_eq!(definition.validate(), Ok(()));
            assert_eq!(definition.maintenance(), maintenance);
            let member = ApplicationSchemaMember::ManagedIndex { definition };
            let bytes = encode_member(&member);
            assert_eq!(decode_member(&bytes), member);
            assert_eq!(encode_member(&decode_member(&bytes)), bytes);
            assert_eq!(
                try_decode_member(&bytes[..bytes.len() - 1])
                    .unwrap_err()
                    .kind(),
                Kind::Truncated
            );
        }
    }
    assert_eq!(
        ErasedApplicationManagedIndexDefinition::from_untrusted_parts(parts()).validate(),
        Err(ApplicationManagedIndexDefinitionDenial::UnsupportedMaintenance)
    );
    assert_eq!(
        ErasedApplicationManagedIndexDefinition::from_refinement_parts(parts(), refinement.clone())
            .validate(),
        Err(ApplicationManagedIndexDefinitionDenial::UnsupportedMaintenance)
    );
    for version in 1..=4 {
        let mut parts = parts();
        parts.semantic_version = version;
        // Use a legal legacy structure/membership so absence is the decisive fence.
        if version == 1 {
            parts.structure = ApplicationManagedIndexSourceStructure::Opaque;
        }
        if version < 3 {
            parts.membership = ApplicationManagedIndexMembership::Candidates;
        }
        assert_eq!(
            ErasedApplicationManagedIndexDefinition::from_maintenance_parts(
                parts,
                Maintenance::WhenRequired,
                (version == 4).then(|| refinement.clone())
            )
            .validate(),
            Err(ApplicationManagedIndexDefinitionDenial::UnsupportedMaintenance)
        );
    }
    let mut candidate = parts();
    candidate.membership = ApplicationManagedIndexMembership::Candidates;
    assert_eq!(
        ErasedApplicationManagedIndexDefinition::from_maintenance_parts(
            candidate,
            Maintenance::Native {
                cold: Cold::AdmitRebuild
            },
            Some(refinement)
        )
        .validate(),
        Err(ApplicationManagedIndexDefinitionDenial::UnsupportedRefinement)
    );
}

#[test]
fn v5_unknown_or_absent_mode_cold_and_refinement_tags_refuse_without_payload_inference() {
    // Independent literal header: member48/version5/name I/provider P/version1/tree1/summary2.
    let prefix = [
        0, 48, 0, 5, 0, 0, 0, 1, b'I', 0, 0, 0, 1, b'P', 0, 0, 0, 1, 0, 1, 0, 2,
    ];
    assert_eq!(
        try_decode_member(&prefix).unwrap_err().kind(),
        Kind::Truncated
    );
    assert_eq!(
        try_decode_member(&[prefix.as_slice(), &[0, 2]].concat())
            .unwrap_err()
            .kind(),
        Kind::UnsupportedRecordVariant
    );
    assert_eq!(
        try_decode_member(&[prefix.as_slice(), &[0, 1]].concat())
            .unwrap_err()
            .kind(),
        Kind::Truncated
    );
    assert_eq!(
        try_decode_member(&[prefix.as_slice(), &[0, 1, 0, 2]].concat())
            .unwrap_err()
            .kind(),
        Kind::UnsupportedRecordVariant
    );
    let member = ApplicationSchemaMember::ManagedIndex {
        definition: ErasedApplicationManagedIndexDefinition::from_maintenance_parts(
            parts(),
            Maintenance::Native { cold: Cold::Refuse },
            None,
        ),
    };
    let mut bytes = encode_member(&member);
    assert_eq!(&bytes[bytes.len() - 2..], &[0, 0]);
    let last = bytes.len() - 1;
    bytes[last] = 2;
    assert_eq!(
        try_decode_member(&bytes).unwrap_err().kind(),
        Kind::UnsupportedRecordVariant
    );
}
