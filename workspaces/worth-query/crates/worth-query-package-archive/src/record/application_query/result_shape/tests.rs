use worth_foundational::facade::ScalarAspectType;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryCardinality, ApplicationQueryPredicate, ApplicationQueryRelationPredicate,
    ApplicationQueryResultRelation, ApplicationQueryResultShape,
    ApplicationQueryResultTraversalDirection,
    WorthQueryPortableApplicationQueryResultRelationParts,
    WorthQueryPortableApplicationQueryResultShapeParts,
};
use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;

use crate::binary_input::BinaryInput;
use crate::binary_output::BinaryOutput;
use crate::limits::WorthQueryPackageArchiveLimits;
use crate::record::decode_budget::RecordDecodeAttempt;

use super::{decode_at_depth, write};
use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;

#[test]
fn relation_predicate_round_trips_exact_owned_meaning() {
    for absent in [false, true] {
        let source = filtered_shape(absent);
        let mut output = BinaryOutput::with_capacity(1024);
        write(&mut output, &source).unwrap();
        let bytes = output.into_bytes();
        let mut input = BinaryInput::new(&bytes);
        let mut budget = RecordDecodeAttempt::begin(
            Default::default(),
            u64::try_from(bytes.len()).unwrap(),
            WorthQueryPackageArchiveLimits::DEFAULT,
        )
        .unwrap();

        let decoded = decode_at_depth(&mut input, &mut budget, 1).unwrap();
        assert!(input.is_finished());
        assert_eq!(decoded, source);
        assert_eq!(
            decoded.relations()[0].predicate().unwrap().field(),
            Some(("Child", "Facts", "key"))
        );
    }
}

#[test]
fn intrinsic_identity_predicate_round_trips_its_current_full_meaning() {
    let source = ApplicationQueryRelationPredicate::EntityEqualParameter {
        entity: "Child".into(),
        parameter: "selected-entity".into(),
    };
    let mut output = BinaryOutput::with_capacity(128);
    super::write_relation_predicate(&mut output, &source).unwrap();
    let bytes = output.into_bytes();
    assert_eq!(bytes[0], 3);
    let mut input = BinaryInput::new(&bytes);
    assert_eq!(
        super::decode_relation_predicate(&mut input).unwrap(),
        source
    );
    assert!(input.is_finished());
}

#[test]
fn unknown_or_incomplete_relation_operators_refuse_in_the_current_record_window() {
    use crate::denial::WorthQueryPackageArchiveDenialKind as Kind;
    assert_eq!(
        super::decode_relation_predicate(&mut BinaryInput::new(&[4]))
            .unwrap_err()
            .kind(),
        Kind::UnsupportedRecordVariant
    );
    for tag in [1, 2, 3] {
        assert_eq!(
            super::decode_relation_predicate(&mut BinaryInput::new(&[tag]))
                .unwrap_err()
                .kind(),
            Kind::Truncated
        );
    }
    use crate::compatibility::{
        WorthQueryPackageArchiveCompatibilityProfile as Profile,
        WorthQueryPackageArchiveProtocolLayer as Layer,
    };
    let current = crate::record::WORTH_QUERY_PACKAGE_ARCHIVE_RECORD_PROTOCOL_VERSION;
    assert!(Profile::CURRENT.admit(Layer::RecordFrame, current).is_ok());
    assert!(Profile::CURRENT
        .admit(Layer::RecordFrame, current - 1)
        .is_err());
}

fn filtered_shape(absent: bool) -> ApplicationQueryResultShape {
    let identity = |value: &str| {
        WorthQueryPortableTypeIdentity::from_untrusted(ARCHIVE_DECODE, value.to_owned())
    };
    let child = ApplicationQueryResultShape::from_untrusted_parts(
        ARCHIVE_DECODE,
        WorthQueryPortableApplicationQueryResultShapeParts {
            query_type: identity("Query"),
            root_entity: "Child".to_owned(),
            result_type: identity("ChildResult"),
            fields: Vec::new(),
            relations: Vec::new(),
        },
    );
    let relation = ApplicationQueryResultRelation::from_untrusted_parts(
        ARCHIVE_DECODE,
        WorthQueryPortableApplicationQueryResultRelationParts {
            query_type: identity("Query"),
            slot_type: identity("ChildSlot"),
            relation: "RootAllChild".to_owned(),
            from: "Root".to_owned(),
            to: "Child".to_owned(),
            direction: ApplicationQueryResultTraversalDirection::Forward,
            output_name: "child".to_owned(),
            cardinality: ApplicationQueryCardinality::ExactlyOne,
            predicate: Some(if absent {
                ApplicationQueryRelationPredicate::FieldAbsent {
                    entity: "Child".to_owned(),
                    aspect: "Facts".to_owned(),
                    field: "key".to_owned(),
                    scalar_family: ScalarAspectType::String,
                }
            } else {
                ApplicationQueryRelationPredicate::EqualParameter(
                    ApplicationQueryPredicate::from_untrusted_fields(
                        ARCHIVE_DECODE,
                        "Child".to_owned(),
                        "Facts".to_owned(),
                        "key".to_owned(),
                        "selected".to_owned(),
                        ScalarAspectType::String,
                    ),
                )
            }),
            nested_shape: child,
        },
    );
    ApplicationQueryResultShape::from_untrusted_parts(
        ARCHIVE_DECODE,
        WorthQueryPortableApplicationQueryResultShapeParts {
            query_type: identity("Query"),
            root_entity: "Root".to_owned(),
            result_type: identity("RootResult"),
            fields: Vec::new(),
            relations: vec![relation],
        },
    )
}
