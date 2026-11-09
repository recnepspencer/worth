use serde_json::{Map, Number, Value};
use worth_foundational::{AspectValue, ContractValidatedAspectValueView, StructAspectValue};

use crate::{
    StoreAspectBoundaryFact, StoreAspectIdentity, StoreTerminalProjectionDenial,
    StoreTerminalProjectionDisplayLabel, StoreTerminalProjectionDocumentBytes,
};

#[derive(Debug, Clone, PartialEq)]
pub struct StoreTerminalJsonProjection {
    terminal_projection_identity: StoreAspectIdentity,
    terminal_projection_document: Value,
}

impl StoreTerminalJsonProjection {
    pub(crate) fn from_terminal_projection_document(
        terminal_projection_identity: StoreAspectIdentity,
        terminal_projection_document: Value,
    ) -> Self {
        Self {
            terminal_projection_identity,
            terminal_projection_document,
        }
    }

    pub fn terminal_projection_identity(&self) -> &StoreAspectIdentity {
        &self.terminal_projection_identity
    }

    pub(crate) fn into_terminal_projection_document(self) -> Value {
        self.terminal_projection_document
    }

    pub fn to_compact_terminal_json_document_bytes(
        &self,
    ) -> Result<StoreTerminalProjectionDocumentBytes, StoreTerminalProjectionDenial> {
        render_compact_terminal_json_document(&self.terminal_projection_document)
    }

    pub fn to_pretty_terminal_json_document_bytes(
        &self,
    ) -> Result<StoreTerminalProjectionDocumentBytes, StoreTerminalProjectionDenial> {
        let rendered = serde_json::to_string_pretty(&self.terminal_projection_document)
            .map_err(|_| StoreTerminalProjectionDenial::TerminalProjectionRenderingDenied)?;
        StoreTerminalProjectionDocumentBytes::from_terminal_projection_bytes(rendered.into_bytes())
    }

    pub fn to_labelled_terminal_json_document_bytes(
        &self,
        label: &StoreTerminalProjectionDisplayLabel,
    ) -> Result<StoreTerminalProjectionDocumentBytes, StoreTerminalProjectionDenial> {
        render_compact_terminal_json_document(&labelled_terminal_projection_document(
            label,
            self.terminal_projection_document.clone(),
        ))
    }
}

pub fn project_store_boundary_fact_to_terminal_json(
    fact: &StoreAspectBoundaryFact,
) -> Result<StoreTerminalJsonProjection, StoreTerminalProjectionDenial> {
    let validated_value = fact
        .authority_input()
        .admitted_state()
        .payload()
        .get(fact.identity().aspect_key())
        .ok_or(StoreTerminalProjectionDenial::MissingProjectedAspectValue)?;
    let terminal_projection_document = match validated_value.view() {
        ContractValidatedAspectValueView::Scalar(value) => aspect_value_to_terminal_json(value)?,
        ContractValidatedAspectValueView::Struct(value) => struct_value_to_terminal_json(value)?,
    };

    Ok(
        StoreTerminalJsonProjection::from_terminal_projection_document(
            fact.identity().clone(),
            terminal_projection_document,
        ),
    )
}

fn struct_value_to_terminal_json(
    value: &StructAspectValue,
) -> Result<Value, StoreTerminalProjectionDenial> {
    let mut object = Map::new();
    for (field, value) in value.fields() {
        object.insert(
            field.as_str().to_string(),
            aspect_value_to_terminal_json(value)?,
        );
    }
    Ok(Value::Object(object))
}

fn labelled_terminal_projection_document(
    label: &StoreTerminalProjectionDisplayLabel,
    terminal_projection_document: Value,
) -> Value {
    let mut envelope = Map::new();
    envelope.insert(
        "display_label".to_string(),
        Value::String(label.terminal_display_label().to_string()),
    );
    envelope.insert(
        "terminal_projection".to_string(),
        terminal_projection_document,
    );
    Value::Object(envelope)
}

fn render_compact_terminal_json_document(
    document: &Value,
) -> Result<StoreTerminalProjectionDocumentBytes, StoreTerminalProjectionDenial> {
    let rendered = serde_json::to_string(document)
        .map_err(|_| StoreTerminalProjectionDenial::TerminalProjectionRenderingDenied)?;
    StoreTerminalProjectionDocumentBytes::from_terminal_projection_bytes(rendered.into_bytes())
}

fn aspect_value_to_terminal_json(
    value: &AspectValue,
) -> Result<Value, StoreTerminalProjectionDenial> {
    match value {
        AspectValue::InlineBytes(_) => Err(
            StoreTerminalProjectionDenial::UnsupportedTerminalProjectionValue(
                "inline domain bytes require the owner's terminal projection",
            ),
        ),
        AspectValue::Null => Ok(Value::Null),
        AspectValue::Bool(value) => Ok(Value::Bool(*value)),
        AspectValue::Int8(value) => Ok(Number::from(*value).into()),
        AspectValue::Int16(value) => Ok(Number::from(*value).into()),
        AspectValue::Int32(value) => Ok(Number::from(*value).into()),
        AspectValue::Int64(value) => Ok(Number::from(*value).into()),
        AspectValue::UInt8(value) => Ok(Number::from(*value).into()),
        AspectValue::UInt16(value) => Ok(Number::from(*value).into()),
        AspectValue::UInt32(value) => Ok(Number::from(*value).into()),
        AspectValue::UInt64(value) => Ok(Number::from(*value).into()),
        AspectValue::Float32(value) => finite_float_to_json(f32::from_bits(value.bits()) as f64),
        AspectValue::Float64(value) => finite_float_to_json(f64::from_bits(value.bits())),
        AspectValue::Decimal(value) => Ok(Value::String(value.as_str().to_string())),
        AspectValue::BigInt(value) => Ok(Value::String(value.as_str().to_string())),
        AspectValue::Rational(value) => Ok(Value::String(format!(
            "{}/{}",
            value.numerator.as_str(),
            value.denominator.as_str()
        ))),
        AspectValue::String(value) => value
            .raw_text()
            .map(|text| Value::String(text.to_owned()))
            .ok_or(
                StoreTerminalProjectionDenial::UnsupportedTerminalProjectionValue(
                    "symbolic string requires explicit symbol table projection",
                ),
            ),
        AspectValue::Uuid(value) => Ok(Value::Array(
            value
                .iter()
                .map(|byte| Number::from(*byte).into())
                .collect(),
        )),
        AspectValue::Date(value) => Ok(Number::from(value.days_from_unix_epoch).into()),
        AspectValue::Time(value) => Ok(Number::from(value.nanos_since_midnight).into()),
        AspectValue::Timestamp(value) => Ok(Number::from(value.micros_since_unix_epoch).into()),
        AspectValue::TimestampTz(_) => Err(
            StoreTerminalProjectionDenial::UnsupportedTerminalProjectionValue(
                "timestamp with timezone has no current JSON readmission shape",
            ),
        ),
        AspectValue::EntityRef(value) => Ok(entity_ref_to_terminal_json(value)),
        AspectValue::ContentRef(value) => Ok(Value::String(value.to_string())),
    }
}

fn finite_float_to_json(value: f64) -> Result<Value, StoreTerminalProjectionDenial> {
    Number::from_f64(value).map(Value::Number).ok_or(
        StoreTerminalProjectionDenial::UnsupportedTerminalProjectionValue(
            "non-finite float cannot be represented by terminal JSON",
        ),
    )
}

fn entity_ref_to_terminal_json(value: &worth_foundational::EntityId) -> Value {
    let mut object = Map::new();
    object.insert(
        "partition_id".to_string(),
        Number::from(value.partition_id.0).into(),
    );
    object.insert(
        "local_slot".to_string(),
        Number::from(value.local_slot.0).into(),
    );
    object.insert(
        "generation".to_string(),
        Number::from(value.generation.0).into(),
    );
    Value::Object(object)
}

#[cfg(test)]
mod textual_projection_tests {
    use super::*;
    use worth_foundational::{InternedString, Symbol};

    #[test]
    fn decoded_stored_text_keeps_raw_text_projection_and_symbols_still_refuse() {
        let raw = InternedString::Raw("kept text".into());
        let restored: InternedString =
            serde_json::from_value(serde_json::to_value(&raw).unwrap()).unwrap();
        assert!(matches!(&restored, InternedString::Stored(_)));
        for text in [raw, restored] {
            assert_eq!(
                aspect_value_to_terminal_json(&AspectValue::String(text)).unwrap(),
                Value::String("kept text".into())
            );
        }
        assert!(matches!(
            aspect_value_to_terminal_json(&AspectValue::String(InternedString::Symbol(Symbol(7)))),
            Err(StoreTerminalProjectionDenial::UnsupportedTerminalProjectionValue(_))
        ));
    }
}
