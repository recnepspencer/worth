use std::marker::PhantomData;

use worth_foundational::facade::{AspectValue, ScalarAspectType};

use crate::application_schema::{ApplicationScalarValueBinding, ApplicationValueEncodeDenial};

pub struct ApplicationQueryParameterRef<Query, Parameter, Binding> {
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

    pub const fn name(self) -> &'static str {
        self.name
    }
}

impl<Query, Parameter, Binding> Clone for ApplicationQueryParameterRef<Query, Parameter, Binding> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Query, Parameter, Binding> Copy for ApplicationQueryParameterRef<Query, Parameter, Binding> {}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationQueryParameterDefinition {
    name: String,
    scalar_family: ScalarAspectType,
    value_type: crate::portable_identity::WorthQueryPortableTypeIdentity,
}

impl ApplicationQueryParameterDefinition {
    pub fn from_untrusted_fields(
        name: String,
        scalar_family: ScalarAspectType,
        value_type: crate::portable_identity::WorthQueryPortableTypeIdentity,
    ) -> Self {
        Self {
            name,
            scalar_family,
            value_type,
        }
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

    pub(super) fn typed<Query, Parameter, Binding>(
        parameter: ApplicationQueryParameterRef<Query, Parameter, Binding>,
    ) -> Self
    where
        Binding: ApplicationScalarValueBinding,
    {
        Self {
            name: parameter.name().to_owned(),
            scalar_family: Binding::SCALAR_FAMILY,
            value_type: Binding::IDENTITY,
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
        Binding::validate(&value)?;
        self.bindings
            .push((parameter.name(), Binding::encode(&value)?));
        Ok(self)
    }

    pub fn bindings(&self) -> &[(&'static str, AspectValue)] {
        &self.bindings
    }
}

impl<Query> Default for ApplicationQueryParameterSet<Query> {
    fn default() -> Self {
        Self::new()
    }
}
