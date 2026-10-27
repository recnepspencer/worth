//! The public authoring route and the distinct canonical transport meet here.
use super::*;
use crate::facade::WorthQueryPackageArchiveDenialKind as Kind;
use worth_query_declaration::facade::application_schema::*;

struct Fixture;
struct Carrier;
struct Uses;
struct Negative;
struct Positive;
struct Coupling;
struct Cohorts;
impl ApplicationEntityMarkerIdentity<Fixture> for Carrier {
    const IDENTIFIER: &'static str = "E";
}
impl ApplicationSchema for Fixture {
    const OWNER: &'static str = "archive.tests";
    const NAME: &'static str = "Logical";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration() -> Result<ApplicationSchemaDeclaration<Self>, Denial> {
        let carrier = ApplicationEntityRef::<Self, Carrier>::from_schema_identifier("E");
        let domain =
            ApplicationProjectedMemberDomainRef::<Self, Uses, Carrier>::new("Uses", carrier);
        let negative =
            ApplicationProjectedMemberCaseRef::<Self, Uses, Negative>::new(domain, "Negative");
        let positive =
            ApplicationProjectedMemberCaseRef::<Self, Uses, Positive>::new(domain, "Positive");
        let pair = || {
            ApplicationDerivedRelation::carrier_case(domain, negative)
                .unwrap()
                .reversed()
                .then(ApplicationDerivedRelation::carrier_case(domain, positive).unwrap())
        };
        let source = ApplicationDerivedRelationRef::<Self, Coupling, Uses, Uses>::projected(
            "Coupling", domain,
        );
        let reference =
            ApplicationComponentSetRef::<Self, Cohorts, Uses>::projected("Cohorts", domain);
        ApplicationSchemaDeclarationBuilder::<Self>::for_schema()
            .entity(carrier)
            .projected_member_domain(
                ApplicationProjectedMemberDomainDefinition::new(domain)
                    .case(negative)
                    .unwrap()
                    .case(positive)
                    .unwrap(),
            )
            .derived_relation(
                source,
                pair()
                    .union(pair().reversed())
                    .intersection(pair().union(pair().reversed()))
                    .without_self_edges(),
                Laws::unbounded(),
            )
            .derived_components(ApplicationDerivedComponentSetDefinition::new(
                reference, source,
            ))
            .build()
    }
}

#[test]
fn logical_recipe_and_components_have_distinct_canonical_transport_and_real_carrier_closure() {
    let declaration = Fixture::declaration().unwrap();
    let record = WorthQueryPortableApplicationSchemaRecord::project(declaration.erased());
    let sources =
        application_derived_relation_source_closure(record.members(), "Coupling").unwrap();
    assert!(sources.physical.is_empty());
    assert_eq!(sources.projected_domains.len(), 1);
    assert_eq!(sources.projected_domains[0].carrier, "E");
    assert_eq!(sources.projected_domains[0].cases, ["Negative", "Positive"]);
    let mut restored = Vec::new();
    for member in record.members() {
        let bytes = encode_member(member);
        match member {
            Member::DerivedRelation { .. } => assert_eq!(&bytes[..4], &[0, 47, 0, 1]),
            Member::DerivedComponentSet { .. } => assert_eq!(&bytes[..4], &[0, 46, 0, 1]),
            _ => {}
        }
        restored.push(decode_member(&bytes));
    }
    assert_eq!(
        readmit(logical_schema(restored.clone())).unwrap().members(),
        readmit(logical_schema(record.members().to_vec()))
            .unwrap()
            .members()
    );
    let derived = restored
        .iter_mut()
        .find_map(|member| match member {
            Member::DerivedRelation { definition } => Some(definition),
            _ => None,
        })
        .unwrap();
    let mut parts = derived.parts().clone();
    let Node::CarrierLift { case, .. } = &mut parts.expression.nodes[0] else {
        unreachable!()
    };
    *case = "Foreign".into();
    *derived = Definition::from_untrusted_parts(parts);
    assert!(matches!(
        readmit(logical_schema(restored)),
        Err(Denial::InvalidApplicationDerivedRelation {
            cause: Cause::ForeignCase,
            ..
        })
    ));
    for tag in [46u16, 47] {
        let bytes = [tag.to_be_bytes(), 2u16.to_be_bytes()].concat();
        assert_eq!(
            try_decode_member(&bytes).unwrap_err().kind(),
            Kind::UnsupportedRecordVariant
        );
    }
}

#[test]
fn legacy_recipe_bytes_stay_exact_and_cannot_be_readmitted_under_extended_tag() {
    let legacy = derived("Legacy", vec![source(SourceKind::Stored, "Edge")]);
    let mut expected = 41u16.to_be_bytes().to_vec();
    for name in ["Legacy", "E", "E"] {
        text(&mut expected, name);
    }
    expected.extend_from_slice(&1u32.to_be_bytes());
    expected.extend_from_slice(&1u16.to_be_bytes());
    expected.extend_from_slice(&1u16.to_be_bytes());
    for name in ["Edge", "E", "E"] {
        text(&mut expected, name);
    }
    let law_offset = expected.len();
    // Same-context, self allowed, six absent bounds, acyclic, then four absences.
    for value in [3u16, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0] {
        expected.extend_from_slice(&value.to_be_bytes());
    }
    assert_eq!(encode_member(&legacy), expected);
    assert_eq!(decode_member(&expected), legacy);
    let mut alternate = vec![0, 47, 0, 1];
    text(&mut alternate, "Legacy");
    for _ in 0..2 {
        alternate.extend_from_slice(&1u16.to_be_bytes());
        text(&mut alternate, "E");
    }
    alternate.extend_from_slice(&1u32.to_be_bytes());
    alternate.extend_from_slice(&1u16.to_be_bytes());
    alternate.extend_from_slice(&1u16.to_be_bytes());
    text(&mut alternate, "Edge");
    for _ in 0..2 {
        alternate.extend_from_slice(&1u16.to_be_bytes());
        text(&mut alternate, "E");
    }
    alternate.extend_from_slice(&expected[law_offset..]);
    assert_eq!(
        try_decode_member(&alternate).unwrap_err().kind(),
        Kind::InvalidRecordShape
    );
}

fn text(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as u32).to_be_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

fn logical_schema(mut members: Vec<Member>) -> WorthQueryPortableApplicationSchemaRecord {
    members.sort();
    WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(Parts {
        owner: Fixture::OWNER.into(),
        name: Fixture::NAME.into(),
        major: 1,
        minor: 0,
        members,
        contributions: vec![],
    })
}
