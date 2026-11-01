use serde::{Deserialize, Serialize};

use super::PortableAspectContractBasis;
use crate::aspects::{ContractValidationInput, FieldKey};
use crate::values::AspectValue;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortableRecordAspectPatch {
    operations: Vec<PortableAspectPatchOperation>,
}

impl PortableRecordAspectPatch {
    pub fn new(operations: impl IntoIterator<Item = PortableAspectPatchOperation>) -> Self {
        Self {
            operations: operations.into_iter().collect(),
        }
    }

    /// Checked heap of an independent Clone, including nested owned data.
    /// No source Vec/String spare capacity is copied or included.
    pub fn clone_allocation_bytes(&self) -> Option<usize> {
        let mut bytes =
            std::alloc::Layout::array::<PortableAspectPatchOperation>(self.operations.len())
                .ok()?
                .size();
        for operation in &self.operations {
            bytes = bytes.checked_add(operation.clone_allocation_bytes()?)?;
        }
        Some(bytes)
    }

    pub fn operations(&self) -> &[PortableAspectPatchOperation] {
        &self.operations
    }

    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    pub(crate) fn into_operations(self) -> Vec<PortableAspectPatchOperation> {
        self.operations
    }

    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        self.operations.iter().fold(
            self.operations
                .capacity()
                .saturating_mul(std::mem::size_of::<PortableAspectPatchOperation>()),
            |bytes, operation| bytes.saturating_add(operation.owned_allocation_capacity_bytes()),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortableAspectPatchOperation {
    SetWhole {
        basis: PortableAspectContractBasis,
        value: ContractValidationInput,
    },
    ClearWhole {
        basis: PortableAspectContractBasis,
    },
    PatchFields {
        basis: PortableAspectContractBasis,
        selected_fields: Vec<FieldKey>,
        field_sets: Vec<PortableAspectFieldSet>,
        field_clears: Vec<FieldKey>,
    },
}

impl PortableAspectPatchOperation {
    fn clone_allocation_bytes(&self) -> Option<usize> {
        let basis = self.basis().key().as_str().len();
        match self {
            Self::SetWhole { value, .. } => basis.checked_add(value.clone_allocation_bytes()?),
            Self::ClearWhole { .. } => Some(basis),
            Self::PatchFields {
                selected_fields,
                field_sets,
                field_clears,
                ..
            } => {
                let mut bytes = basis
                    .checked_add(
                        std::alloc::Layout::array::<FieldKey>(selected_fields.len())
                            .ok()?
                            .size(),
                    )?
                    .checked_add(
                        std::alloc::Layout::array::<PortableAspectFieldSet>(field_sets.len())
                            .ok()?
                            .size(),
                    )?
                    .checked_add(
                        std::alloc::Layout::array::<FieldKey>(field_clears.len())
                            .ok()?
                            .size(),
                    )?;
                for key in selected_fields.iter().chain(field_clears) {
                    bytes = bytes.checked_add(key.as_str().len())?;
                }
                for field in field_sets {
                    bytes = bytes
                        .checked_add(field.field.as_str().len())?
                        .checked_add(field.value.clone_allocation_bytes()?)?;
                }
                Some(bytes)
            }
        }
    }

    pub fn basis(&self) -> &PortableAspectContractBasis {
        match self {
            Self::SetWhole { basis, .. }
            | Self::ClearWhole { basis }
            | Self::PatchFields { basis, .. } => basis,
        }
    }

    fn owned_allocation_capacity_bytes(&self) -> usize {
        let basis = self.basis().owned_allocation_capacity_bytes();
        match self {
            Self::SetWhole { value, .. } => {
                basis.saturating_add(value.owned_allocation_capacity_bytes())
            }
            Self::ClearWhole { .. } => basis,
            Self::PatchFields {
                selected_fields,
                field_sets,
                field_clears,
                ..
            } => basis
                .saturating_add(field_key_vector_bytes(
                    selected_fields,
                    selected_fields.capacity(),
                ))
                .saturating_add(
                    field_sets.iter().fold(
                        field_sets
                            .capacity()
                            .saturating_mul(std::mem::size_of::<PortableAspectFieldSet>()),
                        |bytes, field| {
                            bytes
                                .saturating_add(field.field.owned_allocation_capacity_bytes())
                                .saturating_add(field.value.owned_allocation_capacity_bytes())
                        },
                    ),
                )
                .saturating_add(field_key_vector_bytes(
                    field_clears,
                    field_clears.capacity(),
                )),
        }
    }
}

fn field_key_vector_bytes(fields: &[FieldKey], capacity: usize) -> usize {
    fields.iter().fold(
        capacity.saturating_mul(std::mem::size_of::<FieldKey>()),
        |bytes, field| bytes.saturating_add(field.owned_allocation_capacity_bytes()),
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortableAspectFieldSet {
    field: FieldKey,
    value: AspectValue,
}

impl PortableAspectFieldSet {
    pub fn new(field: FieldKey, value: AspectValue) -> Self {
        Self { field, value }
    }

    pub fn field(&self) -> &FieldKey {
        &self.field
    }

    pub fn value(&self) -> &AspectValue {
        &self.value
    }

    pub(crate) fn into_parts(self) -> (FieldKey, AspectValue) {
        (self.field, self.value)
    }
}
