//! A binding set owns each value's representation, independently of its aspects.

use super::super::{ApplicationReadableScalarValueBinding, ApplicationValue, ValueDecodeAdmission};
use super::{Columns, RecordRefusal, ValueColumns};
use std::collections::BTreeMap;
use std::marker::PhantomData;

/// Supplies the original owner context to checked scalar and row decoding.
pub trait BindingSet: Sized + 'static {
    type Work<'a>: ?Sized;
    fn with_admission<Output>(
        work: &mut Self::Work<'_>,
        use_admission: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> Output,
    ) -> Output;
}

pub trait Scalar<T: 'static>: BindingSet {
    type Binding: ApplicationReadableScalarValueBinding<Value = T>;
    const WRITABLE: bool = true;
    fn content_format() -> Option<super::super::ApplicationContentBindingRecipe> {
        None
    }
}

pub trait Record<T: 'static>: BindingSet {
    fn record() -> impl Shape<T, Self>;
}

/// A record emits only ordinary flat scalar fields. Constructors receive the
/// original decode admission, including its owner's cancellation controls.
pub trait Shape<T, Bindings: BindingSet>: private::Sealed {
    #[doc(hidden)]
    #[inline(always)]
    fn is_scalar(&self) -> bool {
        false
    }
    #[doc(hidden)]
    fn bindings(&self) -> Vec<super::RecordBinding>;
    #[doc(hidden)]
    fn columns(&self, prefix: &str) -> Result<Vec<Column>, RecordRefusal>;
    #[doc(hidden)]
    fn columns_field(&self, prefix: &str) -> Result<Vec<Column>, RecordRefusal> {
        self.columns(prefix)
    }
    #[doc(hidden)]
    fn encode_field(
        &self,
        value: &T,
        prefix: &str,
    ) -> Result<BTreeMap<String, ApplicationValue>, RecordRefusal> {
        self.encode(value, prefix)
    }
    #[doc(hidden)]
    fn decode_field(
        &self,
        values: &BTreeMap<String, ApplicationValue>,
        prefix: &str,
        work: &mut Bindings::Work<'_>,
        admission: &mut DecodeContext<'_, '_>,
    ) -> Result<T, RecordRefusal> {
        self.decode_fields(values, prefix, work, admission)
    }
    fn encode(
        &self,
        value: &T,
        prefix: &str,
    ) -> Result<BTreeMap<String, ApplicationValue>, RecordRefusal>;
    fn decode(
        &self,
        values: &BTreeMap<String, ApplicationValue>,
        prefix: &str,
        work: &mut Bindings::Work<'_>,
    ) -> Result<T, RecordRefusal> {
        self.decode_checked(values, prefix, work, &mut DecodeContext { control: None })
    }
    /// Native callers lend their original admission separately from domain work.
    fn decode_admitted(
        &self,
        values: &BTreeMap<String, ApplicationValue>,
        prefix: &str,
        work: &mut Bindings::Work<'_>,
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<T, RecordRefusal> {
        self.decode_checked(
            values,
            prefix,
            work,
            &mut DecodeContext {
                control: Some(admission),
            },
        )
    }
    #[doc(hidden)]
    fn decode_checked(
        &self,
        values: &BTreeMap<String, ApplicationValue>,
        prefix: &str,
        work: &mut Bindings::Work<'_>,
        admission: &mut DecodeContext<'_, '_>,
    ) -> Result<T, RecordRefusal> {
        let columns = self.columns(prefix)?;
        if let Some(path) = values
            .keys()
            .find(|path| !columns.iter().any(|column| column.path == **path))
        {
            return Err(RecordRefusal::new(
                path,
                super::RecordRefusalKind::UnexpectedField,
            ));
        }
        self.decode_fields(values, prefix, work, admission)
    }
    #[doc(hidden)]
    fn decode_fields(
        &self,
        values: &BTreeMap<String, ApplicationValue>,
        prefix: &str,
        work: &mut Bindings::Work<'_>,
        admission: &mut DecodeContext<'_, '_>,
    ) -> Result<T, RecordRefusal>;
}

#[doc(hidden)]
pub struct DecodeContext<'loan, 'control> {
    pub(crate) control: Option<&'loan mut ValueDecodeAdmission<'control>>,
}

#[derive(Clone, Debug)]
pub struct Column {
    pub(crate) content: Option<super::super::ApplicationContentBindingRecipe>,
    pub(crate) path: String,
    pub(crate) recipe: fn(
        super::super::ApplicationFieldBindingLocus,
    ) -> super::super::ApplicationFieldBindingRecipe,
    pub(crate) presence: super::super::ApplicationFieldPresence,
    pub(crate) writable: bool,
    pub(crate) equality: bool,
    pub(crate) unique: bool,
}

pub fn field<Row, Value, Bindings, Kind>(
    name: &'static str,
    get: impl Fn(&Row) -> Value,
) -> Fields<Row, Bindings, (Accessor<Value, impl Fn(&Row) -> Value>,), (Kind,)>
where
    Bindings: ValueColumns<Value, Kind>,
{
    Fields {
        fields: (Accessor {
            name,
            get,
            read_only: false,
            equality_queryable: false,
            unique: false,
            marker: PhantomData,
        },),
        marker: PhantomData,
    }
}

pub struct Accessor<Value, Get> {
    pub(crate) name: &'static str,
    pub(crate) get: Get,
    pub(crate) read_only: bool,
    pub(crate) equality_queryable: bool,
    pub(crate) unique: bool,
    pub(crate) marker: PhantomData<fn() -> Value>,
}

pub struct Fields<Row, Bindings, Accessors, Kinds> {
    pub(crate) fields: Accessors,
    pub(crate) marker: PhantomData<fn() -> (Row, Bindings, Kinds)>,
}

impl<Row, Bindings, Accessors, Kinds> Fields<Row, Bindings, Accessors, Kinds>
where
    Bindings: BindingSet,
    Accessors: Columns<Row, Bindings, Kinds>,
{
    pub fn build<Constructor>(
        self,
        construct: Constructor,
    ) -> RecordShape<Row, Bindings, Accessors, Kinds, Constructor>
    where
        Constructor: Construct<Row, Accessors::Values, Bindings>,
    {
        RecordShape {
            fields: self.fields,
            construct,
            marker: PhantomData,
        }
    }
}

pub struct RecordShape<Row, Bindings, Accessors, Kinds, Constructor> {
    fields: Accessors,
    construct: Constructor,
    marker: PhantomData<fn() -> (Row, Bindings, Kinds)>,
}

pub trait Construct<Row, Values, Set: BindingSet> {
    fn construct(
        &self,
        values: Values,
        work: &mut Set::Work<'_>,
    ) -> Result<Row, Box<dyn std::error::Error + Send + Sync>>;
}

impl<Row, Bindings, Accessors, Kinds, Constructor> private::Sealed
    for RecordShape<Row, Bindings, Accessors, Kinds, Constructor>
{
}

impl<Row, Bindings, Accessors, Kinds, Constructor> Shape<Row, Bindings>
    for RecordShape<Row, Bindings, Accessors, Kinds, Constructor>
where
    Accessors: Columns<Row, Bindings, Kinds>,
    Constructor: Construct<Row, Accessors::Values, Bindings>,
    Bindings: Record<Row>,
    Row: 'static,
{
    fn bindings(&self) -> Vec<super::RecordBinding> {
        let mut bindings = vec![super::RecordBinding::record::<Bindings, Row>()];
        self.fields.bindings(&mut bindings);
        bindings
    }
    fn columns(&self, prefix: &str) -> Result<Vec<Column>, RecordRefusal> {
        let mut columns = Vec::new();
        self.fields.columns(prefix, &mut columns)?;
        let mut names = std::collections::BTreeSet::new();
        for column in &columns {
            if !names.insert(&column.path) {
                return Err(RecordRefusal::new(
                    &column.path,
                    super::RecordRefusalKind::DuplicateField,
                ));
            }
        }
        Ok(columns)
    }

    fn encode(
        &self,
        value: &Row,
        prefix: &str,
    ) -> Result<BTreeMap<String, ApplicationValue>, RecordRefusal> {
        self.columns(prefix)?;
        let mut encoded = BTreeMap::new();
        self.fields.encode(value, prefix, &mut encoded)?;
        Ok(encoded)
    }

    fn decode_fields(
        &self,
        values: &BTreeMap<String, ApplicationValue>,
        prefix: &str,
        work: &mut Bindings::Work<'_>,
        admission: &mut DecodeContext<'_, '_>,
    ) -> Result<Row, RecordRefusal> {
        let values = self.fields.decode(values, prefix, work, admission)?;
        self.construct.construct(values, work).map_err(|cause| {
            RecordRefusal::new(prefix, super::RecordRefusalKind::Constructor(cause))
        })
    }
}

pub(super) mod private {
    pub trait Sealed {}
}
