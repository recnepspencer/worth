//! Closed storage damage retains the original contract so reads see actual corruption.
use super::*;
use crate::aspects::structs::FieldKey;

impl ContractValidatedAspectValue {
    #[doc(hidden)]
    pub fn corrupt_struct_field_for_test(
        &mut self,
        field: &FieldKey,
        replacement: Option<AspectValue>,
    ) -> bool {
        let ContractValidatedAspectValueKind::Struct(value) =
            &mut std::sync::Arc::make_mut(&mut self.value).kind
        else {
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
        *value = StructAspectValue::new(fields).expect("original field keys remain unique");
        true
    }
}
