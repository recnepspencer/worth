//! Sealed test doors over this module's private members.
use super::*;

impl InstalledProducerOutputEquivalence {
    #[doc(hidden)]
    pub fn field_for_test(&self) -> &FieldExpectation {
        &self.field
    }
}

impl InstalledProducerOutputEquivalence {
    #[doc(hidden)]
    pub fn meaning_for_test(&self) -> &[u8; 32] {
        &self.meaning
    }
}

impl<Schema, Role> WorthQueryProducerOutputEquivalence<Schema, Role> {
    #[doc(hidden)]
    pub fn into_field_for_test(self) -> NativeFieldBindingExpectation {
        Self::into_field(self)
    }
}
