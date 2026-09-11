use std::marker::PhantomData;

use worth_foundational::facade::AspectValue;

use super::{ApplicationScalarValueBinding, ApplicationValueEncodeDenial};

/// A scalar value validated and encoded by one exact application binding.
pub struct ApplicationEncodedScalarValue<Binding: ApplicationScalarValueBinding> {
    encoded: AspectValue,
    _binding: PhantomData<fn() -> Binding>,
}

impl<Binding: ApplicationScalarValueBinding> ApplicationEncodedScalarValue<Binding> {
    pub fn try_new(value: Binding::Value) -> Result<Self, ApplicationValueEncodeDenial> {
        Binding::validate(&value)?;
        Ok(Self {
            encoded: Binding::encode(&value)?,
            _binding: PhantomData,
        })
    }

    pub const fn as_foundational_value(&self) -> &AspectValue {
        &self.encoded
    }

    pub fn into_foundational_value(self) -> AspectValue {
        self.encoded
    }
}
