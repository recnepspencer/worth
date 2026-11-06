//! Encoding a stored row includes only SDK-owned structural columns.
use super::AspectDefinition;
use crate::application_schema::{
    fragment::{Record, RecordRefusal, RecordRefusalKind, Shape},
    ApplicationScalarValueBinding, ApplicationValue, BoolApplicationValueBinding,
};
use std::{any::TypeId, collections::BTreeMap};
impl AspectDefinition {
    pub fn encode<Set: Record<T>, T: 'static>(
        &self,
        value: &T,
    ) -> Result<BTreeMap<String, ApplicationValue>, RecordRefusal> {
        if self.binding().binding_set() != TypeId::of::<Set>()
            || self.binding().value_type() != TypeId::of::<T>()
        {
            return Err(RecordRefusal::new(
                self.path(),
                RecordRefusalKind::WrongValueType,
            ));
        }
        let mut encoded = Set::record().encode(value, self.path())?;
        for field in self
            .fields()
            .iter()
            .filter(|field| field.internal_presence())
        {
            let path = field.path();
            let value = BoolApplicationValueBinding::encode(&true)
                .map_err(|cause| RecordRefusal::new(path, RecordRefusalKind::Encode(cause)))?;
            encoded.insert(path.to_owned(), value);
        }
        Ok(encoded)
    }
}
