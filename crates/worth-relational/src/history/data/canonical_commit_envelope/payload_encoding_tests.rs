//! Schema-continuity codec evidence from a genuine Native transition source.
use crate::history::data::{BranchId, RelationalCommitReceipt};
use crate::identity::data::KindId;
use crate::schema::data::*;
use crate::tests::support::{canonical_schema_envelope_input, AspectSchemaFixture};
use serde::de::IgnoredAny;
use serde::Deserialize;
#[path = "payload_encoding_tests/transition_source.rs"]
mod transition_source;

// The payload is a tuple, distinct from the text/checkpoint envelope codecs.
// Decode unrelated slots opaquely; assert the complete typed schema material,
// transition and semantics against the validated schema-continuity input below.
#[derive(Deserialize)]
struct Payload(
    String,
    Identity,
    IgnoredAny,
    Schema,
    IgnoredAny,
    crate::transactions::data::RelationalCommitSemanticResult,
);
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
    IgnoredAny,
    IgnoredAny,
);
#[derive(Deserialize)]
struct Schema(
    SchemaVersionId,
    (
        worth_foundational::facade::CanonicalDigestId,
        Option<SchemaAuthoritySnapshot>,
    ),
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
    let mut lowered = crate::schema::lower_schema_transition(
        validated,
        policy,
        semantics.current_write_version(),
        crate::schema::data::runtime_descriptor_canonical_basis_policy().current_write_version(),
    );
    // Native's schema-continuity phase binds the lowered lineage to the
    // publishing branch; the standalone lowering deliberately leaves it open.
    lowered
        .reconciliation_descriptor
        .resulting_lineage
        .branch_context = Some(BranchId("main".into()));
    let proposed = lowered.validated.proposed.clone();
    let input = canonical_schema_envelope_input(&registry, Some(lowered));
    assert!(input.authored_index_writes.is_unavailable());
    let envelope = transition_source::commit(&registry, proposed, policy);
    crate::schema::validate_schema_continuity_bundle(&envelope).unwrap();
    assert_eq!(&*envelope.schema_authority, &*input.schema_authority);
    assert_eq!(
        envelope.schema_transition.as_ref(),
        input.schema_transition.as_ref()
    );
    let bytes = envelope.encode_authoritative_payload().unwrap();
    let Payload(domain, identity, _, decoded, _, semantic) =
        rmp_serde::from_slice::<Payload>(&bytes).unwrap();
    assert_eq!(domain, "worth.relational.original-root.v4");
    assert_eq!(&semantic, envelope.semantic_result());
    assert_eq!(identity.0, envelope.commit);
    assert_eq!(identity.1, envelope.branch_context);
    let material = decoded
        .1
         .1
        .expect("a transition payload carries its full registered schema");
    assert!(
        !material.entity_kinds.is_empty(),
        "the schema fixture must be substantive"
    );
    assert_eq!(&*material, &*envelope.schema_authority);
    assert_eq!(decoded.1 .0.bytes(), &material.digest());
    assert_eq!(decoded.0, envelope.schema_version);
    assert_eq!(decoded.2.as_ref(), envelope.schema_transition.as_ref());
    assert!(decoded.2.is_some());
    assert_eq!(decoded.5, envelope.descriptor_semantics_version);
}
