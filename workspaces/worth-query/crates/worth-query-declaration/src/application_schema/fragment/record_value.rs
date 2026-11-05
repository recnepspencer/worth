//! Scalar, array and record lowering share the existing scalar binding owner.

use super::super::{
    ApplicationFieldBindingRecipe, ApplicationReadableScalarValueBinding,
    ApplicationScalarValueBinding, ApplicationValue,
};
use super::record::{Column, DecodeContext, Record, Scalar, Shape};
use super::{RecordRefusal, RecordRefusalKind as Denial};
use std::collections::BTreeMap;
use std::marker::PhantomData;

#[doc(hidden)]
pub struct ScalarColumns;
#[doc(hidden)]
pub struct RecordColumns;
#[doc(hidden)]
pub struct ArrayColumns<Kind>(PhantomData<Kind>);

#[doc(hidden)]
pub trait ValueColumns<T, Kind>: super::BindingSet {
    fn bindings(output: &mut Vec<super::RecordBinding>);
    fn columns(path: &str, output: &mut Vec<Column>) -> Result<(), RecordRefusal>;
    fn encode(
        value: &T,
        path: &str,
        output: &mut BTreeMap<String, ApplicationValue>,
    ) -> Result<(), RecordRefusal>;
    fn decode(
        values: &BTreeMap<String, ApplicationValue>,
        path: &str,
        work: &mut Self::Work<'_>,
        admission: &mut DecodeContext<'_, '_>,
    ) -> Result<T, RecordRefusal>;
}

impl<Set: Scalar<T>, T: 'static> ValueColumns<T, ScalarColumns> for Set {
    fn bindings(output: &mut Vec<super::RecordBinding>) {
        output.push(super::RecordBinding::scalar::<Set, T>());
    }
    fn columns(path: &str, output: &mut Vec<Column>) -> Result<(), RecordRefusal> {
        output.push(Column {
            path: path.to_owned(),
            recipe: ApplicationFieldBindingRecipe::of::<Set::Binding>,
            writable: Set::WRITABLE,
            equality: Set::EQUALITY_QUERYABLE,
            unique: false,
        });
        Ok(())
    }

    fn encode(
        value: &T,
        path: &str,
        output: &mut BTreeMap<String, ApplicationValue>,
    ) -> Result<(), RecordRefusal> {
        <Set::Binding as ApplicationScalarValueBinding>::validate(value)
            .map_err(|cause| RecordRefusal::new(path, Denial::Encode(cause.into())))?;
        let encoded = <Set::Binding as ApplicationScalarValueBinding>::encode(value)
            .map_err(|cause| RecordRefusal::new(path, Denial::Encode(cause)))?;
        output.insert(path.to_owned(), encoded);
        Ok(())
    }

    fn decode(
        values: &BTreeMap<String, ApplicationValue>,
        path: &str,
        work: &mut Self::Work<'_>,
        admission: &mut DecodeContext<'_, '_>,
    ) -> Result<T, RecordRefusal> {
        let value = values
            .get(path)
            .ok_or_else(|| RecordRefusal::new(path, Denial::MissingField))?;
        let decode = |control: &mut super::super::ValueDecodeAdmission<'_>| {
            control.admit(1, 0).map_err(|cause| {
                super::super::ApplicationValueDecodeDenial::Canonical {
                    binding_identity: Set::Binding::IDENTITY,
                    cause,
                }
            })?;
            Set::Binding::decode_admitted(value, control)
        };
        let decoded = match admission.control.as_deref_mut() {
            Some(control) => decode(control),
            None => Set::with_admission(work, decode),
        }
        .map_err(|cause| RecordRefusal::new(path, Denial::Decode(cause)))?;
        <Set::Binding as ApplicationScalarValueBinding>::validate(&decoded)
            .map_err(|cause| RecordRefusal::new(path, Denial::Decode(cause.into())))?;
        Ok(decoded)
    }
}

impl<Set: Record<T>, T: 'static> ValueColumns<T, RecordColumns> for Set {
    fn bindings(output: &mut Vec<super::RecordBinding>) {
        output.extend(Set::record().bindings());
    }
    fn columns(path: &str, output: &mut Vec<Column>) -> Result<(), RecordRefusal> {
        output.extend(Set::record().columns(path)?);
        Ok(())
    }

    fn encode(
        value: &T,
        path: &str,
        output: &mut BTreeMap<String, ApplicationValue>,
    ) -> Result<(), RecordRefusal> {
        output.extend(Set::record().encode(value, path)?);
        Ok(())
    }

    fn decode(
        values: &BTreeMap<String, ApplicationValue>,
        path: &str,
        work: &mut Self::Work<'_>,
        admission: &mut DecodeContext<'_, '_>,
    ) -> Result<T, RecordRefusal> {
        Set::record().decode_fields(values, path, work, admission)
    }
}

impl<Set, T, Kind, const N: usize> ValueColumns<[T; N], ArrayColumns<Kind>> for Set
where
    Set: ValueColumns<T, Kind>,
{
    fn bindings(output: &mut Vec<super::RecordBinding>) {
        <Set as ValueColumns<T, Kind>>::bindings(output);
    }
    fn columns(path: &str, output: &mut Vec<Column>) -> Result<(), RecordRefusal> {
        for index in 0..N {
            <Set as ValueColumns<T, Kind>>::columns(&format!("{path}[{index}]"), output)?;
        }
        Ok(())
    }

    fn encode(
        value: &[T; N],
        path: &str,
        output: &mut BTreeMap<String, ApplicationValue>,
    ) -> Result<(), RecordRefusal> {
        for (index, value) in value.iter().enumerate() {
            <Set as ValueColumns<T, Kind>>::encode(value, &format!("{path}[{index}]"), output)?;
        }
        Ok(())
    }

    fn decode(
        values: &BTreeMap<String, ApplicationValue>,
        path: &str,
        work: &mut Self::Work<'_>,
        admission: &mut DecodeContext<'_, '_>,
    ) -> Result<[T; N], RecordRefusal> {
        (0..N)
            .map(|index| {
                <Set as ValueColumns<T, Kind>>::decode(
                    values,
                    &format!("{path}[{index}]"),
                    work,
                    admission,
                )
            })
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .map_err(|_| RecordRefusal::new(path, Denial::MissingField))
    }
}
