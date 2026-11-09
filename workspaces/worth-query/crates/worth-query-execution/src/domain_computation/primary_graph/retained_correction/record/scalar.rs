//! Recorded scalars use their existing portable representation without coercion.
use super::*;
fn scalar(value: &AspectValue) -> Result<(), Denial> {
    use worth_foundational::facade::InternedString;
    let portable = match value {
        AspectValue::String(InternedString::Symbol(_)) => false,
        AspectValue::Null
        | AspectValue::Bool(_)
        | AspectValue::Int8(_)
        | AspectValue::Int16(_)
        | AspectValue::Int32(_)
        | AspectValue::Int64(_)
        | AspectValue::UInt8(_)
        | AspectValue::UInt16(_)
        | AspectValue::UInt32(_)
        | AspectValue::UInt64(_)
        | AspectValue::Float32(_)
        | AspectValue::Float64(_)
        | AspectValue::Decimal(_)
        | AspectValue::BigInt(_)
        | AspectValue::Rational(_)
        | AspectValue::String(InternedString::Raw(_))
        | AspectValue::Uuid(_)
        | AspectValue::Date(_)
        | AspectValue::Time(_)
        | AspectValue::Timestamp(_)
        | AspectValue::TimestampTz(_)
        | AspectValue::EntityRef(_)
        | AspectValue::ContentRef(_)
        | AspectValue::InlineBytes(_) => value.has_canonical_representation(),
    };
    if portable {
        Ok(())
    } else {
        Err(Denial::NonPortableValue)
    }
}
fn location(location: &RecordedLocation) -> Result<(), Denial> {
    if location.aspect.is_empty()
        || location.fields.is_empty()
        || location.fields.iter().any(String::is_empty)
    {
        return Err(Denial::MalformedFrame);
    }
    Ok(())
}
pub(super) fn validate(record: &MaterialRecord) -> Result<(), Denial> {
    match record {
        MaterialRecord::Capture { material } => {
            if material.mutation.is_empty()
                || material.intent.is_empty()
                || material.input.is_empty()
            {
                return Err(Denial::MalformedFrame);
            }
            if let Some(scope) = &material.scope {
                location(&scope.location)?;
                scalar(&scope.value)?;
                if scope.subject != material.scope_entity {
                    return Err(Denial::OriginalMismatch);
                }
            }
            if material.disclosure.is_some() && material.scope.is_none() {
                return Err(Denial::MalformedFrame);
            }
            if let Some(disclosure) = &material.disclosure {
                let mut names = std::collections::BTreeSet::new();
                for (name, value) in &disclosure.parameters {
                    if name.is_empty() || !names.insert(name) {
                        return Err(Denial::MalformedFrame);
                    }
                    scalar(value)?;
                }
            }
            if let Some(fields) = &material.preimage {
                let mut loci = std::collections::BTreeSet::new();
                for field in fields {
                    if field.entity.is_empty()
                        || !loci.insert((&field.entity, &field.aspect, &field.field))
                        || field.location.aspect != field.aspect
                        || (field.location.fields.len() != 1
                            || field.location.fields.first().map(String::as_str)
                                != Some(field.field.as_str()))
                    {
                        return Err(Denial::MalformedFrame);
                    }
                    location(&field.location)?;
                    scalar(&field.value)?;
                }
            }
        }
        MaterialRecord::Ready { material } => {
            let mut roles = std::collections::BTreeSet::new();
            for role in &material.roles {
                if role.role.is_empty()
                    || role.entity_name.is_empty()
                    || role.posture > 2
                    || !roles.insert(&role.role)
                {
                    return Err(Denial::MalformedFrame);
                }
            }
            let mut fields = std::collections::BTreeSet::new();
            for field in &material.output_fields {
                if !material.roles.iter().any(|role| {
                    role.role == field.role
                        && role.entity_name == field.entity_name
                        && role.entity == field.entity
                }) || !fields.insert((
                    &field.role,
                    &field.location.aspect,
                    &field.location.fields,
                )) {
                    return Err(Denial::MalformedFrame);
                }
                location(&field.location)?;
                scalar(&field.value)?;
            }
        }
        MaterialRecord::Pin {
            prepared, class, ..
        } => {
            if *class != RetentionClass::MandatoryObligation {
                return Err(Denial::RetentionClassMismatch);
            }
            if prepared.action.is_empty()
                || prepared.input.is_empty()
                || prepared.target.branch.is_empty()
            {
                return Err(Denial::MalformedFrame);
            }
            location(&prepared.scope.location)?;
            scalar(&prepared.scope.value)?;
        }
        MaterialRecord::Transfer { class, .. } => {
            if *class != RetentionClass::MandatoryObligation {
                return Err(Denial::RetentionClassMismatch);
            }
        }
        MaterialRecord::Release { class, .. }
        | MaterialRecord::DischargePreparation { class, .. } => {
            if *class != RetentionClass::OptionalRetainedMaterial {
                return Err(Denial::RetentionClassMismatch);
            }
        }
        MaterialRecord::Abandon { .. } | MaterialRecord::Expire { .. } => {}
    }
    Ok(())
}
