//! Bounded tuple machinery pairs getters and checked constructor arguments.

use super::super::ApplicationValue;
use super::record::{Accessor, BindingSet, Column, Construct, DecodeContext, Fields};
use super::{RecordRefusal, RecordRefusalKind, ValueColumns};
use std::collections::BTreeMap;
use std::marker::PhantomData;

#[doc(hidden)]
pub trait Columns<Row, Set: BindingSet, Kinds> {
    type Values;
    fn bindings(&self, output: &mut Vec<super::RecordBinding>);
    fn columns(&self, prefix: &str, output: &mut Vec<Column>) -> Result<(), RecordRefusal>;
    fn encode(
        &self,
        row: &Row,
        prefix: &str,
        output: &mut BTreeMap<String, ApplicationValue>,
    ) -> Result<(), RecordRefusal>;
    fn decode(
        &self,
        values: &BTreeMap<String, ApplicationValue>,
        prefix: &str,
        work: &mut Set::Work<'_>,
        admission: &mut DecodeContext<'_, '_>,
    ) -> Result<Self::Values, RecordRefusal>;
}

fn path(prefix: &str, name: &str) -> Result<String, RecordRefusal> {
    if name.is_empty()
        || name
            .chars()
            .any(|c| c.is_control() || c.is_whitespace() || matches!(c, '.' | '[' | ']'))
    {
        return Err(RecordRefusal::new(
            &format!("{prefix}.{name}"),
            RecordRefusalKind::InvalidFieldName,
        ));
    }
    Ok(if prefix.is_empty() {
        name.to_owned()
    } else {
        format!("{prefix}.{name}")
    })
}

macro_rules! columns {
    ($($v:ident $get:ident $kind:ident $index:tt),+) => {
        impl<Row, Set, $($v, $get, $kind),+> Columns<Row, Set, ($($kind,)+)> for ($(Accessor<$v, $get>,)+)
        where $(Set: ValueColumns<$v, $kind>, $get: Fn(&Row) -> $v,)+ {
            type Values = ($($v,)+);
            fn bindings(&self, output: &mut Vec<super::RecordBinding>) {
                $(<Set as ValueColumns<$v, $kind>>::bindings(output);)+
            }
            fn columns(&self, prefix: &str, output: &mut Vec<Column>) -> Result<(), RecordRefusal> {
                $(let first = output.len();
                <Set as ValueColumns<$v, $kind>>::columns(&path(prefix, self.$index.name)?, output)?;
                if self.$index.unique && output.len() - first != 1 {
                    return Err(RecordRefusal::new(&path(prefix, self.$index.name)?, RecordRefusalKind::UniqueNeedsScalar));
                }
                for column in &mut output[first..] { if self.$index.read_only { column.writable = false; } column.unique |= self.$index.unique; })+
                Ok(())
            }
            fn encode(&self, row: &Row, prefix: &str, output: &mut BTreeMap<String, ApplicationValue>) -> Result<(), RecordRefusal> {
                $(<Set as ValueColumns<$v, $kind>>::encode(&(self.$index.get)(row), &path(prefix, self.$index.name)?, output)?;)+
                Ok(())
            }
            fn decode(&self, values: &BTreeMap<String, ApplicationValue>, prefix: &str, work: &mut Set::Work<'_>, admission: &mut DecodeContext<'_, '_>) -> Result<Self::Values, RecordRefusal> {
                Ok(($(<Set as ValueColumns<$v, $kind>>::decode(values, &path(prefix, self.$index.name)?, work, admission)?,)+))
            }
        }
        impl<Row, Constructor, Error, Set: BindingSet, $($v),+> Construct<Row, ($($v,)+), Set> for Constructor
        where Constructor: for<'work> Fn($($v,)+ &mut Set::Work<'work>) -> Result<Row, Error>, Error: std::error::Error + Send + Sync + 'static {
            fn construct(&self, values: ($($v,)+), work: &mut Set::Work<'_>) -> Result<Row, Box<dyn std::error::Error + Send + Sync>> {
                self($(values.$index,)+ work).map_err(|cause| Box::new(cause) as _)
            }
        }
    };
}

macro_rules! append {
    ($($v:ident $get:ident $kind:ident $index:tt),+) => {
        impl<Row, Set, $($v, $get, $kind),+> Fields<Row, Set, ($(Accessor<$v, $get>,)+), ($($kind,)+)> {
            pub fn and<Next, NextKind>(self, name: &'static str, get: impl Fn(&Row) -> Next) -> Fields<Row, Set, ($(Accessor<$v, $get>,)+ Accessor<Next, impl Fn(&Row) -> Next>), ($($kind,)+ NextKind)>
            where Set: ValueColumns<Next, NextKind> {
                Fields { fields: ($(self.fields.$index,)+ Accessor { name, get, read_only: false, unique: false, marker: PhantomData }), marker: PhantomData }
            }
        }
    };
}

macro_rules! field_posture {
    ($last:tt; $($v:ident $get:ident $kind:ident),+) => {
        impl<Row, Set, $($v, $get, $kind),+> Fields<Row, Set, ($(Accessor<$v, $get>,)+), ($($kind,)+)> {
            pub fn read_only(mut self) -> Self { self.fields.$last.read_only = true; self }
            pub fn unique(mut self) -> Self { self.fields.$last.unique = true; self }
        }
    };
}
field_posture!(0; A GA KA);
field_posture!(1; A GA KA, B GB KB);
field_posture!(2; A GA KA, B GB KB, C GC KC);
field_posture!(3; A GA KA, B GB KB, C GC KC, D GD KD);
field_posture!(4; A GA KA, B GB KB, C GC KC, D GD KD, E GE KE);
field_posture!(5; A GA KA, B GB KB, C GC KC, D GD KD, E GE KE, F GF KF);
field_posture!(6; A GA KA, B GB KB, C GC KC, D GD KD, E GE KE, F GF KF, G GG KG);
field_posture!(7; A GA KA, B GB KB, C GC KC, D GD KD, E GE KE, F GF KF, G GG KG, H GH KH);

columns!(A GA KA 0);
columns!(A GA KA 0, B GB KB 1);
columns!(A GA KA 0, B GB KB 1, C GC KC 2);
columns!(A GA KA 0, B GB KB 1, C GC KC 2, D GD KD 3);
columns!(A GA KA 0, B GB KB 1, C GC KC 2, D GD KD 3, E GE KE 4);
columns!(A GA KA 0, B GB KB 1, C GC KC 2, D GD KD 3, E GE KE 4, F GF KF 5);
columns!(A GA KA 0, B GB KB 1, C GC KC 2, D GD KD 3, E GE KE 4, F GF KF 5, G GG KG 6);
columns!(A GA KA 0, B GB KB 1, C GC KC 2, D GD KD 3, E GE KE 4, F GF KF 5, G GG KG 6, H GH KH 7);
append!(A GA KA 0);
append!(A GA KA 0, B GB KB 1);
append!(A GA KA 0, B GB KB 1, C GC KC 2);
append!(A GA KA 0, B GB KB 1, C GC KC 2, D GD KD 3);
append!(A GA KA 0, B GB KB 1, C GC KC 2, D GD KD 3, E GE KE 4);
append!(A GA KA 0, B GB KB 1, C GC KC 2, D GD KD 3, E GE KE 4, F GF KF 5);
append!(A GA KA 0, B GB KB 1, C GC KC 2, D GD KD 3, E GE KE 4, F GF KF 5, G GG KG 6);
