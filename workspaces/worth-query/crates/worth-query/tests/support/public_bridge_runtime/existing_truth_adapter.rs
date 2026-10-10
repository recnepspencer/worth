use worth_foundational::facade::AspectValue;
use worth_query::facade::runtime::{
    WorthQueryAuthoredAspectMutation, WorthQueryExistingTruthAssertionDenial,
    WorthQueryExistingTruthAssertionDenialKind, WorthQueryExistingTruthProbeDenial,
    WorthQueryExistingTruthProbeDenialKind, WorthQueryExistingTruthProbeField,
    WorthQueryExistingTruthProbeRequest, WorthQueryExistingTruthTargetBinding,
};

use super::state::PublicExistingTruthKey;
use super::SharedRuntimeState;

pub(super) struct PublicExistingTruthVerificationAdapter {
    state: SharedRuntimeState,
}

impl PublicExistingTruthVerificationAdapter {
    pub(super) fn new(state: SharedRuntimeState) -> Self {
        Self { state }
    }
}

impl worth_query::facade::runtime::WorthQueryRuntimeExistingTruthVerificationAdapter
    for PublicExistingTruthVerificationAdapter
{
    fn verify_existing_truth_assertion(
        &self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: &WorthQueryExistingTruthTargetBinding,
        aspects: &[WorthQueryAuthoredAspectMutation],
    ) -> Result<(), WorthQueryExistingTruthAssertionDenial> {
        let state = self.state.borrow();
        for aspect in aspects {
            let aspect_touch = aspect.aspect_touch();
            let key = PublicExistingTruthKey::new(binding, aspect_touch.clone());
            let Some(expected) = aspect.foundational_value() else {
                continue;
            };
            let Some(found) = state.existing_truth_values.get(&key) else {
                return Err(WorthQueryExistingTruthAssertionDenial::new(
                    binding,
                    WorthQueryExistingTruthAssertionDenialKind::MissingAssertedAspect,
                    Some(aspect_touch.clone()),
                    Some(terminal_digest_from_aspect_value(expected)),
                    None,
                    "public bridge verification state did not contain the asserted aspect",
                ));
            };
            if found != expected {
                return Err(WorthQueryExistingTruthAssertionDenial::new(
                    binding,
                    WorthQueryExistingTruthAssertionDenialKind::AssertedValueMismatch,
                    Some(aspect_touch.clone()),
                    Some(terminal_digest_from_aspect_value(expected)),
                    Some(terminal_digest_from_aspect_value(found)),
                    "public bridge verification state did not match the asserted value",
                ));
            }
        }
        Ok(())
    }

    fn probe_existing_truth(
        &self,
        request: &WorthQueryExistingTruthProbeRequest,
    ) -> Result<Vec<WorthQueryExistingTruthProbeField>, WorthQueryExistingTruthProbeDenial> {
        let state = self.state.borrow();
        let mut fields = Vec::with_capacity(request.aspect_touches().len());
        for aspect_touch in request.aspect_touches() {
            let key = PublicExistingTruthKey::new(request.binding(), aspect_touch.clone());
            let Some(value) = state.existing_truth_values.get(&key) else {
                return Err(WorthQueryExistingTruthProbeDenial::new(
                    request.binding(),
                    WorthQueryExistingTruthProbeDenialKind::MissingProbedAspect,
                    Some(aspect_touch.clone()),
                    "public bridge verification state did not contain the probed aspect",
                ));
            };
            fields.push(
                WorthQueryExistingTruthProbeField::from_admitted_aspect_touch(
                    aspect_touch.clone(),
                    value.clone(),
                ),
            );
        }
        Ok(fields)
    }
}

fn terminal_digest_from_aspect_value(value: &AspectValue) -> String {
    match value {
        AspectValue::Null => "null".to_string(),
        AspectValue::Bool(value) => format!("bool:{value}"),
        AspectValue::Int8(value) => format!("i8:{value}"),
        AspectValue::Int16(value) => format!("i16:{value}"),
        AspectValue::Int32(value) => format!("i32:{value}"),
        AspectValue::Int64(value) => format!("i64:{value}"),
        AspectValue::UInt8(value) => format!("u8:{value}"),
        AspectValue::UInt16(value) => format!("u16:{value}"),
        AspectValue::UInt32(value) => format!("u32:{value}"),
        AspectValue::UInt64(value) => format!("u64:{value}"),
        AspectValue::Float32(value) => format!("f32-bits:{}", value.bits()),
        AspectValue::Float64(value) => format!("f64-bits:{}", value.bits()),
        AspectValue::String(value) => match value {
            worth_foundational::facade::InternedString::Raw(value) => {
                format!("string:{}:{value}", value.len())
            }
            worth_foundational::facade::InternedString::Symbol(symbol) => {
                format!("symbol:{}", symbol.0)
            }
        },
        other => format!("{other:?}"),
    }
}
