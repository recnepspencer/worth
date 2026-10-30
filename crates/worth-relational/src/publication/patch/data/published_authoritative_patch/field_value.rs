//! Descriptive positions borrow canonical values after the existing effect grammar validates.
use super::{
    PublishedAuthoritativeFieldEffect as Effect, PublishedAuthoritativePatch,
    PublishedAuthoritativePatchOperation as Operation, PublishedAuthoritativePatchValue as Value,
};
use worth_foundational::facade::{AspectBinding, AspectContract, AspectValue, FieldKey};

/// A value position, never authority. Its holder must retain the exact original patch.
#[derive(Clone, Copy, Debug)]
pub struct PublishedAuthoritativeFieldValueLocation {
    operation: usize,
    part: ValuePart,
}
#[derive(Clone, Copy, Debug)]
enum ValuePart {
    Scalar,
    StructField(usize),
    FieldSet(usize),
}
impl PublishedAuthoritativePatch {
    /// The existing effect owner decides duplicates, original contract and physical binding.
    pub fn field_effect_with_value_location<Stop>(
        &self,
        contract: &AspectContract,
        binding: &AspectBinding,
        field: &FieldKey,
        mut admit: impl FnMut(u64) -> Result<(), Stop>,
    ) -> Result<(Effect, Option<PublishedAuthoritativeFieldValueLocation>), Stop> {
        let effect = self.field_effect(contract, binding, field, &mut admit)?;
        if effect != Effect::Present {
            return Ok((effect, None));
        }
        for (operation_index, operation) in self.operations.iter().enumerate() {
            admit(1)?;
            if operation.aspect_key() != contract.key() {
                continue;
            }
            let part = match operation {
                Operation::WholeAspectSet {
                    value: Value::Scalar(_),
                    ..
                } => Some(ValuePart::Scalar),
                Operation::WholeAspectSet {
                    value: Value::Struct(value),
                    ..
                } => {
                    admit(value.len() as u64 + 1)?;
                    value
                        .fields()
                        .position(|(key, _)| key == field)
                        .map(ValuePart::StructField)
                }
                Operation::FieldLevelPatch { field_sets, .. } => {
                    admit(field_sets.len() as u64 + 1)?;
                    field_sets
                        .iter()
                        .position(|set| &set.field == field)
                        .map(ValuePart::FieldSet)
                }
                _ => None,
            };
            if let Some(part) = part {
                return Ok((
                    effect,
                    Some(PublishedAuthoritativeFieldValueLocation {
                        operation: operation_index,
                        part,
                    }),
                ));
            }
        }
        Ok((effect, None))
    }
    /// Allocation-free original lookup. Struct storage visits at most its declared width;
    /// scalar and field-set positions index their original vectors directly.
    pub fn field_value_at(
        &self,
        location: PublishedAuthoritativeFieldValueLocation,
    ) -> Option<&AspectValue> {
        match (self.operations.get(location.operation)?, location.part) {
            (
                Operation::WholeAspectSet {
                    value: Value::Scalar(value),
                    ..
                },
                ValuePart::Scalar,
            ) => Some(value),
            (
                Operation::WholeAspectSet {
                    value: Value::Struct(value),
                    ..
                },
                ValuePart::StructField(index),
            ) => value.fields().nth(index).map(|(_, value)| value),
            (Operation::FieldLevelPatch { field_sets, .. }, ValuePart::FieldSet(index)) => {
                field_sets.get(index).map(|set| &set.value)
            }
            _ => None,
        }
    }
}
