use crate::archive_decode_authority::ArchiveDecodeAuthority;
use std::marker::PhantomData;

use worth_foundational::facade::{AspectValue, ScalarAspectType};

use crate::application_schema::{ApplicationScalarValueBinding, ApplicationValueEncodeDenial};

pub struct ApplicationQueryParameterRef<Query, Parameter, Binding, const OPTIONAL: bool = false> {
    name: &'static str,
    _marker: PhantomData<fn() -> (Query, Parameter, Binding)>,
}

impl<Query, Parameter, Binding> ApplicationQueryParameterRef<Query, Parameter, Binding> {
    #[doc(hidden)]
    pub const fn from_query_identifier(name: &'static str) -> Self {
        Self {
            name,
            _marker: PhantomData,
        }
    }
}

impl<Query, Parameter, Binding> ApplicationQueryParameterRef<Query, Parameter, Binding, true> {
    /// Selects an optional parameter in a query definition and its request bindings.
    pub const fn optional(name: &'static str) -> Self {
        Self {
            name,
            _marker: PhantomData,
        }
    }
}

impl<Query, Parameter, Binding, const OPTIONAL: bool>
    ApplicationQueryParameterRef<Query, Parameter, Binding, OPTIONAL>
{
    pub const fn name(self) -> &'static str {
        self.name
    }
}

impl<Query, Parameter, Binding, const OPTIONAL: bool> Clone
    for ApplicationQueryParameterRef<Query, Parameter, Binding, OPTIONAL>
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<Query, Parameter, Binding, const OPTIONAL: bool> Copy
    for ApplicationQueryParameterRef<Query, Parameter, Binding, OPTIONAL>
{
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationQueryParameterDefinition {
    name: String,
    scalar_family: ScalarAspectType,
    value_type: crate::portable_identity::WorthQueryPortableTypeIdentity,
    unit: Option<String>,
    frame: Option<String>,
    optional: bool,
}

impl ApplicationQueryParameterDefinition {
    pub fn from_untrusted_fields(
        _: ArchiveDecodeAuthority,
        name: String,
        scalar_family: ScalarAspectType,
        value_type: crate::portable_identity::WorthQueryPortableTypeIdentity,
        unit: Option<String>,
        frame: Option<String>,
    ) -> Self {
        Self {
            name,
            scalar_family,
            value_type,
            unit,
            frame,
            optional: false,
        }
    }

    /// Records optional presence on this portable description. Installation
    /// still validates the complete definition against its use sites.
    pub fn with_optional_presence(mut self) -> Self {
        self.optional = true;
        self
    }

    pub const fn is_optional(&self) -> bool {
        self.optional
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn scalar_family(&self) -> ScalarAspectType {
        self.scalar_family
    }

    pub const fn value_type(&self) -> &str {
        self.value_type.as_str()
    }

    pub fn unit(&self) -> Option<&str> {
        self.unit.as_deref()
    }

    pub fn frame(&self) -> Option<&str> {
        self.frame.as_deref()
    }

    pub(super) fn typed<Query, Parameter, Binding, const OPTIONAL: bool>(
        parameter: ApplicationQueryParameterRef<Query, Parameter, Binding, OPTIONAL>,
    ) -> Self
    where
        Binding: ApplicationScalarValueBinding,
    {
        Self {
            name: parameter.name().to_owned(),
            scalar_family: Binding::SCALAR_FAMILY,
            value_type: Binding::IDENTITY,
            unit: Binding::UNIT.map(|unit| unit.as_str().to_owned()),
            frame: Binding::FRAME.map(|frame| frame.as_str().to_owned()),
            optional: OPTIONAL,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ApplicationQueryParameterSet<Query> {
    bindings: Vec<(&'static str, AspectValue)>,
    _query: PhantomData<fn() -> Query>,
}

impl<Query> Clone for ApplicationQueryParameterSet<Query> {
    fn clone(&self) -> Self {
        Self {
            bindings: self.bindings.clone(),
            _query: PhantomData,
        }
    }
}

impl<Query> ApplicationQueryParameterSet<Query> {
    pub fn new() -> Self {
        Self {
            bindings: Vec::new(),
            _query: PhantomData,
        }
    }

    pub fn bind<Parameter, Binding>(
        mut self,
        parameter: ApplicationQueryParameterRef<Query, Parameter, Binding>,
        value: Binding::Value,
    ) -> Result<Self, ApplicationValueEncodeDenial>
    where
        Binding: ApplicationScalarValueBinding,
    {
        let encoded = encode_non_null::<Binding>(
            &value,
            "a required query parameter must encode a non-null scalar value",
        )?;
        self.bindings.push((parameter.name(), encoded));
        Ok(self)
    }

    pub fn bindings(&self) -> &[(&'static str, AspectValue)] {
        &self.bindings
    }

    /// Binds either a native scalar or an explicit absence. Every declared
    /// parameter still has one entry in the admitted request.
    pub fn bind_optional<Parameter, Binding>(
        mut self,
        parameter: ApplicationQueryParameterRef<Query, Parameter, Binding, true>,
        value: Option<Binding::Value>,
    ) -> Result<Self, ApplicationValueEncodeDenial>
    where
        Binding: ApplicationScalarValueBinding,
    {
        let encoded = match value {
            Some(value) => encode_non_null::<Binding>(
                &value,
                "a present optional query parameter must encode a scalar value",
            )?,
            None => AspectValue::Null,
        };
        self.bindings.push((parameter.name(), encoded));
        Ok(self)
    }

    /// Actual owned backing, for an owner admitting an opaque parameter producer.
    pub fn owned_allocation_capacity_bytes(&self) -> Option<usize> {
        self.bindings.iter().try_fold(
            self.bindings
                .capacity()
                .checked_mul(std::mem::size_of::<(&'static str, AspectValue)>())?,
            |bytes, (_, value)| bytes.checked_add(value.owned_allocation_capacity_bytes()),
        )
    }
}

impl<Query> Default for ApplicationQueryParameterSet<Query> {
    fn default() -> Self {
        Self::new()
    }
}

fn encode_non_null<Binding: ApplicationScalarValueBinding>(
    value: &Binding::Value,
    null_reason: &'static str,
) -> Result<AspectValue, ApplicationValueEncodeDenial> {
    Binding::validate(value)?;
    let encoded = Binding::encode(value)?;
    if matches!(encoded, AspectValue::Null) {
        return Err(ApplicationValueEncodeDenial::CodecRejected {
            binding_identity: Binding::IDENTITY,
            reason: null_reason,
        });
    }
    Ok(encoded)
}
