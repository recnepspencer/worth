//! Archive wire compatibility, round-trip, and fail-closed refusal tests for
//! related-record absence/existence predicates and internal-relation disclosure.
use crate::binary_input::BinaryInput;
use crate::binary_output::BinaryOutput;
use crate::denial::WorthQueryPackageArchiveDenialKind as Kind;
use crate::record::application_query::{decode, write};
use crate::record::decode_budget::RecordDecodeAttempt;
use worth_foundational::expression_api::expressions;
use worth_foundational::facade::{AspectValue, InternedString, ScalarAspectType};
use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
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
        ARCHIVE_DECODE,
        "min_rank".into(),
        ScalarAspectType::Int64,
        WorthQueryPortableTypeIdentity::declared("worth.tests.archive.i64.v1"),
        None,
        None,
    )
    .with_optional_presence()];
    ErasedApplicationQueryDefinition::from_untrusted_parts(ARCHIVE_DECODE, parts)
}

fn with_related() -> ErasedApplicationQueryDefinition {
    let mut parts = plain_query().into_parts();
    let draft = expressions()
        .parse("!is_some(min_rank) || rank >= unwrap(min_rank)")
        .unwrap()
        .encode();
    let target_filter = ApplicationQueryWhereFilterProgram::from_untrusted(
        ARCHIVE_DECODE,
        vec![
            ApplicationQueryWhereFilterOperand::from_untrusted(
                ARCHIVE_DECODE,
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
                ARCHIVE_DECODE,
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
            ARCHIVE_DECODE,
            WorthQueryPortableApplicationQueryRelatedPredicateParts {
                relation: "BlockedBy".into(),
                from: "Task".into(),
                to: "Task".into(),
                direction: RelationDirection::Forward,
                polarity: ApplicationQueryRelatedPolarity::Absent,
                target_predicates: vec![
                    ApplicationQueryRelatedTargetPredicate::from_untrusted_value(
                        ARCHIVE_DECODE,
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
    ErasedApplicationQueryDefinition::from_untrusted_parts(ARCHIVE_DECODE, parts)
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
    let only_related =
        ErasedApplicationQueryDefinition::from_untrusted_parts(ARCHIVE_DECODE, only_related_parts);
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
        ARCHIVE_DECODE,
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
            ARCHIVE_DECODE,
            WorthQueryPortableApplicationQueryRelatedPredicateParts {
                relation: "BlockedBy".into(),
                from: "Task".into(),
                to: "Task".into(),
                direction: RelationDirection::Forward,
                polarity: ApplicationQueryRelatedPolarity::Absent,
                target_predicates: vec![
                    ApplicationQueryRelatedTargetPredicate::from_untrusted_value(
                        ARCHIVE_DECODE,
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
            &ErasedApplicationQueryDefinition::from_untrusted_parts(ARCHIVE_DECODE, oversized)
        )
        .unwrap_err()
        .kind(),
        Kind::InvalidRecordShape
    );

    // Authoring/archive encode parity: wrong-endpoint, wrong-target-field,
    // optional-equality parameter, contradictory clauses, and orbit shapes
    // all refuse at encode before emitting wire bytes.
    let encode_err = |parts: WorthQueryPortableApplicationQueryParts| {
        let mut out = BinaryOutput::with_capacity(256);
        write(
            &mut out,
            &ErasedApplicationQueryDefinition::from_untrusted_parts(ARCHIVE_DECODE, parts),
        )
        .unwrap_err()
        .kind()
    };

    let mut wrong_endpoint = plain_query().into_parts();
    wrong_endpoint.related_predicates = vec![
        ApplicationQueryRelatedPredicateMeaning::from_untrusted_parts(
            ARCHIVE_DECODE,
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
    assert_eq!(encode_err(wrong_endpoint), Kind::InvalidRecordShape);

    let mut contra = plain_query().into_parts();
    let absent_one = ApplicationQueryRelatedPredicateMeaning::from_untrusted_parts(
        ARCHIVE_DECODE,
        WorthQueryPortableApplicationQueryRelatedPredicateParts {
            relation: "BlockedBy".into(),
            from: "Task".into(),
            to: "Task".into(),
            direction: RelationDirection::Forward,
            polarity: ApplicationQueryRelatedPolarity::Absent,
            target_predicates: vec![],
            target_where_filter: None,
        },
    );
    let exists_one = ApplicationQueryRelatedPredicateMeaning::from_untrusted_parts(
        ARCHIVE_DECODE,
        WorthQueryPortableApplicationQueryRelatedPredicateParts {
            relation: "BlockedBy".into(),
            from: "Task".into(),
            to: "Task".into(),
            direction: RelationDirection::Forward,
            polarity: ApplicationQueryRelatedPolarity::Exists,
            target_predicates: vec![],
            target_where_filter: None,
        },
    );
    contra.related_predicates = vec![absent_one, exists_one];
    assert_eq!(encode_err(contra), Kind::InvalidRecordShape);

    // Raw wire forgery tests proving decode parity independently of encode:
    // 1. Splice a raw WQRP trailer with wrong root endpoint onto plain query bytes.
    let mut forged_wrong_endpoint = plain.clone();
    {
        let mut raw = BinaryOutput::with_capacity(128);
        raw.raw_bytes(b"WQRP");
        raw.u16(1);
        raw.u32(1);
        raw.text("Contains");
        raw.text("Project");
        raw.text("Task");
        raw.raw_bytes(&[0, 0]); // Forward, Absent
        raw.u32(0); // 0 target predicates
        raw.raw_bytes(&[0]); // no target filter
        forged_wrong_endpoint.extend_from_slice(&raw.into_bytes());
    }
    assert_eq!(kind(&forged_wrong_endpoint), Kind::InvalidRecordShape);

    // 2. Splice a valid WQRP trailer onto an orbit-selected query's wire bytes:
    //    decode must refuse definition-level unsupported shape!
    let mut orbit_parts = plain_query().into_parts();
    orbit_parts.orbit = Some(ApplicationQueryOrbitMeaning::from_untrusted_parts(
        ARCHIVE_DECODE,
        WorthQueryPortableApplicationQueryOrbitParts {
            query: orbit_parts.query_type.clone(),
            slot: WorthQueryPortableTypeIdentity::declared("worth.tests.archive.orbit.v1"),
            output_name: "orbit".into(),
            relation: "BlockedBy".into(),
            from: "Task".into(),
            to: "Task".into(),
        },
    ));
    let mut forged_orbit_with_related = encode(
        &ErasedApplicationQueryDefinition::from_untrusted_parts(ARCHIVE_DECODE, orbit_parts),
    );
    forged_orbit_with_related.extend_from_slice(&bytes[plain.len()..]);
    assert_eq!(kind(&forged_orbit_with_related), Kind::InvalidRecordShape);

    // 3. Raw forged WQRP trailer with contradictory Absent + Exists on identical matcher.
    let mut forged_absent_exists = plain.clone();
    {
        let mut raw = BinaryOutput::with_capacity(128);
        raw.raw_bytes(b"WQRP");
        raw.u16(1);
        raw.u32(2);
        for pol in [0_u8, 1_u8] {
            raw.text("BlockedBy");
            raw.text("Task");
            raw.text("Task");
            raw.raw_bytes(&[0, pol]); // Forward, Absent then Exists
            raw.u32(0); // 0 target predicates
            raw.raw_bytes(&[0]); // no target filter
        }
        forged_absent_exists.extend_from_slice(&raw.into_bytes());
    }
    assert_eq!(kind(&forged_absent_exists), Kind::InvalidRecordShape);

    // 4. Raw forged WQRP trailer with unequal literal equalities on same target field.
    let mut forged_unequal_literals = plain.clone();
    {
        let mut raw = BinaryOutput::with_capacity(256);
        raw.raw_bytes(b"WQRP");
        raw.u16(1);
        raw.u32(1);
        raw.text("BlockedBy");
        raw.text("Task");
        raw.text("Task");
        raw.raw_bytes(&[0, 0]); // Forward, Absent
        raw.u32(2); // 2 target predicates on Task.TaskFacts.Status in canonical order
        for literal in ["done", "open"] {
            raw.text("Task");
            raw.text("TaskFacts");
            raw.text("Status");
            crate::record::foundational_aspect::write_scalar_type(
                &mut raw,
                ScalarAspectType::String,
            )
            .unwrap();
            raw.raw_bytes(&[0, 1]); // Required, Value
            crate::record::foundational_value::write_aspect_value(
                &mut raw,
                &AspectValue::String(InternedString::Raw(literal.into())),
            )
            .unwrap();
        }
        raw.raw_bytes(&[0]); // no target filter
        forged_unequal_literals.extend_from_slice(&raw.into_bytes());
    }
    assert_eq!(kind(&forged_unequal_literals), Kind::InvalidRecordShape);
}
