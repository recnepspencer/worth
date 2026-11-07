//! Optional scalar values use Native's existing optional flat columns.
use super::record::{Column, DecodeContext};
use super::record_value::{ScalarValue, ValueColumns};
use super::{RecordBinding, RecordRefusal, Scalar};
use crate::application_schema::{ApplicationFieldPresence, ApplicationValue};
use std::collections::BTreeMap;

#[doc(hidden)]
pub struct OptionalScalarColumns;
impl<Set: Scalar<T>, T: 'static> ValueColumns<Option<T>, OptionalScalarColumns> for Set {
    fn bindings(output: &mut Vec<RecordBinding>) {
        <Set as ScalarValue<T>>::bindings(output);
    }
    fn columns(
        path: &str,
        equality_queryable: bool,
        output: &mut Vec<Column>,
    ) -> Result<(), RecordRefusal> {
        let first = output.len();
        <Set as ScalarValue<T>>::columns(path, output)?;
        for column in &mut output[first..] {
            column.presence = ApplicationFieldPresence::Optional;
            column.equality = equality_queryable;
        }
        Ok(())
    }
    fn encode(
        value: &Option<T>,
        path: &str,
        output: &mut BTreeMap<String, ApplicationValue>,
    ) -> Result<(), RecordRefusal> {
        if let Some(value) = value {
            <Set as ScalarValue<T>>::encode(value, path, output)?;
        }
        Ok(())
    }
    fn decode(
        values: &BTreeMap<String, ApplicationValue>,
        path: &str,
        work: &mut Self::Work<'_>,
        admission: &mut DecodeContext<'_, '_>,
    ) -> Result<Option<T>, RecordRefusal> {
        if !values.contains_key(path) {
            return Ok(None);
        }
        <Set as ScalarValue<T>>::decode(values, path, work, admission).map(Some)
    }
}
