//! Actual typed declaration, transport and fresh closure for finite logical members.
use super::{decode_member, encode_member, encode_payload, frame_payload, try_decode_member};
use crate::facade::{
    WorthQueryPackageArchiveDenialKind as Kind, WorthQueryPackageArchiveLimits as Limits,
    WorthQueryPackageArchiveRecordDecoder,
};
use worth_query_declaration::facade::application_schema::*;
use worth_query_installation::facade::WorthQueryPortablePackageRecord;

struct Schema;
struct Face;
struct FaceUses;
struct Negative;
struct Positive;

impl ApplicationEntityMarkerIdentity<Schema> for Face {
    const IDENTIFIER: &'static str = "Face";
}

#[test]
fn unprojected_schema_keeps_frozen_v18_basis_and_entity_wire() {
    use worth_foundational::facade::{
        prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
        CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue as Value,
        CanonicalIntegerWidth as Width, CanonicalizationRuleVersion,
    };
    let entity = ApplicationSchemaMember::Entity {
        entity: "Face".into(),
    };
    let bytes = [0, 1, 0, 0, 0, 4, b'F', b'a', b'c', b'e'];
    assert_eq!(encode_member(&entity), bytes);
    assert_eq!(decode_member(&bytes), entity);
    let actual = validate_portable_application_schema_freshly(schema(vec![entity])).unwrap();
    let domain = CanonicalBasisDomain::Future("worth-query.application-schema");
    let entries: Vec<_> = [
        (
            "header.owner",
            Value::ExactText("worth.tests.projected".into()),
        ),
        ("header.name", Value::ExactText("ProjectedDomain".into())),
        (
            "header.major",
            Value::UnsignedInteger {
                width: Width::Bits32,
                value: 1,
            },
        ),
        (
            "header.minor",
            Value::UnsignedInteger {
                width: Width::Bits32,
                value: 0,
            },
        ),
        (
            "contribution-count",
            Value::UnsignedInteger {
                width: Width::Bits64,
                value: 0,
            },
        ),
        (
            "member-count",
            Value::UnsignedInteger {
                width: Width::Bits64,
                value: 1,
            },
        ),
        ("member[0].kind", Value::ExactText("entity".into())),
        ("member[0].entity", Value::ExactText("Face".into())),
    ]
    .into_iter()
    .map(|(locus, value)| {
        CanonicalBasisEntry::new(
            domain,
            CanonicalBasisLocus::Named(locus.into()),
            CanonicalBasisEntryKind::Identity,
            value,
        )
    })
    .collect();
    let expected = prepare_canonical_basis_sequence(
        CanonicalizationRuleVersion::new("worth-query-application-schema-v18").unwrap(),
        domain,
        entries,
    )
    .into_result()
    .unwrap();
    assert_eq!(actual.identity().canonical_basis(), &expected);
}

impl ApplicationSchema for Schema {
    const OWNER: &'static str = "worth.tests.projected";
    const NAME: &'static str = "ProjectedDomain";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        typed_schema(false)
    }
}

fn typed_schema(
    reverse: bool,
) -> Result<ApplicationSchemaDeclaration<Schema>, ApplicationSchemaDeclarationDenial> {
    let face = ApplicationEntityRef::<Schema, Face>::from_schema_identifier("Face");
    let domain =
        ApplicationProjectedMemberDomainRef::<Schema, FaceUses, Face>::new("FaceUses", face);
    let negative =
        ApplicationProjectedMemberCaseRef::<Schema, FaceUses, Negative>::new(domain, "Negative");
    let positive =
        ApplicationProjectedMemberCaseRef::<Schema, FaceUses, Positive>::new(domain, "Positive");
    let definition = if reverse {
        ApplicationProjectedMemberDomainDefinition::new(domain)
            .case(positive)
            .unwrap()
            .case(negative)
            .unwrap()
    } else {
        ApplicationProjectedMemberDomainDefinition::new(domain)
            .case(negative)
            .unwrap()
            .case(positive)
            .unwrap()
    };
    ApplicationSchemaDeclarationBuilder::<Schema>::for_schema()
        .entity(face)
        .projected_member_domain(definition)
        .build()
}

fn domain(version: u16, name: &str, carrier: &str, cases: &[&str]) -> ApplicationSchemaMember {
    ApplicationSchemaMember::ProjectedMemberDomain {
        definition: ErasedApplicationProjectedMemberDomainDefinition::from_untrusted_parts(
            WorthQueryPortableProjectedMemberDomainParts {
                semantic_version: version,
                domain: name.into(),
                carrier: carrier.into(),
                cases: cases.iter().map(|case| (*case).into()).collect(),
            },
        ),
    }
}

fn schema(mut members: Vec<ApplicationSchemaMember>) -> WorthQueryPortableApplicationSchemaRecord {
    members.sort();
    WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(
        WorthQueryPortableApplicationSchemaParts {
            owner: Schema::OWNER.into(),
            name: Schema::NAME.into(),
            major: 1,
            minor: 0,
            members,
            contributions: vec![],
        },
    )
}

#[test]
fn typed_projected_cases_have_order_independent_identity_and_round_trip_without_entity_members() {
    let declaration = Schema::declaration().unwrap();
    assert_eq!(
        declaration.erased().identity(),
        typed_schema(true).unwrap().erased().identity()
    );
    let record = WorthQueryPortableApplicationSchemaRecord::project(declaration.erased());
    assert_eq!(
        record
            .members()
            .iter()
            .filter(|member| matches!(member, ApplicationSchemaMember::Entity { .. }))
            .count(),
        1
    );
    let value = record
        .members()
        .iter()
        .find(|member| {
            matches!(
                member,
                ApplicationSchemaMember::ProjectedMemberDomain { .. }
            )
        })
        .unwrap();
    let bytes = encode_member(value);
    assert_eq!(&bytes[..4], &[0, 44, 0, 1]);
    assert_eq!(decode_member(&bytes), *value);
    let frame = frame_payload(&encode_payload(&record, Limits::DEFAULT));
    let mut decoder = WorthQueryPackageArchiveRecordDecoder::new(Limits::DEFAULT);
    let decoded = decoder.decode_frame(&frame).unwrap();
    let WorthQueryPortablePackageRecord::ApplicationSchema(restored) = decoded.record() else {
        unreachable!()
    };
    let fresh = validate_portable_application_schema_freshly(restored.clone()).unwrap();
    assert_eq!(fresh.identity(), declaration.erased().identity());
    let changed = validate_portable_application_schema_freshly(schema(vec![
        ApplicationSchemaMember::Entity {
            entity: "Face".into(),
        },
        domain(1, "FaceUses", "Face", &["Negative", "Third"]),
    ]))
    .unwrap();
    assert_ne!(changed.identity(), fresh.identity());
}

#[test]
fn projected_domain_shape_and_fresh_carrier_namespace_close_untrusted_meaning() {
    use ApplicationProjectedMemberDomainDefinitionDenial as Cause;
    for (value, cause) in [
        (domain(1, "FaceUses", "Face", &[]), Cause::EmptyCases),
        (
            domain(1, "FaceUses", "Face", &["Negative", "Negative"]),
            Cause::DuplicateCase,
        ),
        (
            domain(2, "FaceUses", "Face", &["Negative"]),
            Cause::UnsupportedVersion { observed: 2 },
        ),
        (
            domain(1, "Face.Uses", "Face", &["Negative"]),
            Cause::InvalidIdentifier,
        ),
    ] {
        let mut output = crate::binary_output::BinaryOutput::with_capacity(0);
        assert_eq!(
            super::super::member::write(&mut output, &value)
                .unwrap_err()
                .kind(),
            Kind::InvalidRecordShape
        );
        assert!(
            output.into_bytes().is_empty(),
            "invalid member emits no tag"
        );
        assert!(
            matches!(validate_portable_application_schema_freshly(schema(vec![value])),
            Err(ApplicationSchemaDeclarationDenial::InvalidProjectedMemberDomain { cause: observed, .. }) if observed == cause)
        );
    }
    let face = ApplicationSchemaMember::Entity {
        entity: "Face".into(),
    };
    for (members, expected) in [
        (
            vec![domain(1, "FaceUses", "Missing", &["Negative"])],
            Cause::MissingCarrier,
        ),
        (
            vec![face.clone(), domain(1, "Face", "Face", &["Negative"])],
            Cause::EntityNameCollision,
        ),
        (
            vec![
                face,
                domain(1, "FaceUses", "Face", &["Negative"]),
                domain(1, "FaceUses", "Face", &["Positive"]),
            ],
            Cause::DuplicateDomain,
        ),
    ] {
        // Individual transport validates its own shape. Only fresh schema
        // readmission has the actual carrier/domain roster needed for closure.
        let restored = members
            .iter()
            .map(|value| decode_member(&encode_member(value)))
            .collect();
        assert!(
            matches!(validate_portable_application_schema_freshly(schema(restored)),
            Err(ApplicationSchemaDeclarationDenial::InvalidProjectedMemberDomain { cause, .. }) if cause == expected)
        );
    }
    let carrier = ApplicationEntityRef::<Schema, Face>::from_schema_identifier("Face");
    let target =
        ApplicationProjectedMemberDomainRef::<Schema, FaceUses, Face>::new("FaceUses", carrier);
    let foreign =
        ApplicationProjectedMemberDomainRef::<Schema, FaceUses, Face>::new("ForeignUses", carrier);
    let foreign_case =
        ApplicationProjectedMemberCaseRef::<Schema, FaceUses, Negative>::new(foreign, "Negative");
    assert!(matches!(
        ApplicationProjectedMemberDomainDefinition::new(target).case(foreign_case),
        Err(Cause::ForeignCase)
    ));
}

#[test]
fn unknown_projected_subgrammar_refuses_before_fields_and_existing_nested_budget_still_applies() {
    assert_eq!(
        try_decode_member(&[0, 44, 0, 2]).unwrap_err().kind(),
        Kind::UnsupportedRecordVariant
    );
    let declaration = Schema::declaration().unwrap();
    let record = WorthQueryPortableApplicationSchemaRecord::project(declaration.erased());
    let frame = frame_payload(&encode_payload(&record, Limits::DEFAULT));
    let mut decoder =
        WorthQueryPackageArchiveRecordDecoder::new(Limits::DEFAULT.with_maximum_nesting_depth(1));
    assert_eq!(
        decoder.decode_frame(&frame).unwrap_err().kind(),
        Kind::NestingDepthBudgetExceeded
    );
    assert_eq!(decoder.work(), Default::default());
}
