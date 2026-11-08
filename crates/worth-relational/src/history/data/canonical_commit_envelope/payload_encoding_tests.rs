//! Pure schema-continuity codec evidence; no runtime publication is claimed.
use crate::history::data::{BranchId, RelationalCommitReceipt};
use crate::identity::data::KindId;
use crate::schema::data::*;
use crate::tests::support::{canonical_schema_envelope_input, AspectSchemaFixture};
use serde::de::IgnoredAny;
use serde::Deserialize;

// The payload is a tuple, distinct from the text/checkpoint envelope codecs.
// Decode unrelated slots opaquely; assert the complete typed schema material,
// transition and semantics against the validated schema-continuity input below.
#[derive(Deserialize)]
struct Payload(Identity, Semantics);
#[derive(Deserialize)]
struct Identity(
    RelationalCommitReceipt,
    BranchId,
    IgnoredAny,
    IgnoredAny,
    IgnoredAny,
    IgnoredAny,
    IgnoredAny,
    IgnoredAny,
);
#[derive(Deserialize)]
struct Semantics(
    IgnoredAny,
    IgnoredAny,
    IgnoredAny,
    IgnoredAny,
    SchemaVersionId,
    ([u8; 32], Option<SchemaAuthoritySnapshot>),
    IgnoredAny,
    IgnoredAny,
    IgnoredAny,
    IgnoredAny,
    IgnoredAny,
    Option<SchemaTransitionArtifact>,
    IgnoredAny,
    IgnoredAny,
    DescriptorSemanticsVersion,
);

#[test]
fn transition_payload_round_trips_schema_material_and_transition_identity() {
    let registry = AspectSchemaFixture {
        schema_version_id: SchemaVersionId(2),
        ..AspectSchemaFixture::with_default_declared_aspects(
            crate::config::data::CascadeDeletePolicy::CascadeDeleteRelations,
        )
    }
    .build_registry();
    let policy = Some(SchemaReconciliationPolicy::PreserveInformation);
    let validated = crate::schema::validate_schema_transition(
        ProposedSchemaTransition {
            source_schema_id: SchemaId("test".into()),
            source_schema_version_id: SchemaVersionId(1),
            target_schema_id: SchemaId("test".into()),
            target_schema_version_id: SchemaVersionId(2),
            diff_atoms: vec![SchemaDiffAtom::new(
                SchemaElementRef::new(
                    SchemaElementKind::Field,
                    SchemaId("test".into()),
                    SchemaVersionId(2),
                    Some(KindId(1)),
                    "tag",
                ),
                vec![
                    SchemaStratum::StructuralShape,
                    SchemaStratum::PublicationContract,
                ],
                SchemaPublicationImpact::ObservableSurfaceChanged,
                SchemaSubscriberImpact::ConsumableSurfaceChanged,
                HistoricalInterpretationSensitivity::NotSensitive,
                SchemaDiffDetail::AddedField {
                    field: worth_foundational::facade::FieldKey::new("tag").unwrap(),
                    required: false,
                    default_expression: Some("null".into()),
                },
            )
            .with_boundary_visibility_proof(
                SubscriberBoundaryVisibility::VisibleSemanticallyIgnorable,
            )],
        },
        policy,
    )
    .unwrap();
    let semantics = crate::schema::data::runtime_descriptor_semantics_policy();
    let lowered = crate::schema::lower_schema_transition(
        validated,
        policy,
        semantics.current_write_version(),
        crate::schema::data::runtime_descriptor_canonical_basis_policy().current_write_version(),
    );
    let envelope = canonical_schema_envelope_input(&registry, Some(lowered));
    crate::schema::validate_schema_continuity_bundle(&envelope).unwrap();
    assert!(envelope.authored_index_writes.is_unavailable());
    let bytes = envelope.encode_authoritative_payload().unwrap();
    let Payload(identity, decoded) = rmp_serde::from_slice::<Payload>(&bytes).unwrap();
    assert_eq!(identity.0, envelope.commit);
    assert_eq!(identity.1, envelope.branch_context);
    let material = decoded
        .5
         .1
        .expect("a transition payload carries its full registered schema");
    assert!(
        !material.entity_kinds.is_empty(),
        "the schema fixture must be substantive"
    );
    assert_eq!(&*material, &*envelope.schema_authority);
    assert_eq!(decoded.5 .0, material.digest());
    assert_eq!(decoded.4, envelope.schema_version);
    assert_eq!(decoded.11.as_ref(), envelope.schema_transition.as_ref());
    assert!(decoded.11.is_some());
    assert_eq!(decoded.14, envelope.descriptor_semantics_version);
}
