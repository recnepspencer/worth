//! Archive wire compatibility, round-trip, and fail-closed refusal tests for
//! related-record absence/existence predicates and internal-relation disclosure.
use crate::binary_input::BinaryInput;
use crate::binary_output::BinaryOutput;
use crate::denial::WorthQueryPackageArchiveDenialKind as Kind;
use crate::record::application_query::{decode, write};
use crate::record::decode_budget::RecordDecodeAttempt;
use worth_foundational::expression_api::expressions;
use worth_foundational::facade::{AspectValue, InternedString, ScalarAspectType};
use worth_query_declaration::facade::application_query::*;
use worth_query_declaration::facade::application_schema::*;
use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;
use worth_query_declaration::*;

struct Schema;
struct Task;
struct BlockedBy;
struct Row;
worth_query_portable_type!(Row => "worth.tests.archive.related-row.v1");
worth_query_structured_value_binding!(Params for () { identity: "worth.rust.unit" });
worth_query_structured_value_binding!(Rows for Row { identity: "worth.tests.archive.related-row.v1" });
worth_query_application_query!(Query for Schema, identity "worth.tests.archive.related.v1",
    parameters Params, result Rows, scope Task => "Task", name "related_archive");

fn plain_query() -> ErasedApplicationQueryDefinition {
    let task = ApplicationEntityRef::<Schema, Task>::from_schema_identifier("Task");
    let mut parts = ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(task)
        .scope(task)
        .result_shape(
            ApplicationQueryResultShapeBuilder::<Schema, Query, Task, Row, Rows>::new(task).build(),
        )
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .into_erased()
        .into_parts();
    parts.parameters = vec![ApplicationQueryParameterDefinition::from_untrusted_fields(
        "min_rank".into(),
        ScalarAspectType::Int64,
        WorthQueryPortableTypeIdentity::declared("worth.tests.archive.i64.v1"),
        None,
        None,
    )
    .with_optional_presence()];
    ErasedApplicationQueryDefinition::from_untrusted_parts(parts)
}

fn with_related() -> ErasedApplicationQueryDefinition {
    let mut parts = plain_query().into_parts();
    let draft = expressions()
        .parse("!is_some(min_rank) || rank >= unwrap(min_rank)")
        .unwrap()
        .encode();
    let target_filter = ApplicationQueryWhereFilterProgram::from_untrusted(
        vec![
            ApplicationQueryWhereFilterOperand::from_untrusted(
                "rank".into(),
                ApplicationQueryWhereFilterSource::Field {
                    entity: "Task".into(),
                    aspect: "TaskFacts".into(),
                    field: "Rank".into(),
                    scalar_family: ScalarAspectType::Int64,
                    presence: ApplicationFieldPresence::Required,
                },
            ),
            ApplicationQueryWhereFilterOperand::from_untrusted(
                "min_rank".into(),
                ApplicationQueryWhereFilterSource::OptionalParameter {
                    parameter: "min_rank".into(),
                    scalar_family: ScalarAspectType::Int64,
                },
            ),
        ],
        &draft,
    )
    .unwrap();
    parts.related_predicates = vec![
        ApplicationQueryRelatedPredicateMeaning::from_untrusted_parts(
            WorthQueryPortableApplicationQueryRelatedPredicateParts {
                relation: "BlockedBy".into(),
                from: "Task".into(),
                to: "Task".into(),
                direction: RelationDirection::Forward,
                polarity: ApplicationQueryRelatedPolarity::Absent,
                target_predicates: vec![
                    ApplicationQueryRelatedTargetPredicate::from_untrusted_value(
                        "Task".into(),
                        "TaskFacts".into(),
                        "Status".into(),
                        AspectValue::String(InternedString::Raw("open".into())),
                        ScalarAspectType::String,
                        ApplicationFieldPresence::Required,
                    ),
                ],
                target_where_filter: Some(target_filter),
            },
        ),
    ];
    let rel = ApplicationRelationRef::<Schema, BlockedBy, Task, Task>::from_schema_identifiers(
        "BlockedBy",
        "Task",
        "Task",
        worth_query_installation::facade::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
    );
    let allow =
        ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new("allow".into())
            .unwrap();
    parts.disclosure = parts.disclosure.use_relation_by(
        rel,
        RelationDirection::Forward,
        allow,
        ApplicationQueryInfluenceContract::permit([
            ApplicationQueryObservableInfluence::RowPresence,
            ApplicationQueryObservableInfluence::Count,
        ]),
    );
    ErasedApplicationQueryDefinition::from_untrusted_parts(parts)
}

fn encode(query: &ErasedApplicationQueryDefinition) -> Vec<u8> {
    let mut out = BinaryOutput::with_capacity(1024);
    write(&mut out, query).unwrap();
    out.into_bytes()
}

fn decode_bytes(
    bytes: &[u8],
) -> Result<(ErasedApplicationQueryDefinition, bool), crate::denial::WorthQueryPackageArchiveDenial>
{
    let mut input = BinaryInput::new(bytes);
    let mut budget = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        crate::limits::WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    let query = decode(&mut input, &mut budget)?;
    Ok((query, input.is_finished()))
}

#[test]
fn related_predicate_and_internal_relation_disclosure_round_trip_without_affecting_plain_bytes() {
    let plain = encode(&plain_query());
    assert!(!plain.windows(4).any(|w| w == b"WQRP"));

    let mut only_related_parts = plain_query().into_parts();
    only_related_parts.related_predicates = with_related().related_predicates().to_vec();
    let only_related = ErasedApplicationQueryDefinition::from_untrusted_parts(only_related_parts);
    let related_bytes = encode(&only_related);
    assert!(
        related_bytes.starts_with(&plain),
        "trailing WQRP extension leaves plain prefix byte-identical"
    );
    assert_eq!(&related_bytes[plain.len()..plain.len() + 4], b"WQRP");

    let full = with_related();
    let full_bytes = encode(&full);
    let (decoded, finished) = decode_bytes(&full_bytes).unwrap();
    assert!(finished);
    assert_eq!(decoded, full);
    assert_eq!(decoded.canonical_basis(), full.canonical_basis());
}

#[test]
fn forged_related_predicate_payloads_refuse_at_archive_boundary() {
    let plain = encode(&plain_query());
    let mut only_related_parts = plain_query().into_parts();
    only_related_parts.related_predicates = with_related().related_predicates().to_vec();
    let bytes = encode(&ErasedApplicationQueryDefinition::from_untrusted_parts(
        only_related_parts,
    ));
    let kind = |raw: &[u8]| decode_bytes(raw).unwrap_err().kind();

    // Bad version refuses.
    let mut bad_ver = bytes.clone();
    bad_ver[plain.len() + 5] = 9;
    assert_eq!(kind(&bad_ver), Kind::UnsupportedRecordVersion);

    // Truncated payload refuses.
    assert_eq!(kind(&bytes[..bytes.len() - 1]), Kind::Truncated);

    // Forged empty WQRP trailer (0 predicates) refuses.
    let mut empty_trailer = plain.clone();
    empty_trailer.extend_from_slice(b"WQRP");
    empty_trailer.extend_from_slice(&1_u16.to_be_bytes());
    empty_trailer.extend_from_slice(&0_u32.to_be_bytes());
    assert_eq!(kind(&empty_trailer), Kind::InvalidRecordShape);

    // Forged oversized (> 64 KiB) target literal refuses encoding before emitting wire bytes.
    let mut oversized = plain_query().into_parts();
    oversized.related_predicates = vec![
        ApplicationQueryRelatedPredicateMeaning::from_untrusted_parts(
            WorthQueryPortableApplicationQueryRelatedPredicateParts {
                relation: "BlockedBy".into(),
                from: "Task".into(),
                to: "Task".into(),
                direction: RelationDirection::Forward,
                polarity: ApplicationQueryRelatedPolarity::Absent,
                target_predicates: vec![
                    ApplicationQueryRelatedTargetPredicate::from_untrusted_value(
                        "Task".into(),
                        "TaskFacts".into(),
                        "Status".into(),
                        AspectValue::String(InternedString::Raw("a".repeat(64 * 1024 + 1))),
                        ScalarAspectType::String,
                        ApplicationFieldPresence::Required,
                    ),
                ],
                target_where_filter: None,
            },
        ),
    ];
    let mut out = BinaryOutput::with_capacity(256);
    assert_eq!(
        write(
            &mut out,
            &ErasedApplicationQueryDefinition::from_untrusted_parts(oversized)
        )
        .unwrap_err()
        .kind(),
        Kind::InvalidRecordShape
    );

    // Forged wrong-endpoint relation refuses on decode.
    let mut wrong_endpoint = plain_query().into_parts();
    wrong_endpoint.related_predicates = vec![
        ApplicationQueryRelatedPredicateMeaning::from_untrusted_parts(
            WorthQueryPortableApplicationQueryRelatedPredicateParts {
                relation: "Contains".into(),
                from: "Project".into(),
                to: "Task".into(),
                direction: RelationDirection::Forward,
                polarity: ApplicationQueryRelatedPolarity::Absent,
                target_predicates: vec![],
                target_where_filter: None,
            },
        ),
    ];
    let wrong_bytes = encode(&ErasedApplicationQueryDefinition::from_untrusted_parts(
        wrong_endpoint,
    ));
    assert_eq!(kind(&wrong_bytes), Kind::InvalidRecordShape);
}
