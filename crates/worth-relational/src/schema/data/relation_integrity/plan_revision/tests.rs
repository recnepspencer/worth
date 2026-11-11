//! Byte-sequence regression against the recorded aggregate revision grammar.
use super::*;
use crate::schema::data::{
    AggregateIdentityContractDeclaration, AggregateIdentitySource, AggregateIdentityStorage,
    AggregateProjectedMemberStorage,
};
use worth_foundational::facade::{AggregateDefinitionIdentity, AspectIdentity};

#[test]
fn aggregate_revision_streaming_preserves_the_recorded_canonical_byte_sequence() {
    let identity = crate::derived_relation::DerivedRelationIdentity {
        schema: crate::schema::data::SchemaId("schema".into()),
        version: crate::schema::data::SchemaVersionId(2),
        name: "path".into(),
    };
    for source in [
        AggregateIdentitySource::Orbit(KindId(1)),
        AggregateIdentitySource::Components {
            relations: vec![KindId(2), KindId(3)],
        },
        AggregateIdentitySource::DerivedOrbit {
            derived_orbit: identity.clone(),
        },
        AggregateIdentitySource::DerivedComponents {
            derived_components: identity,
        },
    ] {
        for projected in [
            None,
            Some(AggregateProjectedMemberStorage {
                coordinate_aspect: AspectIdentity(31),
                membership_aspect: AspectIdentity(32),
            }),
        ] {
            let aggregate = AggregateIdentityContractDeclaration {
                definition: AggregateDefinitionIdentity::from_bytes([19; 32]),
                source: source.clone(),
                member_kind: KindId(4),
                membership_kind: KindId(5),
                storage: AggregateIdentityStorage {
                    aggregate_kind: KindId(6),
                    lineage_kind: KindId(7),
                    aggregate_aspect: AspectIdentity(8),
                    lineage_aspect: AspectIdentity(9),
                    projected,
                },
            };
            // Reference is the original recorded byte sequence, built without
            // calling either production aggregate/identity byte emitter.
            let mut bytes = vec![19; 32];
            match &source {
                AggregateIdentitySource::Orbit(kind) => bytes.extend(kind.0.to_be_bytes()),
                AggregateIdentitySource::Components { relations } => {
                    bytes.extend(b"WORTH_COMPONENT_SOURCES_V1\0");
                    bytes.extend((relations.len() as u64).to_be_bytes());
                    for kind in relations {
                        bytes.extend(kind.0.to_be_bytes());
                    }
                }
                AggregateIdentitySource::DerivedOrbit { derived_orbit } => {
                    bytes.extend(b"WORTH_DERIVED_ORBIT_SOURCE_V1\0");
                    reference_identity(&mut bytes, derived_orbit);
                }
                AggregateIdentitySource::DerivedComponents { derived_components } => {
                    bytes.extend(b"WORTH_DERIVED_COMPONENT_SOURCE_V1\0");
                    reference_identity(&mut bytes, derived_components);
                }
            }
            for kind in [4u32, 5, 6, 7] {
                bytes.extend(kind.to_be_bytes());
            }
            for aspect in [8u64, 9] {
                bytes.extend(aspect.to_be_bytes());
            }
            if projected.is_some() {
                bytes.extend(b"WORTH_PROJECTED_AGGREGATE_STORAGE_V1\0");
                for aspect in [31u64, 32] {
                    bytes.extend(aspect.to_be_bytes());
                }
            }
            let mut hash = 0x6c62272e07bb014262b821756295c58du128;
            for part in [b"aggregate_identity_v1".as_slice(), bytes.as_slice()] {
                for byte in part.iter().copied().chain([255]) {
                    hash =
                        (hash ^ u128::from(byte)).wrapping_mul(0x0000000001000000000000000000013B);
                }
            }
            let actual = derive_relation_integrity_plan_revision(
                &[],
                &[],
                &[],
                &[],
                &[],
                &[],
                &[],
                &[],
                Some(&aggregate),
                None,
            );
            assert_eq!(actual.0, hash);
            let mut emitted = Vec::new();
            aggregate
                .try_write_canonical(&mut |part| {
                    emitted.extend_from_slice(part);
                    Ok(())
                })
                .unwrap();
            assert_eq!(emitted, bytes);
        }
    }
}
fn reference_identity(
    bytes: &mut Vec<u8>,
    identity: &crate::derived_relation::DerivedRelationIdentity,
) {
    for text in [&identity.schema.0, &identity.name] {
        bytes.extend((text.len() as u64).to_le_bytes());
        bytes.extend(text.as_bytes());
    }
    bytes.extend(identity.version.0.to_le_bytes());
}
