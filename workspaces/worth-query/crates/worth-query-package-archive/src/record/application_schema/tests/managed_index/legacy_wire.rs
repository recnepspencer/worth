//! Baseline full-member receipts captured before public-maintenance codec edits.
use super::*;

fn member(version: u16) -> ApplicationSchemaMember {
    let mut parts = definition().parts().clone();
    parts.semantic_version = version;
    if version >= 2 {
        parts.structure = ApplicationManagedIndexSourceStructure::BinarySourceTree;
    }
    if version >= 3 {
        parts.membership = ApplicationManagedIndexMembership::CheckedSummarySourceTree;
    }
    let definition = if version == 4 {
        ErasedApplicationManagedIndexDefinition::from_refinement_parts(
            parts,
            ApplicationManagedIndexRefinement::row_local("domain.exact-name.v1", 7).meaning(),
        )
    } else {
        ErasedApplicationManagedIndexDefinition::from_untrusted_parts(parts)
    };
    ApplicationSchemaMember::ManagedIndex { definition }
}

// Captured from production at 61b17083ed before any v5 meaning/codec edits.
const LEGACY: [&str; 4] = [
    include_str!("legacy_wire/v1.hex"),
    include_str!("legacy_wire/v2.hex"),
    include_str!("legacy_wire/v3.hex"),
    include_str!("legacy_wire/v4.hex"),
];

fn literal(hex: &str) -> Vec<u8> {
    let hex = hex.trim();
    assert_eq!(hex.len() % 2, 0);
    (0..hex.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&hex[offset..offset + 2], 16).unwrap())
        .collect()
}

#[test]
fn legacy_wire_preserves_v1_through_v4() {
    for version in 1..=4 {
        let member = member(version);
        let expected = literal(LEGACY[usize::from(version - 1)]);
        assert_eq!(
            encode_member(&member),
            expected,
            "v{version} encoder changed old bytes"
        );
        assert_eq!(
            decode_member(&expected),
            member,
            "v{version} decoder changed old meaning"
        );
    }
}
