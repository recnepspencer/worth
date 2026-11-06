//! A scalar binding supplies the one-column record form without a second impl.
use super::record::{Column, DecodeContext};
use super::record_value::ScalarValue;
use super::{BindingSet, Record, RecordBinding, RecordRefusal, Scalar, Shape};
use crate::application_schema::ApplicationValue;
use std::{collections::BTreeMap, marker::PhantomData};

struct ScalarRecord<Set, T>(PhantomData<fn() -> (Set, T)>);
impl<Set: Scalar<T>, T: 'static> Record<T> for Set {
    fn record() -> impl Shape<T, Self> {
        ScalarRecord(PhantomData)
    }
}
impl<Set, T> super::record::private::Sealed for ScalarRecord<Set, T> {}
fn path(prefix: &str) -> String {
    if prefix.is_empty() {
        "value".to_owned()
    } else {
        format!("{prefix}.value")
    }
}
impl<Set: Scalar<T>, T: 'static> Shape<T, Set> for ScalarRecord<Set, T> {
    fn bindings(&self) -> Vec<RecordBinding> {
        let mut bindings = Vec::new();
        <Set as ScalarValue<T>>::bindings(&mut bindings);
        bindings
    }
    fn columns(&self, prefix: &str) -> Result<Vec<Column>, RecordRefusal> {
        self.columns_field(&path(prefix))
    }
    fn columns_field(&self, prefix: &str) -> Result<Vec<Column>, RecordRefusal> {
        let mut columns = Vec::new();
        <Set as ScalarValue<T>>::columns(prefix, &mut columns)?;
        Ok(columns)
    }
    fn encode(
        &self,
        value: &T,
        prefix: &str,
    ) -> Result<BTreeMap<String, ApplicationValue>, RecordRefusal> {
        self.encode_field(value, &path(prefix))
    }
    fn encode_field(
        &self,
        value: &T,
        prefix: &str,
    ) -> Result<BTreeMap<String, ApplicationValue>, RecordRefusal> {
        let mut values = BTreeMap::new();
        <Set as ScalarValue<T>>::encode(value, prefix, &mut values)?;
        Ok(values)
    }
    fn decode_fields(
        &self,
        values: &BTreeMap<String, ApplicationValue>,
        prefix: &str,
        work: &mut <Set as BindingSet>::Work<'_>,
        admission: &mut DecodeContext<'_, '_>,
    ) -> Result<T, RecordRefusal> {
        self.decode_field(values, &path(prefix), work, admission)
    }
    fn decode_field(
        &self,
        values: &BTreeMap<String, ApplicationValue>,
        prefix: &str,
        work: &mut Set::Work<'_>,
        admission: &mut DecodeContext<'_, '_>,
    ) -> Result<T, RecordRefusal> {
        <Set as ScalarValue<T>>::decode(values, prefix, work, admission)
    }
}
