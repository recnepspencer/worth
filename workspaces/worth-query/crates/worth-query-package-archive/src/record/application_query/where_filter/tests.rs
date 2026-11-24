//! Filtered queries round-trip; unfiltered bytes and malformed drafts are exact.
use crate::binary_input::BinaryInput;
use crate::binary_output::BinaryOutput;
use crate::denial::WorthQueryPackageArchiveDenialKind as Kind;
use crate::record::application_query::{decode, write};
use crate::record::decode_budget::RecordDecodeAttempt;
use worth_foundational::expression_api::expressions;
use worth_foundational::facade::{
    AspectValue, CanonicalDate, InternedString, ScalarAspectType, Symbol,
};
use worth_query_declaration::facade::application_query::*;
use worth_query_declaration::facade::application_schema::*;
use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;
use worth_query_declaration::*;

struct Schema;
struct Task;
struct Row;
worth_query_portable_type!(Row => "worth.tests.archive.where-filter-row.v1");
worth_query_structured_value_binding!(ParametersBinding for () { identity: "worth.rust.unit" });
worth_query_structured_value_binding!(RowsBinding for Row { identity: "worth.tests.archive.where-filter-row.v1" });
worth_query_application_query!(Query for Schema, identity "worth.tests.archive.where-filter.v1",
    parameters ParametersBinding, result RowsBinding, scope Task => "Task", name "where_filter");

fn unfiltered() -> ErasedApplicationQueryDefinition {
    let task = ApplicationEntityRef::<Schema, Task>::from_schema_identifier("Task");
    let mut parts = ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(task)
        .scope(task)
        .result_shape(
            ApplicationQueryResultShapeBuilder::<Schema, Query, Task, Row, RowsBinding>::new(task)
                .build(),
        )
        .cardinality(ApplicationQueryCardinality::OptionalOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .into_erased()
        .into_parts();
    parts.parameters = vec![ApplicationQueryParameterDefinition::from_untrusted_fields(
        "max_due".to_owned(),
        ScalarAspectType::Date,
        WorthQueryPortableTypeIdentity::declared("worth.tests.archive.date.v1"),
        None,
        None,
    )];
    ErasedApplicationQueryDefinition::from_untrusted_parts(parts)
}

fn filter() -> ApplicationQueryWhereFilterProgram {
    let operands = vec![
        ApplicationQueryWhereFilterOperand::from_untrusted(
            "due".to_owned(),
            ApplicationQueryWhereFilterSource::Field {
                entity: "Task".to_owned(),
                aspect: "Schedule".to_owned(),
                field: "due".to_owned(),
                scalar_family: ScalarAspectType::Date,
                presence: ApplicationFieldPresence::Optional,
            },
        ),
        ApplicationQueryWhereFilterOperand::from_untrusted(
            "max_due".to_owned(),
            ApplicationQueryWhereFilterSource::Parameter {
                parameter: "max_due".to_owned(),
                scalar_family: ScalarAspectType::Date,
            },
        ),
    ];
    let draft = expressions()
        .parse("is_some(due) && unwrap(due) <= max_due")
        .unwrap()
        .encode();
    ApplicationQueryWhereFilterProgram::from_untrusted(operands, &draft).unwrap()
}

fn filtered() -> ErasedApplicationQueryDefinition {
    let mut parts = unfiltered().into_parts();
    parts.where_filter = Some(filter());
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
fn a_filtered_query_round_trips_and_the_unfiltered_prefix_is_byte_identical() {
    let plain = encode(&unfiltered());
    let with_filter = encode(&filtered());
    assert!(
        with_filter.starts_with(&plain),
        "the extension only appends; every unfiltered byte is unchanged"
    );
    assert_eq!(&with_filter[plain.len()..plain.len() + 4], b"WQWF");
    assert!(!plain.windows(4).any(|window| window == b"WQWF"));
    assert_eq!(decode_bytes(&plain).unwrap(), (unfiltered(), true));
    let (decoded, finished) = decode_bytes(&with_filter).unwrap();
    assert!(finished);
    assert_eq!(decoded, filtered());
    assert_eq!(decoded.canonical_basis(), filtered().canonical_basis());
}

#[test]
fn malformed_filter_extensions_refuse_with_typed_archive_denials() {
    let plain_len = encode(&unfiltered()).len();
    let bytes = encode(&filtered());
    let kind = |bytes: &[u8]| decode_bytes(bytes).unwrap_err().kind();
    let mut version = bytes.clone();
    version[plain_len + 5] = 9;
    assert_eq!(kind(&version), Kind::UnsupportedRecordVersion);
    assert_eq!(kind(&bytes[..bytes.len() - 1]), Kind::Truncated);
    let mut draft = bytes.clone();
    let last = draft.len() - 1;
    draft[last] ^= 0xff;
    assert_eq!(kind(&draft), Kind::InvalidRecordShape);
    let mut variant = bytes;
    // marker(4) + version(2) + operand count(4) + first name text, then its kind tag.
    let name_end = plain_len + 10 + 4 + "due".len();
    variant[name_end] = 7;
    assert_eq!(kind(&variant), Kind::UnsupportedRecordVariant);
}

#[test]
fn optional_parameter_preserves_its_presence_and_filter_identity_through_the_archive() {
    let mut parts = unfiltered().into_parts();
    parts.parameters[0] = parts.parameters[0].clone().with_optional_presence();
    let draft = expressions().parse("!is_some(max_due)").unwrap().encode();
    parts.where_filter = Some(
        ApplicationQueryWhereFilterProgram::from_untrusted(
            vec![ApplicationQueryWhereFilterOperand::from_untrusted(
                "max_due".to_owned(),
                ApplicationQueryWhereFilterSource::OptionalParameter {
                    parameter: "max_due".to_owned(),
                    scalar_family: ScalarAspectType::Date,
                },
            )],
            &draft,
        )
        .unwrap(),
    );
    let optional = ErasedApplicationQueryDefinition::from_untrusted_parts(parts);
    assert_ne!(optional.canonical_basis(), filtered().canonical_basis());
    assert_eq!(decode_bytes(&encode(&optional)).unwrap(), (optional, true));
}

#[test]
fn literal_operands_and_value_predicates_round_trip_and_refuse_malformed_payloads() {
    let mut parts = unfiltered().into_parts();
    parts.parameters.clear();
    parts.predicates = vec![ApplicationQueryPredicate::from_untrusted_value(
        "Task".into(),
        "Schedule".into(),
        "status".into(),
        AspectValue::String(InternedString::Raw("open".into())),
        ScalarAspectType::String,
    )];
    let cutoff = CanonicalDate {
        days_from_unix_epoch: 150,
    };
    let draft = expressions()
        .parse("is_some(due) && unwrap(due) <= cutoff")
        .unwrap()
        .encode();
    parts.where_filter = Some(
        ApplicationQueryWhereFilterProgram::from_untrusted(
            vec![
                ApplicationQueryWhereFilterOperand::from_untrusted(
                    "due".into(),
                    ApplicationQueryWhereFilterSource::Field {
                        entity: "Task".into(),
                        aspect: "Schedule".into(),
                        field: "due".into(),
                        scalar_family: ScalarAspectType::Date,
                        presence: ApplicationFieldPresence::Optional,
                    },
                ),
                ApplicationQueryWhereFilterOperand::from_untrusted(
                    "cutoff".into(),
                    ApplicationQueryWhereFilterSource::Literal {
                        scalar_family: ScalarAspectType::Date,
                        value: AspectValue::Date(cutoff),
                    },
                ),
            ],
            &draft,
        )
        .unwrap(),
    );
    let literal_query = ErasedApplicationQueryDefinition::from_untrusted_parts(parts);
    let bytes = encode(&literal_query);
    let (decoded, finished) = decode_bytes(&bytes).unwrap();
    assert!(finished);
    assert_eq!(decoded, literal_query);
    assert_eq!(decoded.canonical_basis(), literal_query.canonical_basis());
    assert_ne!(decoded.canonical_basis(), filtered().canonical_basis());

    // Forged empty parameter name on a parameter predicate refuses encoding before bytes.
    let mut bad_param_pred = unfiltered().into_parts();
    bad_param_pred.predicates = vec![ApplicationQueryPredicate::from_untrusted_fields(
        "Task".into(),
        "Schedule".into(),
        "status".into(),
        String::new(),
        ScalarAspectType::String,
    )];
    let mut out = BinaryOutput::with_capacity(256);
    assert_eq!(
        write(
            &mut out,
            &ErasedApplicationQueryDefinition::from_untrusted_parts(bad_param_pred)
        )
        .unwrap_err()
        .kind(),
        Kind::InvalidRecordShape
    );

    // Optional Null parameter refuses archive decode with InvalidRecordShape.
    let mut opt_null = unfiltered().into_parts();
    opt_null.parameters[0] = ApplicationQueryParameterDefinition::from_untrusted_fields(
        "bad".into(),
        ScalarAspectType::Null,
        WorthQueryPortableTypeIdentity::declared("worth.tests.archive.null.v1"),
        None,
        None,
    )
    .with_optional_presence();
    let opt_null_bytes = encode(&ErasedApplicationQueryDefinition::from_untrusted_parts(
        opt_null,
    ));
    assert_eq!(
        decode_bytes(&opt_null_bytes).unwrap_err().kind(),
        Kind::InvalidRecordShape
    );

    // Mismatched literal predicate (Date scalar tag, Int64 value) refuses decode.
    let mut bad_lit_pred = unfiltered().into_parts();
    bad_lit_pred.predicates = vec![ApplicationQueryPredicate::from_untrusted_value(
        "Task".into(),
        "Schedule".into(),
        "due".into(),
        AspectValue::Int64(150),
        ScalarAspectType::Date,
    )];
    let bad_lit_bytes = encode(&ErasedApplicationQueryDefinition::from_untrusted_parts(
        bad_lit_pred,
    ));
    assert_eq!(
        decode_bytes(&bad_lit_bytes).unwrap_err().kind(),
        Kind::InvalidRecordShape
    );

    // Non-portable InternedString::Symbol literal predicate refuses decode.
    let mut sym_pred = unfiltered().into_parts();
    sym_pred.predicates = vec![ApplicationQueryPredicate::from_untrusted_value(
        "Task".into(),
        "Schedule".into(),
        "status".into(),
        AspectValue::String(InternedString::Symbol(Symbol(7))),
        ScalarAspectType::String,
    )];
    let sym_bytes = encode(&ErasedApplicationQueryDefinition::from_untrusted_parts(
        sym_pred,
    ));
    assert_eq!(
        decode_bytes(&sym_bytes).unwrap_err().kind(),
        Kind::InvalidRecordShape
    );

    // Over-limit (> 64 KiB) structured literal predicate refuses decode.
    let mut oversized_pred = unfiltered().into_parts();
    oversized_pred.predicates = vec![ApplicationQueryPredicate::from_untrusted_value(
        "Task".into(),
        "Schedule".into(),
        "status".into(),
        AspectValue::String(InternedString::Raw("a".repeat(64 * 1024 + 1))),
        ScalarAspectType::String,
    )];
    let oversized_bytes = encode(&ErasedApplicationQueryDefinition::from_untrusted_parts(
        oversized_pred,
    ));
    assert_eq!(
        decode_bytes(&oversized_bytes).unwrap_err().kind(),
        Kind::InvalidRecordShape
    );

    // Noncanonical native value (out-of-range CanonicalTime) refuses decode.
    let mut noncanonical_pred = unfiltered().into_parts();
    noncanonical_pred.predicates = vec![ApplicationQueryPredicate::from_untrusted_value(
        "Task".into(),
        "Schedule".into(),
        "at".into(),
        AspectValue::Time(worth_foundational::facade::CanonicalTime {
            nanos_since_midnight: worth_foundational::facade::CanonicalTime::NANOS_PER_DAY,
        }),
        ScalarAspectType::Time,
    )];
    let noncanonical_bytes = encode(&ErasedApplicationQueryDefinition::from_untrusted_parts(
        noncanonical_pred,
    ));
    assert_eq!(
        decode_bytes(&noncanonical_bytes).unwrap_err().kind(),
        Kind::InvalidRecordShape
    );

    // OptionalParameter operand against a required parameter declaration refuses decode.
    let mut opt_operand_for_req = unfiltered().into_parts();
    let opt_draft = expressions().parse("!is_some(max_due)").unwrap().encode();
    opt_operand_for_req.where_filter = Some(
        ApplicationQueryWhereFilterProgram::from_untrusted(
            vec![ApplicationQueryWhereFilterOperand::from_untrusted(
                "max_due".to_owned(),
                ApplicationQueryWhereFilterSource::OptionalParameter {
                    parameter: "max_due".to_owned(),
                    scalar_family: ScalarAspectType::Date,
                },
            )],
            &opt_draft,
        )
        .unwrap(),
    );
    let opt_for_req_bytes = encode(&ErasedApplicationQueryDefinition::from_untrusted_parts(
        opt_operand_for_req,
    ));
    assert_eq!(
        decode_bytes(&opt_for_req_bytes).unwrap_err().kind(),
        Kind::InvalidRecordShape
    );

    // Required Parameter operand against an optional parameter declaration refuses decode.
    let mut req_operand_for_opt = filtered().into_parts();
    req_operand_for_opt.parameters[0] = req_operand_for_opt.parameters[0]
        .clone()
        .with_optional_presence();
    let req_for_opt_bytes = encode(&ErasedApplicationQueryDefinition::from_untrusted_parts(
        req_operand_for_opt,
    ));
    assert_eq!(
        decode_bytes(&req_for_opt_bytes).unwrap_err().kind(),
        Kind::InvalidRecordShape
    );
}
