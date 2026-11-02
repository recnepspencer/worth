use std::collections::BTreeMap;

use crate::authoring::AspectFieldKey;
use crate::canonicalization::{
    CanonicalPredicateEntry, CanonicalPredicateFamily, CanonicalPredicateOperand,
};
use crate::schema_view::{QuerySchemaView, ScalarAspectType};
use worth_foundational::facade::{AspectValue, InternedString};

use super::predicate_state::FieldPredicateState;
use super::{
    failure::ValidationFailureArtifact, QueryValidationCounters, QueryValidationError,
    ValidatedPredicateEntry, ValidationEvent, ValidationRejectionMatrix,
};

type LegalPredicate = (CanonicalPredicateEntry, ScalarAspectType, &'static str);

pub fn validate_predicate_entries(
    predicates: &[CanonicalPredicateEntry],
    schema_view: &QuerySchemaView,
    counters: &mut QueryValidationCounters,
    rejection_matrix: &mut ValidationRejectionMatrix,
) -> Result<(Vec<ValidatedPredicateEntry>, Vec<ValidationEvent>), ValidationFailureArtifact> {
    let mut legal_predicates = Vec::new();
    let mut events = Vec::new();

    for predicate in predicates {
        counters.record_schema_lookup();
        let key = predicate.field_key();
        let Some(field) = schema_view.field(key.aspect(), key.field()) else {
            counters.record_rejection();
            rejection_matrix.record_predicate_rejection();
            return Err(ValidationFailureArtifact::new(
                QueryValidationError::UnknownField {
                    aspect: key.aspect().to_string(),
                    field: key.field().to_string(),
                },
                counters.clone(),
                rejection_matrix.clone(),
            ));
        };

        let predicate_family = predicate_family_name(predicate.family);
        let value_kind = operand_kind_name(&predicate.operand);

        if field.is_workflow_semantic() && !field.is_workflow_predicate_queryable() {
            counters.record_rejection();
            rejection_matrix.record_predicate_rejection();
            return Err(ValidationFailureArtifact::new(
                QueryValidationError::IllegalWorkflowPredicateCapabilityOrContextShape {
                    aspect: key.aspect().to_string(),
                    field: key.field().to_string(),
                    predicate_family,
                },
                counters.clone(),
                rejection_matrix.clone(),
            ));
        }

        if !predicate_capability_is_admitted(predicate, field) {
            counters.record_rejection();
            rejection_matrix.record_predicate_rejection();
            return Err(ValidationFailureArtifact::new(
                QueryValidationError::IncompatiblePredicateFamily {
                    aspect: key.aspect().to_string(),
                    field: key.field().to_string(),
                    predicate_family,
                    field_kind: field_kind_name(field.kind()),
                },
                counters.clone(),
                rejection_matrix.clone(),
            ));
        }

        if !predicate_operand_matches_field_kind(predicate, field.kind()) {
            counters.record_rejection();
            rejection_matrix.record_predicate_rejection();
            return Err(ValidationFailureArtifact::new(
                QueryValidationError::IncompatiblePredicateFamily {
                    aspect: key.aspect().to_string(),
                    field: key.field().to_string(),
                    predicate_family,
                    field_kind: field_kind_name(field.kind()),
                },
                counters.clone(),
                rejection_matrix.clone(),
            ));
        }

        counters.record_predicate_validated();
        legal_predicates.push((predicate.clone(), *field.kind(), value_kind));
        events.push(ValidationEvent::PredicateValidated {
            aspect: key.aspect().to_string(),
            field: key.field().to_string(),
            predicate_family,
            field_kind: format!("{:?}", field.kind()),
        });
    }

    let normalized = normalize_legal_predicates(legal_predicates, counters, rejection_matrix)?;
    counters.set_validated_predicate_count(normalized.len());

    Ok((normalized, events))
}

fn normalize_legal_predicates(
    legal_predicates: Vec<LegalPredicate>,
    counters: &mut QueryValidationCounters,
    rejection_matrix: &mut ValidationRejectionMatrix,
) -> Result<Vec<ValidatedPredicateEntry>, ValidationFailureArtifact> {
    let mut by_field: BTreeMap<AspectFieldKey, Vec<LegalPredicate>> = BTreeMap::new();

    for predicate in legal_predicates {
        by_field
            .entry(predicate.0.field_key().clone())
            .or_default()
            .push(predicate);
    }

    let mut normalized = Vec::new();

    for (field_key, entries) in by_field {
        let mut state = FieldPredicateState::default();
        for entry in entries {
            state.ingest(
                entry,
                field_key.aspect().as_str(),
                field_key.field().as_str(),
                counters,
                rejection_matrix,
            )?;
        }
        normalized.extend(state.into_validated(
            field_key.aspect().as_str(),
            field_key.field().as_str(),
            counters,
            rejection_matrix,
        )?);
    }

    normalized.sort();
    Ok(normalized)
}

fn predicate_capability_is_admitted(
    predicate: &CanonicalPredicateEntry,
    field: &crate::schema_view::SchemaFieldView,
) -> bool {
    match predicate.family {
        CanonicalPredicateFamily::StringContains => field.is_text_predicate_queryable(),
        CanonicalPredicateFamily::ScalarMembership => field.is_membership_predicate_queryable(),
        CanonicalPredicateFamily::PresenceIsPresent => field.is_presence_predicate_queryable(),
        _ => true,
    }
}

fn predicate_operand_matches_field_kind(
    predicate: &CanonicalPredicateEntry,
    field_kind: &ScalarAspectType,
) -> bool {
    let field_family = *field_kind;
    match predicate.family {
        CanonicalPredicateFamily::Equality => scalar_operand_family(&predicate.operand)
            .is_some_and(|operand_family| operand_family == field_family),
        CanonicalPredicateFamily::NativeGreaterThan | CanonicalPredicateFamily::NativeLessThan => {
            native_comparison_family(field_family)
                && scalar_operand_family(&predicate.operand)
                    .is_some_and(|operand_family| operand_family == field_family)
        }
        CanonicalPredicateFamily::StringContains => {
            field_family == ScalarAspectType::String
                && matches!(
                    &predicate.operand,
                    CanonicalPredicateOperand::Scalar(value)
                        if matches!(value.as_native(), AspectValue::String(InternedString::Raw(_)))
                )
        }
        CanonicalPredicateFamily::ScalarMembership => {
            let CanonicalPredicateOperand::ScalarSet(values) = &predicate.operand else {
                return false;
            };
            values
                .as_slice()
                .iter()
                .all(|value| value.value_family() == field_family)
        }
        CanonicalPredicateFamily::PresenceIsPresent => {
            matches!(&predicate.operand, CanonicalPredicateOperand::Presence(_))
        }
    }
}

fn scalar_operand_family(operand: &CanonicalPredicateOperand) -> Option<ScalarAspectType> {
    match operand {
        CanonicalPredicateOperand::Scalar(value) => Some(value.value_family()),
        CanonicalPredicateOperand::ScalarSet(_) | CanonicalPredicateOperand::Presence(_) => None,
    }
}

fn native_comparison_family(family: ScalarAspectType) -> bool {
    matches!(
        family,
        ScalarAspectType::Int8
            | ScalarAspectType::Int16
            | ScalarAspectType::Int32
            | ScalarAspectType::Int64
            | ScalarAspectType::UInt8
            | ScalarAspectType::UInt16
            | ScalarAspectType::UInt32
            | ScalarAspectType::UInt64
            | ScalarAspectType::Float32
            | ScalarAspectType::Float64
            | ScalarAspectType::Decimal
            | ScalarAspectType::BigInt
            | ScalarAspectType::Rational
            | ScalarAspectType::Date
            | ScalarAspectType::Time
            | ScalarAspectType::Timestamp
            | ScalarAspectType::TimestampTz
    )
}

fn predicate_family_name(family: CanonicalPredicateFamily) -> &'static str {
    match family {
        CanonicalPredicateFamily::Equality => "equality",
        CanonicalPredicateFamily::NativeGreaterThan => "native-greater-than",
        CanonicalPredicateFamily::NativeLessThan => "native-less-than",
        CanonicalPredicateFamily::StringContains => "string-contains",
        CanonicalPredicateFamily::ScalarMembership => "scalar-membership",
        CanonicalPredicateFamily::PresenceIsPresent => "presence-is-present",
    }
}

fn operand_kind_name(operand: &CanonicalPredicateOperand) -> &'static str {
    match operand {
        CanonicalPredicateOperand::Scalar(value) => native_family_name(value.value_family()),
        CanonicalPredicateOperand::ScalarSet(values) => match values.first() {
            Some(value) => native_set_family_name(value.value_family()),
            None => "Set<Empty>",
        },
        CanonicalPredicateOperand::Presence(_) => "Presence",
    }
}

fn field_kind_name(field_kind: &ScalarAspectType) -> &'static str {
    native_family_name(*field_kind)
}

fn native_family_name(family: ScalarAspectType) -> &'static str {
    match family {
        ScalarAspectType::InlineBytes => "InlineBytes",
        ScalarAspectType::Null => "Null",
        ScalarAspectType::Bool => "Bool",
        ScalarAspectType::Int8 => "Int8",
        ScalarAspectType::Int16 => "Int16",
        ScalarAspectType::Int32 => "Int32",
        ScalarAspectType::Int64 => "Int64",
        ScalarAspectType::UInt8 => "UInt8",
        ScalarAspectType::UInt16 => "UInt16",
        ScalarAspectType::UInt32 => "UInt32",
        ScalarAspectType::UInt64 => "UInt64",
        ScalarAspectType::Float32 => "Float32",
        ScalarAspectType::Float64 => "Float64",
        ScalarAspectType::Decimal => "Decimal",
        ScalarAspectType::BigInt => "BigInt",
        ScalarAspectType::Rational => "Rational",
        ScalarAspectType::String => "String",
        ScalarAspectType::Bytes => "Bytes",
        ScalarAspectType::Uuid => "Uuid",
        ScalarAspectType::Date => "Date",
        ScalarAspectType::Time => "Time",
        ScalarAspectType::Timestamp => "Timestamp",
        ScalarAspectType::TimestampTz => "TimestampTz",
        ScalarAspectType::EntityRef => "EntityRef",
        ScalarAspectType::ContentRef => "ContentRef",
    }
}

fn native_set_family_name(family: ScalarAspectType) -> &'static str {
    match family {
        ScalarAspectType::InlineBytes => "Set<InlineBytes>",
        ScalarAspectType::Null => "Set<Null>",
        ScalarAspectType::Bool => "Set<Bool>",
        ScalarAspectType::Int8 => "Set<Int8>",
        ScalarAspectType::Int16 => "Set<Int16>",
        ScalarAspectType::Int32 => "Set<Int32>",
        ScalarAspectType::Int64 => "Set<Int64>",
        ScalarAspectType::UInt8 => "Set<UInt8>",
        ScalarAspectType::UInt16 => "Set<UInt16>",
        ScalarAspectType::UInt32 => "Set<UInt32>",
        ScalarAspectType::UInt64 => "Set<UInt64>",
        ScalarAspectType::Float32 => "Set<Float32>",
        ScalarAspectType::Float64 => "Set<Float64>",
        ScalarAspectType::Decimal => "Set<Decimal>",
        ScalarAspectType::BigInt => "Set<BigInt>",
        ScalarAspectType::Rational => "Set<Rational>",
        ScalarAspectType::String => "Set<String>",
        ScalarAspectType::Bytes => "Set<Bytes>",
        ScalarAspectType::Uuid => "Set<Uuid>",
        ScalarAspectType::Date => "Set<Date>",
        ScalarAspectType::Time => "Set<Time>",
        ScalarAspectType::Timestamp => "Set<Timestamp>",
        ScalarAspectType::TimestampTz => "Set<TimestampTz>",
        ScalarAspectType::EntityRef => "Set<EntityRef>",
        ScalarAspectType::ContentRef => "Set<ContentRef>",
    }
}
