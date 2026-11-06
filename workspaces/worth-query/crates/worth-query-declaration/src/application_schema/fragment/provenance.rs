//! Compiler-local row codecs and exact field-handle expectations.

use super::super::{
    ApplicationFieldBindingRecipe, ApplicationSchemaDeclaration,
    ApplicationSchemaDeclarationDenial, ApplicationValue,
};
use super::record_value::ScalarValue;
use super::{Record, RecordRefusal, RecordRefusalKind, Scalar, Shape};
use std::any::{Any, TypeId};
use std::collections::BTreeMap;

type Encoded = BTreeMap<String, ApplicationValue>;
type Encode = fn(&dyn Any, &str) -> Result<Encoded, RecordRefusal>;

#[derive(Clone, Debug)]
pub struct RecordBinding {
    set: TypeId,
    value: TypeId,
    value_name: &'static str,
    identity: String,
    encode: Encode,
}
impl PartialEq for RecordBinding {
    fn eq(&self, other: &Self) -> bool {
        self.set == other.set && self.value == other.value
    }
}
impl Eq for RecordBinding {}
impl RecordBinding {
    pub(crate) fn record<Set: Record<T>, T: 'static>() -> Self {
        Self {
            set: TypeId::of::<Set>(),
            value: TypeId::of::<T>(),
            value_name: std::any::type_name::<T>(),
            identity: format!(
                "{}<{}>",
                std::any::type_name::<Set>(),
                std::any::type_name::<T>()
            ),
            encode: |value, prefix| {
                Set::record().encode(
                    value.downcast_ref::<T>().ok_or_else(|| {
                        RecordRefusal::new(prefix, RecordRefusalKind::WrongValueType)
                    })?,
                    prefix,
                )
            },
        }
    }
    pub(crate) fn scalar<Set: Scalar<T>, T: 'static>() -> Self {
        Self {
            set: TypeId::of::<Set>(),
            value: TypeId::of::<T>(),
            value_name: std::any::type_name::<T>(),
            identity: format!(
                "{}<{}>",
                std::any::type_name::<Set>(),
                std::any::type_name::<T>()
            ),
            encode: |value, prefix| {
                let mut encoded = BTreeMap::new();
                <Set as ScalarValue<T>>::encode(
                    value.downcast_ref::<T>().ok_or_else(|| {
                        RecordRefusal::new(prefix, RecordRefusalKind::WrongValueType)
                    })?,
                    prefix,
                    &mut encoded,
                )?;
                Ok(encoded)
            },
        }
    }
    pub fn value_type(&self) -> TypeId {
        self.value
    }
    pub fn binding_set(&self) -> TypeId {
        self.set
    }
    pub fn value_name(&self) -> &'static str {
        self.value_name
    }
    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn encode(&self, value: &dyn Any, prefix: &str) -> Result<Encoded, RecordRefusal> {
        (self.encode)(value, prefix)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct FragmentProvenance {
    pub(crate) catalog: super::FragmentCatalog,
    bindings: Vec<RecordBinding>,
    expected_fields: Vec<ApplicationFieldBindingRecipe>,
    pub(crate) access: Vec<super::catalog::RecordAccess>,
}

impl super::super::ApplicationSchemaMemberProvenance {
    #[doc(hidden)]
    pub fn fragment_catalog(&self) -> &super::FragmentCatalog {
        &self.fragments.catalog
    }
}

impl FragmentProvenance {
    pub(crate) fn register_bindings(&mut self, bindings: Vec<RecordBinding>) {
        for binding in bindings {
            if !self.bindings.contains(&binding) {
                self.bindings.push(binding);
            }
        }
    }
    pub(crate) fn expect_field(&mut self, field: ApplicationFieldBindingRecipe) {
        self.expected_fields.push(field);
    }
    pub(crate) fn validate(
        &self,
        fields: &[ApplicationFieldBindingRecipe],
    ) -> Result<(), ApplicationSchemaDeclarationDenial> {
        for expected in &self.expected_fields {
            if self.catalog.internal_field(expected.locus())
                || !fields.iter().any(|field| {
                    field.locus() == expected.locus()
                        && field.value_type() == expected.value_type()
                        && field.binding_type() == expected.binding_type()
                })
            {
                return Err(ApplicationSchemaDeclarationDenial::InvalidRecordField {
                    field: expected.locus().clone(),
                });
            }
        }
        Ok(())
    }
}

impl<S> ApplicationSchemaDeclaration<S> {
    /// The codecs actually reached by the declared records, including nested
    /// records and scalar bindings. No separate registration list is authored.
    pub fn record_bindings(&self) -> &[RecordBinding] {
        &self.member_provenance.fragments.bindings
    }
}
