//! Portable history meaning is exact; these descriptions confer no retained authority.
use super::*;
use worth_foundational::facade::{
    AbsenceLaw, AspectContractRevision, AspectIdentity, AspectValue, ScalarAspectType,
};
use worth_query_declaration::facade::{
    application_query::*, application_schema::*, portable_identity::WorthQueryPortableTypeIdentity,
};

fn identity(name: &str) -> WorthQueryPortableTypeIdentity {
    WorthQueryPortableTypeIdentity::from_untrusted(format!("worth.archive.history.{name}.v1"))
}
fn history() -> ErasedApplicationHistoryQueryDefinition {
    let history_type = identity("history");
    let disclosure = ApplicationQueryDisclosureContract::from_untrusted_parts(
        WorthQueryPortableApplicationQueryDisclosureParts {
            posture: ApplicationQueryDisclosurePosture::Governed,
            classification: "audit-metadata".to_owned(),
            capability_name: Some("InspectHistory".to_owned()),
            capability_type: Some(identity("capability")),
            rules: ApplicationHistoryMetadataFacet::ALL
                .into_iter()
                .map(|facet| {
                    ApplicationQueryDisclosureRule::from_untrusted_fields(
                        ApplicationQueryDisclosureSelector::HistoryMetadata(
                            ApplicationHistoryMetadataMeaning::from_untrusted_fields(
                                history_type.clone(),
                                facet,
                            ),
                        ),
                        AspectValue::Bool(true),
                        ApplicationQueryInfluenceContract::permit([facet.influence()]),
                    )
                })
                .collect(),
        },
    );
    ErasedApplicationHistoryQueryDefinition::from_untrusted_parts(
        WorthQueryPortableApplicationHistoryQueryParts {
            name: "InspectHistory".to_owned(),
            history_type,
            subject_read: ApplicationQueryBindingReference::from_untrusted_parts(
                ApplicationQueryBindingReferenceParts {
                    binding: identity("binding"),
                    input: identity("input"),
                    query_name: "SubjectRead".to_owned(),
                    query: identity("query"),
                    parameters: identity("parameters"),
                    result: identity("result"),
                    scope: identity("scope"),
                    scope_entity: "Node".to_owned(),
                    scope_aspect: "Identity".to_owned(),
                    scope_field: "Key".to_owned(),
                    scope_value: identity("key"),
                    scope_resolution: ApplicationQueryScopeResolutionMode::InputField,
                    scope_writable: false,
                    principal_binding: "AuditPrincipal".to_owned(),
                },
            ),
            target: ApplicationHistoryTarget::Entity {
                entity: "Node".to_owned(),
            },
            aspects: vec![ApplicationHistoryAspectSelection {
                aspect: "Payload".to_owned(),
                identity: AspectIdentity(0x91811003),
                revision: AspectContractRevision(1),
            }],
            metadata_disclosure: disclosure,
        },
    )
}
fn relation_aspect() -> ErasedApplicationRelationAspectDefinition {
    ErasedApplicationRelationAspectDefinition::from_untrusted_parts(
        WorthQueryPortableRelationAspectParts {
            relation: "Edge".to_owned(),
            from: "Node".to_owned(),
            to: "Node".to_owned(),
            aspect: "Payload".to_owned(),
            identity: AspectIdentity(0x91811003),
            revision: AspectContractRevision(1),
            absence: AbsenceLaw::Optional,
            fields: vec![ApplicationRelationFieldDefinition {
                field: "Weight".to_owned(),
                presence: ApplicationFieldPresence::Required,
                scalar_family: ScalarAspectType::UInt64,
                value_type: "worth.rust.u64".to_owned(),
                canonical: None,
                unit: Some("worth.units.count.v1".to_owned()),
                frame: None,
                writable: true,
            }],
        },
    )
}
#[test]
fn exact_history_and_relation_aspect_members_roundtrip_without_entity_aliases() {
    let entity = history();
    entity.validate().unwrap();
    let mut relation = entity.parts().clone();
    relation.target = ApplicationHistoryTarget::Relation {
        query: identity("query"),
        slot: identity("edge-slot"),
        relation: "Edge".to_owned(),
        from: "Node".to_owned(),
        to: "Node".to_owned(),
        direction: ApplicationQueryResultTraversalDirection::Forward,
        cardinality: ApplicationQueryCardinality::Many,
        output_name: "edges".to_owned(),
    };
    let relation = ErasedApplicationHistoryQueryDefinition::from_untrusted_parts(relation);
    relation.validate().unwrap();
    assert_ne!(entity.canonical_basis(), relation.canonical_basis());
    for member in [
        ApplicationSchemaMember::ApplicationHistoryQuery { definition: entity },
        ApplicationSchemaMember::ApplicationHistoryQuery {
            definition: relation,
        },
        ApplicationSchemaMember::RelationAspect {
            definition: relation_aspect(),
        },
    ] {
        let bytes = encode_member(&member);
        assert_eq!(decode_member(&bytes), member);
        assert_eq!(encode_member(&decode_member(&bytes)), bytes);
    }
}
#[test]
fn history_metadata_cannot_borrow_subject_slots_or_omit_a_facet() {
    let source = history();
    let mut parts = source.parts().clone();
    let source_metadata = &parts.metadata_disclosure;
    let mut metadata = WorthQueryPortableApplicationQueryDisclosureParts {
        posture: source_metadata.posture(),
        classification: source_metadata.classification().to_owned(),
        capability_name: source_metadata.capability_name().map(str::to_owned),
        capability_type: source_metadata.capability_identity(),
        rules: source_metadata.rules().to_vec(),
    };
    metadata.rules.pop();
    parts.metadata_disclosure = ApplicationQueryDisclosureContract::from_untrusted_parts(metadata);
    let incomplete = ErasedApplicationHistoryQueryDefinition::from_untrusted_parts(parts);
    assert_eq!(
        incomplete.validate(),
        Err(ApplicationHistoryQueryDefinitionDenial::InvalidMetadataDisclosure)
    );
    assert_ne!(source.canonical_basis(), incomplete.canonical_basis());
    let mut stolen = source.parts().clone();
    stolen.history_type = identity("other-history");
    assert_eq!(
        ErasedApplicationHistoryQueryDefinition::from_untrusted_parts(stolen).validate(),
        Err(ApplicationHistoryQueryDefinitionDenial::InvalidMetadataDisclosure)
    );
    let member = ApplicationSchemaMember::ApplicationHistoryQuery {
        definition: incomplete,
    };
    let mut output = BinaryOutput::with_capacity(0);
    assert_eq!(
        super::super::member::write(&mut output, &member)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::InvalidRecordShape
    );
}
#[test]
fn relation_payload_canonical_contract_refuses_before_any_member_bytes() {
    let mut parts = relation_aspect().parts().clone();
    parts.fields[0].canonical = Some(ApplicationCanonicalValueContract::of::<u64>(
        worth_foundational::facade::ValueDecodeBounds::new(16, 16, 1, 1),
    ));
    let member = ApplicationSchemaMember::RelationAspect {
        definition: ErasedApplicationRelationAspectDefinition::from_untrusted_parts(parts),
    };
    let mut output = BinaryOutput::with_capacity(0);
    assert_eq!(
        super::super::member::write(&mut output, &member)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::UnsupportedCanonicalValueContract
    );
    assert!(output.into_bytes().is_empty());
}
#[test]
fn legacy_entity_body_refuses_a_relation_locus_hidden_in_a_child_of_a_peer() {
    let field = ApplicationOperationBodyCreatedField::Literal {
        field: ApplicationFieldBindingLocus::for_relation("Edge", "Payload", "Weight"),
        binding: identity("u64"),
        value: AspectValue::UInt64(7),
    };
    let child = ApplicationOperationBodyChildCreation {
        parent_key: "peer".to_owned(),
        creation: ApplicationOperationBodyCreation::from_untrusted(
            "Node".to_owned(),
            "child".to_owned(),
            vec![field],
        ),
    };
    let peer = ApplicationOperationBodyCreation::from_untrusted(
        "Node".to_owned(),
        "peer".to_owned(),
        vec![],
    )
    .with_untrusted_children(vec![child]);
    let root = ApplicationOperationBodyCreation::from_untrusted(
        "Node".to_owned(),
        "root".to_owned(),
        vec![],
    )
    .with_untrusted_peers(vec![peer]);
    let body = ApplicationOperationBodyProgram::from_untrusted_with_creation(vec![], Some(root));
    assert!(!body.has_only_entity_field_loci());
    let mut output = BinaryOutput::with_capacity(0);
    assert_eq!(
        crate::record::operation_body::write(&mut output, &body)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::InvalidRecordShape
    );
    assert!(output.into_bytes().is_empty());
}
