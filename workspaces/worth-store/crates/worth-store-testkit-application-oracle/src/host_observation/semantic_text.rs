//! Scalar meaning is independent of the Native interner's process-local slots.
use serde_json::{Map, Value};
use worth_foundational::facade::{
    AspectValue, AuthoritativeRecordAspectState, ContractValidatedAspectValueView, InternedString,
};
use worth_relational::facade::symbols::SymbolTableSnapshot;
pub(super) fn text(value: &InternedString, symbols: &SymbolTableSnapshot) -> String {
    match value {
        InternedString::Raw(value) => value.clone(),
        InternedString::Symbol(symbol) => symbols
            .entries
            .iter()
            .find(|(candidate, _)| candidate.0 == symbol.0)
            .expect("the owner snapshot resolves its exact symbol")
            .1
            .as_str()
            .to_owned(),
    }
}
fn scalar(value: &AspectValue, symbols: &SymbolTableSnapshot) -> Value {
    match value {
        AspectValue::String(value) => serde_json::to_value(AspectValue::String(
            InternedString::Raw(text(value, symbols)),
        ))
        .unwrap(),
        value @ (AspectValue::Null
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
        | AspectValue::Uuid(_)
        | AspectValue::Date(_)
        | AspectValue::Time(_)
        | AspectValue::Timestamp(_)
        | AspectValue::TimestampTz(_)
        | AspectValue::EntityRef(_)
        | AspectValue::ContentRef(_)
        | AspectValue::InlineBytes(_)) => serde_json::to_value(value).unwrap(),
    }
}
pub(super) fn state(
    state: &Option<AuthoritativeRecordAspectState>,
    symbols: &SymbolTableSnapshot,
) -> Value {
    let Some(state) = state else {
        return Value::Null;
    };
    Value::Object(
        state
            .aspects()
            .entries()
            .map(|(key, aspect)| {
                let value = match aspect.view() {
                    ContractValidatedAspectValueView::Scalar(value) => {
                        Value::Object(Map::from_iter([("Scalar".into(), scalar(value, symbols))]))
                    }
                    ContractValidatedAspectValueView::Struct(fields) => {
                        Value::Object(Map::from_iter([(
                            "Struct".into(),
                            Value::Object(
                                fields
                                    .fields()
                                    .map(|(field, value)| {
                                        (field.as_str().into(), scalar(value, symbols))
                                    })
                                    .collect(),
                            ),
                        )]))
                    }
                };
                (
                    key.as_str().into(),
                    Value::Object(Map::from_iter([
                        (
                            "identity".into(),
                            serde_json::to_value(aspect.contract_identity()).unwrap(),
                        ),
                        (
                            "revision".into(),
                            serde_json::to_value(aspect.contract_revision()).unwrap(),
                        ),
                        ("value".into(), value),
                    ])),
                )
            })
            .collect(),
    )
}
