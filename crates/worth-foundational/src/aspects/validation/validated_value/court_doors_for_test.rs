//! Deliberate damage preserves contract meaning and invalidates validation reuse.
use super::*;
use crate::aspects::structs::FieldKey;
impl ContractValidatedAspectValue {
    #[doc(hidden)]
    pub fn corrupt_struct_field_for_test(
        &mut self,
        field: &FieldKey,
        replacement: Option<AspectValue>,
    ) -> bool {
        let ContractValidatedAspectValueKind::Struct(value) = &self.value.kind else {
            return false;
        };
        if value.get(field).is_none() {
            return false;
        }
        let fields = value.fields().filter_map(|(key, value)| {
            if key == field {
                replacement.clone().map(|value| (key.clone(), value))
            } else {
                Some((key.clone(), value.clone()))
            }
        });
        let next = StructAspectValue::new(fields).expect("original field keys remain unique");
        let current = self.contract();
        let untrusted = AspectContract::new(
            current.key().clone(),
            current.identity(),
            current.revision(),
            current.shape().clone(),
            current.masks().clone(),
            current.absence(),
            current.equivalence(),
            current.evolution(),
        );
        let inner = std::sync::Arc::make_mut(&mut self.value);
        inner.contract = untrusted;
        inner.kind = ContractValidatedAspectValueKind::Struct(next);
        true
    }
}
