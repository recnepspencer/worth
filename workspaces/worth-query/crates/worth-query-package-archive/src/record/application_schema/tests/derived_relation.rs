//! Fresh transport and completed-schema closure for descriptive derived sources.
use super::{decode_member, encode_member, try_decode_member};
use worth_query_declaration::facade::application_schema::{
    validate_portable_application_schema_freshly as readmit,
    ApplicationDerivedRelationDefinitionDenial as Cause,
    ApplicationDerivedRelationExpression as Expression, ApplicationDerivedRelationNode as Node,
    ApplicationDerivedRelationSource as Source, ApplicationDerivedRelationSourceKind as SourceKind,
    ApplicationRelationGraphLaws as Laws, ApplicationRelationIntegrity,
    ApplicationRuleReadableCollectionAssociation as Association,
    ApplicationSchemaDeclarationDenial as Denial, ApplicationSchemaMember as Member,
    ErasedApplicationDerivedRelationDefinition as Definition,
    WorthQueryPortableApplicationSchemaParts as Parts,
    WorthQueryPortableApplicationSchemaRecord as Schema,
    WorthQueryPortableDerivedRelationParts as DerivedParts,
};

#[test]
fn derived_recipe_collection_and_read_meaning_round_trip_without_stored_ids() {
    let definition = derived(
        "TwoSteps",
        vec![
            source(SourceKind::Stored, "Edge"),
            source(SourceKind::Stored, "Edge"),
            Node::Compose { left: 0, right: 1 },
            Node::WithoutSelfEdges { input: 2 },
        ],
    );
    let collection = association("TwoSteps");
    for (member, tag) in [
        (definition.clone(), 41_u16),
        (collection.clone(), 42),
        (
            Member::OperationDerivedRead {
                operation: "Inspect".into(),
                relation: "TwoSteps".into(),
            },
            43,
        ),
    ] {
        let bytes = encode_member(&member);
        assert_eq!(&bytes[..2], &tag.to_be_bytes());
        assert_eq!(decode_member(&bytes), member);
    }
    let expected = readmit(schema(vec![definition.clone(), collection.clone()])).unwrap();
    let restored = readmit(schema(vec![
        decode_member(&encode_member(&definition)),
        decode_member(&encode_member(&collection)),
    ]))
    .unwrap();
    assert_eq!(restored.identity(), expected.identity());
    let changed = derived(
        "TwoSteps",
        vec![
            source(SourceKind::Stored, "Edge"),
            Node::Reverse { input: 0 },
        ],
    );
    assert_ne!(
        readmit(schema(vec![changed, collection]))
            .unwrap()
            .identity(),
        expected.identity()
    );
}

#[test]
fn fresh_schema_rejects_cycle_wrong_source_and_collection_association() {
    let cases = [
        (
            vec![
                derived("First", vec![source(SourceKind::Derived, "Second")]),
                derived("Second", vec![source(SourceKind::Derived, "First")]),
            ],
            Cause::CyclicDefinitions,
        ),
        (
            vec![derived(
                "First",
                vec![source(SourceKind::Stored, "Missing")],
            )],
            Cause::MissingSource,
        ),
        (
            vec![
                derived("First", vec![source(SourceKind::Stored, "Edge")]),
                association("Missing"),
            ],
            Cause::InvalidCollectionAssociation,
        ),
        (
            vec![
                derived("First", vec![source(SourceKind::Stored, "Edge")]),
                Member::OperationDerivedRead {
                    operation: "Missing".into(),
                    relation: "First".into(),
                },
            ],
            Cause::MissingOperation,
        ),
    ];
    for (members, expected) in cases {
        let decoded = members
            .iter()
            .map(|member| decode_member(&encode_member(member)))
            .collect();
        assert!(
            matches!(readmit(schema(decoded)),Err(Denial::InvalidApplicationDerivedRelation {cause,..}) if cause==expected)
        );
    }
}

#[test]
fn malformed_expression_refuses_before_tag_and_unknown_node_refuses_transport() {
    let malformed = derived(
        "First",
        vec![
            source(SourceKind::Stored, "Edge"),
            Node::Reverse { input: 1 },
        ],
    );
    let mut output = crate::binary_output::BinaryOutput::with_capacity(0);
    assert_eq!(
        super::super::member::write(&mut output, &malformed)
            .unwrap_err()
            .kind(),
        crate::facade::WorthQueryPackageArchiveDenialKind::InvalidRecordShape
    );
    assert!(output.into_bytes().is_empty());
    let valid = derived("First", vec![source(SourceKind::Stored, "Edge")]);
    let mut bytes = encode_member(&valid);
    // Member tag, three owned strings, then node count precede the first node.
    let node_tag = 2 + (4 + "First".len()) + 2 * (4 + "E".len()) + 4;
    bytes[node_tag..node_tag + 2].copy_from_slice(&99_u16.to_be_bytes());
    assert_eq!(
        try_decode_member(&bytes).unwrap_err().kind(),
        crate::facade::WorthQueryPackageArchiveDenialKind::UnsupportedRecordVariant
    );
    let record = schema(vec![valid]);
    let frame = super::frame_payload(&super::encode_payload(
        &record,
        crate::facade::WorthQueryPackageArchiveLimits::DEFAULT,
    ));
    let mut decoder = crate::facade::WorthQueryPackageArchiveRecordDecoder::new(
        crate::facade::WorthQueryPackageArchiveLimits::DEFAULT.with_maximum_nesting_depth(1),
    );
    assert_eq!(
        decoder.decode_frame(&frame).unwrap_err().kind(),
        crate::facade::WorthQueryPackageArchiveDenialKind::NestingDepthBudgetExceeded
    );
    assert_eq!(decoder.work(), Default::default());
}

#[test]
fn shared_graph_law_extraction_preserves_literal_stored_relation_wire() {
    let member = Member::Relation {
        relation: "Edge".into(),
        from: "E".into(),
        to: "E".into(),
        integrity: ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
            .acyclic()
            .into(),
    };
    let mut expected = 26_u16.to_be_bytes().to_vec();
    for text in ["Edge", "E", "E"] {
        expected.extend_from_slice(&(text.len() as u32).to_be_bytes());
        expected.extend_from_slice(text.as_bytes());
    }
    // Forbid cross-context; self allowed; six absent bounds; RetainDangling;
    // acyclic; symmetric/orbit/unique/same-partition/minimum absent.
    for value in [3_u16, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0] {
        expected.extend_from_slice(&value.to_be_bytes());
    }
    assert_eq!(encode_member(&member), expected);
    assert_eq!(decode_member(&expected), member);
}

fn source(kind: SourceKind, relation: &str) -> Node {
    Node::Source(Source {
        kind,
        relation: relation.into(),
        from: "E".into(),
        to: "E".into(),
    })
}

fn derived(name: &str, nodes: Vec<Node>) -> Member {
    Member::DerivedRelation {
        definition: Definition::from_untrusted_parts(DerivedParts {
            relation: name.into(),
            from: "E".into(),
            to: "E".into(),
            expression: Expression { nodes },
            laws: Laws::unbounded().acyclic(),
        }),
    }
}

fn association(relation: &str) -> Member {
    Member::RuleReadableCollection {
        association: Association {
            relation: relation.into(),
            feature: "fixture.feature".into(),
            feature_major: 1,
            feature_minor: 0,
            collection: "fixture.collection".into(),
            computation: "fixture.computation".into(),
            output_artifact: "fixture.output".into(),
            completeness: "fixture.complete".into(),
        },
    }
}

fn schema(mut members: Vec<Member>) -> Schema {
    members.extend([
        Member::Entity { entity: "E".into() },
        Member::Relation {
            relation: "Edge".into(),
            from: "E".into(),
            to: "E".into(),
            integrity: ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
                .into(),
        },
    ]);
    members.sort();
    Schema::from_untrusted_parts(Parts {
        owner: "archive.tests".into(),
        name: "DerivedSources".into(),
        major: 1,
        minor: 0,
        members,
        contributions: Vec::new(),
    })
}
