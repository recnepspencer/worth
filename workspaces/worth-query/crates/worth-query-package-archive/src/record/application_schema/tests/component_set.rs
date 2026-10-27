//! Component meaning uses the sole record grammar and fresh schema readmission.
use super::{decode_member, encode_member, encode_payload, frame_payload};
use crate::facade::WorthQueryPackageArchiveDenialKind as Kind;
use crate::facade::{
    WorthQueryPackageArchiveLimits as Limits, WorthQueryPackageArchiveRecordDecoder,
};
use worth_query_declaration::facade::application_schema::*;
use worth_query_installation::facade::WorthQueryPortablePackageRecord;

fn component(sources: &[&str]) -> ApplicationSchemaMember {
    ApplicationSchemaMember::ComponentSet {
        definition: ApplicationComponentSetMeaning::from_untrusted_parts(
            WorthQueryPortableApplicationComponentSetParts {
                name: "Shells".into(),
                member: "Node".into(),
                relations: sources.iter().map(|s| (*s).into()).collect(),
            },
        ),
    }
}
fn schema(
    component: ApplicationSchemaMember,
    heterogeneous: bool,
) -> WorthQueryPortableApplicationSchemaRecord {
    let mut members = vec![
        ApplicationSchemaMember::Entity {
            entity: "Node".into(),
        },
        ApplicationSchemaMember::Entity {
            entity: "Other".into(),
        },
        ApplicationSchemaMember::Relation {
            relation: "A".into(),
            from: "Node".into(),
            to: "Node".into(),
            integrity: ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
                .into(),
        },
        ApplicationSchemaMember::Relation {
            relation: "B".into(),
            from: "Node".into(),
            to: if heterogeneous { "Other" } else { "Node" }.into(),
            integrity: ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
                .into(),
        },
        component,
    ];
    members.sort();
    WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(
        WorthQueryPortableApplicationSchemaParts {
            owner: "worth.tests.components".into(),
            name: "ComponentArchive".into(),
            major: 1,
            minor: 0,
            members,
            contributions: vec![],
        },
    )
}

#[test]
fn component_roster_has_one_additive_member_tag_and_exact_fresh_schema_meaning() {
    let meaning = component(&["B", "A", "B"]);
    assert_eq!(meaning, component(&["A", "B"]));
    let bytes = encode_member(&meaning);
    assert_eq!(&bytes[..2], &38_u16.to_be_bytes());
    assert_eq!(decode_member(&bytes), meaning);
    let record = schema(meaning, false);
    let expected = validate_portable_application_schema_freshly(record.clone()).unwrap();
    let payload = encode_payload(&record, Limits::DEFAULT);
    let frame = frame_payload(&payload);
    let mut decoder = WorthQueryPackageArchiveRecordDecoder::new(Limits::DEFAULT);
    let decoded = decoder.decode_frame(&frame).unwrap();
    assert_eq!(
        decoded.record(),
        &WorthQueryPortablePackageRecord::ApplicationSchema(record)
    );
    let WorthQueryPortablePackageRecord::ApplicationSchema(restored) = decoded.record() else {
        unreachable!()
    };
    assert_eq!(
        expected.identity(),
        validate_portable_application_schema_freshly(restored.clone())
            .unwrap()
            .identity()
    );
}

#[test]
fn decoded_empty_missing_or_heterogeneous_rosters_and_duplicate_names_refuse_readmission() {
    for (sources, heterogeneous) in [
        (vec![], false),
        (vec!["Missing"], false),
        (vec!["A", "B"], true),
    ] {
        let restored = decode_member(&encode_member(&component(&sources)));
        assert_eq!(
            validate_portable_application_schema_freshly(schema(restored, heterogeneous))
                .unwrap_err(),
            ApplicationSchemaDeclarationDenial::InvalidComponentSet
        );
    }
    let mut record = schema(component(&["A"]), false);
    let mut members = record.members().to_vec();
    members.push(component(&["B"]));
    members.sort();
    record = WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(
        WorthQueryPortableApplicationSchemaParts {
            owner: record.owner().into(),
            name: record.name().into(),
            major: record.major(),
            minor: record.minor(),
            members,
            contributions: vec![],
        },
    );
    assert_eq!(
        validate_portable_application_schema_freshly(record).unwrap_err(),
        ApplicationSchemaDeclarationDenial::DuplicateMember
    );
}

#[test]
fn component_roster_decode_admits_nested_entries_and_depth_before_allocating() {
    let bytes = encode_member(&component(&["A", "B"]));
    for (limits, expected) in [
        (
            Limits::DEFAULT.with_maximum_nested_entries(1),
            Kind::NestedEntryBudgetExceeded,
        ),
        (
            Limits::DEFAULT.with_maximum_nesting_depth(1),
            Kind::NestingDepthBudgetExceeded,
        ),
    ] {
        let mut input = crate::binary_input::BinaryInput::new(&bytes);
        let mut budget = crate::record::decode_budget::RecordDecodeAttempt::begin(
            Default::default(),
            bytes.len() as u64,
            limits,
        )
        .unwrap();
        assert_eq!(
            super::super::member::decode(&mut input, &mut budget)
                .unwrap_err()
                .kind(),
            expected
        );
    }
}
