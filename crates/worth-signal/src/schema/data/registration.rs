use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

use std::fmt;

use serde::{Deserialize, Serialize};

use super::SignalSchemaDescriptor;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalSchemaRegistration {
    descriptor: SignalSchemaDescriptor,
}

impl SignalSchemaRegistration {
    pub fn new(descriptor: SignalSchemaDescriptor) -> Result<Self, SignalSchemaRegistrationError> {
        validate_non_empty("semantic_name", descriptor.semantic_name().as_str())?;
        Ok(Self { descriptor })
    }

    pub fn descriptor(&self) -> &SignalSchemaDescriptor {
        &self.descriptor
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalSchemaRegistrationError {
    field: &'static str,
}

impl fmt::Display for SignalSchemaRegistrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "signal schema registration field `{}` must not be empty",
            self.field
        )
    }
}

impl std::error::Error for SignalSchemaRegistrationError {}

fn validate_non_empty(
    field: &'static str,
    value: &str,
) -> Result<(), SignalSchemaRegistrationError> {
    if value.trim().is_empty() {
        return Err(SignalSchemaRegistrationError { field });
    }
    Ok(())
}

impl RetainedStorageMeasurement for SignalSchemaRegistration {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self { descriptor } = self;
        Ok(Charge::ZERO.checked_add(descriptor.retained_heap_charge(work)?)?)
    }
}
