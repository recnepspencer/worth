use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
use worth_query_declaration::facade::authoring::{
    AspectFieldKey, DeliveredFieldName, OrderingDirection, QueryFamily, RelationName,
    ResultShapeFamily, RootEntityKey, WorthQueryPredicateOperand,
};
use worth_query_declaration::facade::binding::{
    IdentityBindingDescriptor, QueryBindingSlot, QueryBindingSubject,
};
use worth_query_declaration::facade::canonicalization::{
    CanonicalOrderingEntry, CanonicalPredicateEntry, CanonicalPredicateFamily,
    CanonicalPredicateOperand, CanonicalProjectionEntry, CanonicalResultField, CanonicalScalarSet,
    CanonicalTraversalEntry, WorthQueryPortableCanonicalQueryRecord,
    WorthQueryPortableCanonicalResultShapeRecord,
};
use worth_query_declaration::facade::identity::{CanonicalQueryDigest, CanonicalResultShapeDigest};

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};
use crate::record::decode_budget::RecordDecodeAttempt;
use crate::record::foundational_value::{decode_aspect_value, write_aspect_value};
use crate::record::sequence::{decode_sequence, require_canonical_sequence, write_sequence};

pub(super) struct DecodedQuery {
    pub(super) digest: CanonicalQueryDigest,
    pub(super) family: QueryFamily,
    pub(super) root: RootEntityKey,
    pub(super) projection: Vec<CanonicalProjectionEntry>,
    pub(super) predicates: Vec<CanonicalPredicateEntry>,
    pub(super) ordering: Vec<CanonicalOrderingEntry>,
    pub(super) traversal: Vec<CanonicalTraversalEntry>,
    pub(super) identity_bindings: Vec<IdentityBindingDescriptor>,
}

pub(super) struct DecodedResultShape {
    pub(super) digest: CanonicalResultShapeDigest,
    pub(super) family: ResultShapeFamily,
    pub(super) fields: Vec<CanonicalResultField>,
}

pub(super) fn write_query(
    output: &mut dyn BinaryEncodingSink,
    query: &WorthQueryPortableCanonicalQueryRecord,
) -> Result<(), Denial> {
    output.text(query.digest().as_str())?;
    output.u16(query_family_tag(query.family()))?;
    output.text(query.root().as_str())?;
    write_sequence(output, query.projection(), |output, entry| {
        write_field_key(output, &entry.field)
    })?;
    write_sequence(output, query.predicates(), |output, entry| {
        write_predicate(output, entry)
    })?;
    write_sequence(output, query.ordering(), |output, entry| {
        write_field_key(output, &entry.field)?;
        output.u16(match entry.direction {
            OrderingDirection::Ascending => 1,
            OrderingDirection::Descending => 2,
        })
    })?;
    write_sequence(output, query.traversal(), |output, entry| {
        output.text(entry.relation.as_str())?;
        output.u8(entry.depth)
    })?;
    write_sequence(output, query.identity_bindings(), |output, binding| {
        output.text(binding.slot().as_str())?;
        output.u16(match binding.subject() {
            QueryBindingSubject::RootEntity => 1,
            QueryBindingSubject::TraversalRoot => 2,
        })
    })
}

pub(super) fn decode_query(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<DecodedQuery, Denial> {
    Ok(DecodedQuery {
        digest: CanonicalQueryDigest::from_untrusted(ARCHIVE_DECODE, input.text()?.to_owned()),
        family: query_family_from_tag(input.u16()?)?,
        root: RootEntityKey::new(input.text()?.to_owned())
            .map_err(|_| Denial::new(Kind::InvalidRecordShape))?,
        projection: decode_sequence(input, budget, 12, |input, _| {
            Ok(CanonicalProjectionEntry {
                field: decode_field_key(input)?,
            })
        })?,
        predicates: decode_sequence(input, budget, 16, decode_predicate)?,
        ordering: decode_sequence(input, budget, 14, |input, _| {
            Ok(CanonicalOrderingEntry {
                field: decode_field_key(input)?,
                direction: match input.u16()? {
                    1 => OrderingDirection::Ascending,
                    2 => OrderingDirection::Descending,
                    _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
                },
            })
        })?,
        traversal: decode_sequence(input, budget, 7, |input, _| {
            Ok(CanonicalTraversalEntry {
                relation: RelationName::new(input.text()?.to_owned())
                    .map_err(|_| Denial::new(Kind::InvalidRecordShape))?,
                depth: input.u8()?,
            })
        })?,
        identity_bindings: decode_sequence(input, budget, 6, |input, _| {
            Ok(IdentityBindingDescriptor::new(
                QueryBindingSlot::new(input.text()?.to_owned())
                    .map_err(|_| Denial::new(Kind::InvalidRecordShape))?,
                match input.u16()? {
                    1 => QueryBindingSubject::RootEntity,
                    2 => QueryBindingSubject::TraversalRoot,
                    _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
                },
            ))
        })?,
    })
}

pub(super) fn write_result_shape(
    output: &mut dyn BinaryEncodingSink,
    shape: &WorthQueryPortableCanonicalResultShapeRecord,
) -> Result<(), Denial> {
    output.text(shape.digest().as_str())?;
    output.u16(result_shape_family_tag(shape.family()))?;
    write_sequence(output, shape.fields(), |output, field| {
        write_field_key(output, &field.source)?;
        output.text(field.delivered_name.as_str())
    })
}

pub(super) fn decode_result_shape(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<DecodedResultShape, Denial> {
    Ok(DecodedResultShape {
        digest: CanonicalResultShapeDigest::from_untrusted(
            ARCHIVE_DECODE,
            input.text()?.to_owned(),
        ),
        family: result_shape_family_from_tag(input.u16()?)?,
        fields: decode_sequence(input, budget, 16, |input, _| {
            Ok(CanonicalResultField {
                source: decode_field_key(input)?,
                delivered_name: DeliveredFieldName::new(input.text()?.to_owned())
                    .map_err(|_| Denial::new(Kind::InvalidRecordShape))?,
            })
        })?,
    })
}

fn write_predicate(
    output: &mut dyn BinaryEncodingSink,
    entry: &CanonicalPredicateEntry,
) -> Result<(), Denial> {
    write_field_key(output, &entry.field)?;
    output.u16(match entry.family {
        CanonicalPredicateFamily::Equality => 1,
        CanonicalPredicateFamily::NativeGreaterThan => 2,
        CanonicalPredicateFamily::NativeLessThan => 3,
        CanonicalPredicateFamily::StringContains => 4,
        CanonicalPredicateFamily::ScalarMembership => 5,
        CanonicalPredicateFamily::PresenceIsPresent => 6,
    })?;
    match &entry.operand {
        CanonicalPredicateOperand::Scalar(value) => {
            output.u16(1)?;
            write_aspect_value(output, value.as_native())
        }
        CanonicalPredicateOperand::ScalarSet(values) => {
            output.u16(2)?;
            write_sequence(output, values.as_slice(), |output, value| {
                write_aspect_value(output, value.as_native())
            })
        }
        CanonicalPredicateOperand::Presence("is-present") => output.u16(3),
        CanonicalPredicateOperand::Presence(_) => Err(Denial::new(Kind::InvalidRecordShape)),
    }
}

fn decode_predicate(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<CanonicalPredicateEntry, Denial> {
    let field = decode_field_key(input)?;
    let family = match input.u16()? {
        1 => CanonicalPredicateFamily::Equality,
        2 => CanonicalPredicateFamily::NativeGreaterThan,
        3 => CanonicalPredicateFamily::NativeLessThan,
        4 => CanonicalPredicateFamily::StringContains,
        5 => CanonicalPredicateFamily::ScalarMembership,
        6 => CanonicalPredicateFamily::PresenceIsPresent,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    let operand = match input.u16()? {
        1 => CanonicalPredicateOperand::Scalar(WorthQueryPredicateOperand::native(
            decode_aspect_value(input)?,
        )),
        2 => {
            let values = decode_sequence(input, budget, 2, |input, _| {
                Ok(WorthQueryPredicateOperand::native(decode_aspect_value(
                    input,
                )?))
            })?;
            require_canonical_sequence(&values)?;
            CanonicalPredicateOperand::ScalarSet(CanonicalScalarSet::new(values))
        }
        3 => CanonicalPredicateOperand::Presence("is-present"),
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    Ok(CanonicalPredicateEntry {
        field,
        family,
        operand,
    })
}

fn write_field_key(
    output: &mut dyn BinaryEncodingSink,
    field: &AspectFieldKey,
) -> Result<(), Denial> {
    output.text(field.aspect().as_str())?;
    output.text(field.field().as_str())
}

fn decode_field_key(input: &mut BinaryInput<'_>) -> Result<AspectFieldKey, Denial> {
    AspectFieldKey::from_authoring_parts(input.text()?.to_owned(), input.text()?.to_owned())
        .map_err(|_| Denial::new(Kind::InvalidRecordShape))
}

const fn query_family_tag(family: &QueryFamily) -> u16 {
    match family {
        QueryFamily::Detail => 1,
        QueryFamily::Collection => 2,
    }
}

fn query_family_from_tag(tag: u16) -> Result<QueryFamily, Denial> {
    match tag {
        1 => Ok(QueryFamily::Detail),
        2 => Ok(QueryFamily::Collection),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}

const fn result_shape_family_tag(family: &ResultShapeFamily) -> u16 {
    match family {
        ResultShapeFamily::Detail => 1,
        ResultShapeFamily::Collection => 2,
    }
}

fn result_shape_family_from_tag(tag: u16) -> Result<ResultShapeFamily, Denial> {
    match tag {
        1 => Ok(ResultShapeFamily::Detail),
        2 => Ok(ResultShapeFamily::Collection),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}
