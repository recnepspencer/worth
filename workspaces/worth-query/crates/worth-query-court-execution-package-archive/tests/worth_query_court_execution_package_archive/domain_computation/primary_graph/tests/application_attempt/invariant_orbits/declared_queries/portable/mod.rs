//! Real archive frames and fresh untrusted-schema validation share this artifact.
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::*;
use worth_query_declaration::facade::application_query::*;
use worth_query_declaration::facade::application_schema::*;
use worth_query_installation::facade::*;
use worth_query_package_archive::facade::*;

#[test]
fn orbit_slots_and_disclosure_round_trip_through_current_records_and_reject_v16() {
    let declaration = OrbitSchema::<true>::declaration().unwrap();
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "invariant_orbit_test",
        1,
        0,
    ))
    .application_schema(declaration.clone())
    .validate()
    .unwrap();
    let records = package.export_typed_records().unwrap();
    let view = records
        .views()
        .find(|view| view.family() == WorthQueryPortablePackageRecordFamily::ApplicationSchema)
        .unwrap();
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let mut bytes = encode_record_frame(view, limits).unwrap();
    let decoded = WorthQueryPackageArchiveRecordDecoder::new(limits)
        .decode_frame(&bytes)
        .unwrap();
    assert_eq!(decoded.record(), view.record());
    let WorthQueryPortablePackageRecord::ApplicationSchema(record) = decoded.into_record() else {
        panic!("schema frame required")
    };
    let readmitted = validate_portable_application_schema_freshly(record).unwrap();
    assert_eq!(&readmitted, declaration.erased());
    assert_eq!(
        &bytes[..2],
        &WORTH_QUERY_PACKAGE_ARCHIVE_RECORD_PROTOCOL_VERSION.to_be_bytes()
    );
    bytes[..2].copy_from_slice(&16_u16.to_be_bytes());
    assert_eq!(
        WorthQueryPackageArchiveRecordDecoder::new(limits)
            .decode_frame(&bytes)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::UnsupportedRecordVersion
    );
}
#[test]
fn incomplete_filtered_reordered_and_non_orbit_declarations_refuse() {
    let public = || {
        builder(
            Orbit::reference::<OrbitSchema<true>>(),
            ApplicationQueryDisclosureContract::public(),
        )
    };
    assert_eq!(
        refusal(public().root_path(
            ApplicationQueryRootPath::from(Node::reference()).forward(Next::reference())
        )),
        ApplicationQueryDefinitionDenial::InvalidOrbitShape
    );
    assert_eq!(
        refusal(public().order_by(
            key::<OrbitSchema<true>, Orbit>(),
            ApplicationQueryOrderingDirection::Ascending
        )),
        ApplicationQueryDefinitionDenial::InvalidOrbitShape
    );
    let collision = ApplicationQueryOrbitRef::<OrbitSchema<true>, Orbit, KeySlot, Next, Node>::new(
        "orbit",
        Next::reference(),
    );
    assert_eq!(
        refusal(public().orbit(collision)),
        ApplicationQueryDefinitionDenial::DuplicateResultSlot
    );
    for edit in [
        (|parts: &mut WorthQueryPortableApplicationQueryParts| {
            parts.cardinality = ApplicationQueryCardinality::OptionalOne
        }) as fn(&mut WorthQueryPortableApplicationQueryParts),
        |parts| parts.root_entity = "Principal".into(),
        |parts| {
            parts
                .predicates
                .push(ApplicationQueryPredicate::from_untrusted_fields(
                    "Node".into(),
                    "NodeData".into(),
                    "NodeKey".into(),
                    "unused".into(),
                    worth_foundational::facade::ScalarAspectType::UInt64,
                ))
        },
    ] {
        let declaration = OrbitSchema::<true>::declaration().unwrap();
        let mut schema =
            WorthQueryPortableApplicationSchemaRecord::project(declaration.erased()).into_parts();
        for member in &mut schema.members {
            if let ApplicationSchemaMember::ApplicationQuery { definition } = member
                && definition.name() == "declared_orbit"
            {
                let mut parts = definition.clone().into_parts();
                edit(&mut parts);
                *definition = ErasedApplicationQueryDefinition::from_untrusted_parts(parts);
            }
        }
        assert!(validate_portable_application_schema_freshly(
            WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(schema)
        )
        .is_err());
    }
    let declaration = OrbitSchema::<false>::declaration().unwrap();
    let mut schema =
        WorthQueryPortableApplicationSchemaRecord::project(declaration.erased()).into_parts();
    schema
        .members
        .push(ApplicationSchemaMember::ApplicationQuery {
            definition: definition::<OrbitSchema<false>>().into_erased(),
        });
    schema.members.sort();
    assert!(
        validate_portable_application_schema_freshly(
            WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(schema)
        )
        .is_err(),
        "a caller's typed selector cannot install the orbit law"
    );
}

fn refusal<Q, P, R, Scope>(
    expression: ApplicationQueryDefinitionBuilder<OrbitSchema<true>, Q, P, R, Scope>,
) -> ApplicationQueryDefinitionDenial {
    match ApplicationSchemaDeclarationBuilder::<OrbitSchema<true>>::for_schema()
        .application_query(expression)
        .build()
        .unwrap_err()
    {
        ApplicationSchemaDeclarationDenial::InvalidQueryDefinition { cause, .. } => cause,
        other => panic!("expected query grammar refusal: {other:?}"),
    }
}
