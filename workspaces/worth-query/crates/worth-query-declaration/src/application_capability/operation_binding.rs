//! Retained operation identity for capability transitions.

use crate::application_schema::ApplicationOperationRef;
use crate::portable_identity::WorthQueryPortableTypeIdentity;

mod portable_parts;
pub use portable_parts::WorthQueryPortableApplicationCapabilityOperationBindingParts;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityOperationBinding {
    operation: String,
    operation_identity: WorthQueryPortableTypeIdentity,
    input_identity: WorthQueryPortableTypeIdentity,
}

impl ApplicationCapabilityOperationBinding {
    pub fn from_reference<Schema, Operation, Input>(
        operation: ApplicationOperationRef<Schema, Operation, Input>,
    ) -> Self {
        Self {
            operation: operation.name().to_string(),
            operation_identity: WorthQueryPortableTypeIdentity::declared(operation.name()),
            input_identity: operation.input_identity(),
        }
    }

    pub fn operation(&self) -> &str {
        &self.operation
    }

    pub fn operation_type(&self) -> &str {
        self.operation_identity.as_str()
    }

    pub fn operation_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.operation_identity.clone()
    }

    pub fn input_type(&self) -> &str {
        self.input_identity.as_str()
    }

    pub fn input_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.input_identity.clone()
    }
}
