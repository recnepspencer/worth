//! SDK structural columns are emitted once after the owning row codec.
use super::AspectDefinition;
use crate::application_schema::{
    fragment::{Record, RecordRefusal, RecordRefusalKind},
    ApplicationScalarValueBinding, ApplicationValue, BoolApplicationValueBinding,
};
use std::{
    any::{Any, TypeId},
    collections::BTreeMap,
};
impl AspectDefinition {
    #[inline(always)]
    pub fn encode<Set: Record<T>, T: 'static>(
        &self,
        value: &T,
    ) -> Result<BTreeMap<String, ApplicationValue>, RecordRefusal> {
        self.encode_record(TypeId::of::<Set>(), TypeId::of::<T>(), value)
    }
    fn encode_record(
        &self,
        set: TypeId,
        row: TypeId,
        value: &dyn Any,
    ) -> Result<BTreeMap<String, ApplicationValue>, RecordRefusal> {
        if self.binding().binding_set() != set || self.binding().value_type() != row {
            return Err(RecordRefusal::new(
                self.path(),
                RecordRefusalKind::WrongValueType,
            ));
        }
        let mut encoded = self.binding().encode(value, self.path())?;
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
