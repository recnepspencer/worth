use crate::aspects::structs::StructAspectValue;
use crate::values::AspectValue;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractValidationInput {
    Scalar(AspectValue),
    Struct(StructAspectValue),
}

impl From<AspectValue> for ContractValidationInput {
    fn from(value: AspectValue) -> Self {
        Self::Scalar(value)
    }
}

impl From<StructAspectValue> for ContractValidationInput {
    fn from(value: StructAspectValue) -> Self {
        Self::Struct(value)
    }
}

impl ContractValidationInput {
    pub(crate) fn clone_retained_storage_bytes(&self) -> Option<usize> {
        match self {
            Self::Scalar(value) => value.clone_retained_storage_bytes(),
            Self::Struct(value) => value.retained_storage_bytes(),
        }
    }

    pub(crate) fn clone_allocation_bytes(&self) -> Option<usize> {
        match self {
            Self::Scalar(value) => value.clone_allocation_bytes(),
            Self::Struct(value) => value.clone_allocation_bytes(),
        }
    }

    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        match self {
            Self::Scalar(value) => value.owned_allocation_capacity_bytes(),
            Self::Struct(value) => value.owned_allocation_capacity_bytes(),
        }
    }
}
