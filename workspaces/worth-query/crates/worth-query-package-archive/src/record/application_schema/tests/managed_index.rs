//! Versioned provider meaning never serializes an executable provider or root.
use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
mod legacy_wire;
mod maintenance;
mod managed_source;
mod refinement;
use super::{decode_member, encode_member, try_decode_member};
use crate::facade::WorthQueryPackageArchiveDenialKind as Kind;
use worth_foundational::facade::ScalarAspectType;
use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity as Identity;
use worth_query_declaration::facade::{application_query::*, application_schema::*};

#[derive(worth_foundational::facade::WorthValue)]
pub struct Probe {
    text: String,
}
worth_query_declaration::worth_query_value_binding!(pub ProbeBinding for Probe {
    identity: "worth.query.tests.archive.index-probe.v1", scalar: String,
    canonical: ValueDecodeBounds::new(1024, 512, 4, 4),
});

struct BinaryProbeDelegate;
pub struct BinaryProbe(u8);
impl BinaryValueDelegate for BinaryProbeDelegate {
    type Value = BinaryProbe;
    const MAXIMUM_WIRE_BYTES: usize = 1;
    fn encode(
        value: &BinaryProbe,
        writer: &mut dyn BinaryValueWriter,
    ) -> Result<(), ValueCodecDenial> {
        writer.write(&[value.0])
    }
    fn admit(
        bytes: &[u8],
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<BinaryProbe, ValueCodecDenial> {
        admission.admit(1, 0)?;
        let [byte] = bytes else {
            return Err(ValueCodecDenial::Malformed);
        };
        Ok(BinaryProbe(*byte))
    }
}
worth_query_declaration::worth_query_value_binding!(pub BinaryProbeBinding for BinaryProbe {
    identity: "worth.query.tests.archive.binary-index-probe.v1",
    binary: BinaryProbeDelegate,
});

pub(super) fn definition() -> ErasedApplicationManagedIndexDefinition {
    ErasedApplicationManagedIndexDefinition::from_untrusted_parts(
        ARCHIVE_DECODE,
        WorthQueryPortableManagedIndexParts {
            semantic_version: 1,
            name: "Prefix".into(),
            provider: "worth.tests.prefix.v1".into(),
            provider_version: 1,
            structure: ApplicationManagedIndexSourceStructure::Opaque,
            membership: ApplicationManagedIndexMembership::Candidates,
            source: ApplicationManagedIndexSourceMeaning::same_scope_without_parameters(
                ApplicationQueryBindingReference::from_untrusted_parts(
                    ARCHIVE_DECODE,
                    ApplicationQueryBindingReferenceParts {
                        binding: Identity::declared("worth.tests.source-binding.v1"),
                        input: Identity::declared("worth.tests.source-input.v1"),
                        query_name: "Source".into(),
                        query: Identity::declared("worth.tests.source-query.v1"),
                        parameters: Identity::declared("worth.rust.unit"),
                        result: Identity::declared("worth.tests.source-row.v1"),
                        scope: Identity::declared("worth.tests.scope.v1"),
                        scope_entity: "Document".into(),
                        scope_aspect: "Identity".into(),
                        scope_field: "Id".into(),
                        scope_value: Identity::declared("worth.rust.u64"),
                        scope_resolution: ApplicationQueryScopeResolutionMode::PrincipalIdentity,
                        principal_binding: "Principal".into(),
                    },
                ),
            ),
            entity: "Document".into(),
            probe: ApplicationManagedIndexProbeMeaning::of::<ProbeBinding>(),
        },
    )
}

#[test]
fn ordered_predecessor_scope_round_trips_as_distinct_provider_meaning() {
    let mut parts = definition().parts().clone();
    parts.source.scope = ApplicationManagedIndexSourceScope::OrderedPredecessor;
    let ordered = ApplicationSchemaMember::ManagedIndex {
        definition: ErasedApplicationManagedIndexDefinition::from_untrusted_parts(
            ARCHIVE_DECODE,
            parts,
        ),
    };
    let same = ApplicationSchemaMember::ManagedIndex {
        definition: definition(),
    };
    assert_ne!(encode_member(&ordered), encode_member(&same));
    assert_eq!(decode_member(&encode_member(&ordered)), ordered);
}

#[test]
fn invariant_source_policy_revision_is_explicit_and_refuses_unversioned_or_future_bytes() {
    let member = ApplicationSchemaMember::InvariantManagedIndexSource {
        invariant: "UniqueNames".into(),
        execution_point: ApplicationInvariantExecutionPoint::CommitBoundary,
        requirement: ApplicationInvariantManagedIndexSource::from_untrusted_parts(
            ARCHIVE_DECODE,
            definition(),
            ApplicationManagedIndexColdSource::AdmitRebuild,
        ),
    };
    let bytes = encode_member(&member);
    assert_eq!(&bytes[..4], &[0, 49, 0, 1]);
    let restored = decode_member(&bytes);
    assert_eq!(restored, member);
    assert_eq!(encode_member(&restored), bytes);
    // Unknown policy refuses before any definition/projection payload is read.
    assert_eq!(
        try_decode_member(&[0, 49, 0, 2]).unwrap_err().kind(),
        Kind::UnsupportedRecordVariant
    );
    assert_eq!(
        try_decode_member(&[0, 49]).unwrap_err().kind(),
        Kind::Truncated
    );
    // The pre-policy WIP layout cannot silently acquire revision1 meaning.
    let unversioned = [&bytes[..2], &bytes[4..]].concat();
    assert_eq!(
        try_decode_member(&unversioned).unwrap_err().kind(),
        Kind::UnsupportedRecordVariant
    );
}

#[test]
fn managed_index_v1_round_trips_full_probe_and_rejects_future_version_before_payload() {
    let read = ApplicationSchemaMember::OperationDecisionRead {
        operation: "Rename".into(),
        target: ApplicationOperationDecisionReadTarget::ManagedIndex {
            parent: definition().parts().source.binding.clone(),
        },
    };
    assert_eq!(decode_member(&encode_member(&read)), read);
    let member = ApplicationSchemaMember::ManagedIndex {
        definition: definition(),
    };
    let bytes = encode_member(&member);
    assert_eq!(&bytes[..4], &[0, 48, 0, 1]);
    assert_eq!(decode_member(&bytes), member);
    // No payload follows: version refusal must precede any source/profile read.
    assert_eq!(
        try_decode_member(&[0, 48, 0, 6]).unwrap_err().kind(),
        Kind::UnsupportedRecordVariant
    );
    let mut parts = definition().parts().clone();
    parts.probe = ApplicationManagedIndexProbeMeaning::of::<StringApplicationValueBinding>();
    let native = ApplicationSchemaMember::ManagedIndex {
        definition: ErasedApplicationManagedIndexDefinition::from_untrusted_parts(
            ARCHIVE_DECODE,
            parts.clone(),
        ),
    };
    assert_eq!(decode_member(&encode_member(&native)), native);
    parts.probe = ApplicationManagedIndexProbeMeaning::of::<BinaryProbeBinding>();
    let binary = ApplicationSchemaMember::ManagedIndex {
        definition: ErasedApplicationManagedIndexDefinition::from_untrusted_parts(
            ARCHIVE_DECODE,
            parts.clone(),
        ),
    };
    assert_eq!(decode_member(&encode_member(&binary)), binary);
    let mut unfunded = parts.probe.parts().clone();
    unfunded.supports_admitted_decode = false;
    parts.probe =
        ApplicationManagedIndexProbeMeaning::from_untrusted_parts(ARCHIVE_DECODE, unfunded);
    assert_eq!(
        ErasedApplicationManagedIndexDefinition::from_untrusted_parts(ARCHIVE_DECODE, parts)
            .validate(),
        Err(ApplicationManagedIndexDefinitionDenial::UnsupportedProbeProfile)
    );
    // The old physical entity grammar remains an independent literal byte oracle.
    let entity = ApplicationSchemaMember::Entity {
        entity: "Document".into(),
    };
    assert_eq!(
        encode_member(&entity),
        [0, 1, 0, 0, 0, 8, b'D', b'o', b'c', b'u', b'm', b'e', b'n', b't']
    );
}

#[test]
fn managed_index_structure_is_versioned_and_old_bytes_cannot_claim_source_coverage() {
    let mut parts = definition().parts().clone();
    parts.structure = ApplicationManagedIndexSourceStructure::BinarySourceTree;
    assert_eq!(
        ErasedApplicationManagedIndexDefinition::from_untrusted_parts(
            ARCHIVE_DECODE,
            parts.clone()
        )
        .validate(),
        Err(ApplicationManagedIndexDefinitionDenial::UnsupportedSourceStructure)
    );
    parts.semantic_version = 2;
    let member = ApplicationSchemaMember::ManagedIndex {
        definition: ErasedApplicationManagedIndexDefinition::from_untrusted_parts(
            ARCHIVE_DECODE,
            parts,
        ),
    };
    let bytes = encode_member(&member);
    assert_eq!(&bytes[..4], &[0, 48, 0, 2]);
    let restored = decode_member(&bytes);
    assert_eq!(restored, member);
    assert_eq!(encode_member(&restored), bytes);
    // Literal v2 prefix: one-byte name/provider, version1, unsupported structure2.
    // No Source follows; the structural profile must refuse before reading it.
    assert_eq!(
        try_decode_member(&[0, 48, 0, 2, 0, 0, 0, 1, b'I', 0, 0, 0, 1, b'P', 0, 0, 0, 1, 0, 3])
            .unwrap_err()
            .kind(),
        Kind::UnsupportedRecordVariant
    );
    let old = decode_member(&encode_member(&ApplicationSchemaMember::ManagedIndex {
        definition: definition(),
    }));
    let ApplicationSchemaMember::ManagedIndex { definition: old } = old else {
        unreachable!()
    };
    assert_eq!(
        old.parts().structure,
        ApplicationManagedIndexSourceStructure::Opaque
    );
}

#[test]
fn managed_index_membership_keeps_old_versions_conservative_and_requires_actual_tree_profile() {
    for version in [1, 2, 3] {
        let mut parts = definition().parts().clone();
        parts.semantic_version = version;
        let member = ApplicationSchemaMember::ManagedIndex {
            definition: ErasedApplicationManagedIndexDefinition::from_untrusted_parts(
                ARCHIVE_DECODE,
                parts.clone(),
            ),
        };
        assert_eq!(decode_member(&encode_member(&member)), member);
        parts.membership = ApplicationManagedIndexMembership::ExhaustiveSourceTree;
        assert_eq!(
            ErasedApplicationManagedIndexDefinition::from_untrusted_parts(
                ARCHIVE_DECODE,
                parts.clone()
            )
            .validate(),
            Err(ApplicationManagedIndexDefinitionDenial::UnsupportedMembership)
        );
        if version >= 2 {
            parts.structure = ApplicationManagedIndexSourceStructure::BinarySourceTree;
            let definition = ErasedApplicationManagedIndexDefinition::from_untrusted_parts(
                ARCHIVE_DECODE,
                parts,
            );
            if version == 2 {
                assert_eq!(
                    definition.validate(),
                    Err(ApplicationManagedIndexDefinitionDenial::UnsupportedMembership)
                );
            } else {
                let member = ApplicationSchemaMember::ManagedIndex { definition };
                let bytes = encode_member(&member);
                assert_eq!(&bytes[..4], &[0, 48, 0, 3]);
                assert_eq!(decode_member(&bytes), member);
            }
        }
    }
    // Literal v3 name/provider/version, BinarySourceTree and unknown membership3.
    let mut parts = definition().parts().clone();
    parts.semantic_version = 3;
    parts.structure = ApplicationManagedIndexSourceStructure::BinarySourceTree;
    parts.membership = ApplicationManagedIndexMembership::CheckedSummarySourceTree;
    let member = ApplicationSchemaMember::ManagedIndex {
        definition: ErasedApplicationManagedIndexDefinition::from_untrusted_parts(
            ARCHIVE_DECODE,
            parts.clone(),
        ),
    };
    assert_eq!(decode_member(&encode_member(&member)), member);
    for version in [1, 2] {
        parts.semantic_version = version;
        if version == 1 {
            parts.structure = ApplicationManagedIndexSourceStructure::Opaque;
        }
        assert_eq!(
            ErasedApplicationManagedIndexDefinition::from_untrusted_parts(
                ARCHIVE_DECODE,
                parts.clone()
            )
            .validate(),
            Err(ApplicationManagedIndexDefinitionDenial::UnsupportedMembership)
        );
    }
    // No Source bytes follow: reject the strategy before interpreting any payload.
    assert_eq!(
        try_decode_member(&[
            0, 48, 0, 3, 0, 0, 0, 1, b'I', 0, 0, 0, 1, b'P', 0, 0, 0, 1, 0, 1, 0, 3
        ])
        .unwrap_err()
        .kind(),
        Kind::UnsupportedRecordVariant
    );
}
