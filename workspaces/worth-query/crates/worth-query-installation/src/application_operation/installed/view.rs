//! Borrowed facts of one genuine installed operation, without its application types.
use super::*;

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct InstalledOperationView<'a> {
    binding: &'a ApplicationSchemaBindingIdentity,
    operation: &'a str,
    input: &'a str,
    contracts: &'a WorthQueryCompiledApplicationOperationContracts,
    seal: &'a AuthoritySeal,
    definition: &'a [u8; 32],
    operation_type: std::any::TypeId,
    input_type: std::any::TypeId,
}

impl<'a> InstalledOperationView<'a> {
    pub fn binding_identity(&self) -> &'a ApplicationSchemaBindingIdentity {
        self.binding
    }
    pub fn operation(&self) -> &'a str {
        self.operation
    }
    pub fn input_type(&self) -> &'a str {
        self.input
    }
    pub fn contracts(&self) -> &'a WorthQueryCompiledApplicationOperationContracts {
        self.contracts
    }
    pub fn authority_identity_bytes(&self) -> [u8; 32] {
        *self.seal.bytes()
    }
    pub fn definition_identity_bytes(&self) -> [u8; 32] {
        *self.definition
    }
    pub fn operation_type_id(&self) -> std::any::TypeId {
        self.operation_type
    }
    pub fn input_type_id(&self) -> std::any::TypeId {
        self.input_type
    }

    pub(super) fn same_installation(&self, other: InstalledOperationView<'_>) -> bool {
        self.binding == other.binding
            && self.operation == other.operation
            && self.input == other.input
            && self.seal == other.seal
            && self.definition == other.definition
            && self.operation_type == other.operation_type
            && self.input_type == other.input_type
            && self.contracts == other.contracts
    }
}

impl super::backing::InstalledOperationBacking {
    pub(super) fn view(&self) -> InstalledOperationView<'_> {
        InstalledOperationView {
            binding: &self.binding_identity,
            operation: &self.operation,
            input: &self.input_type,
            contracts: &self.contracts,
            seal: &self.authority_identity,
            definition: &self.definition_identity,
            operation_type: self.operation_type_id,
            input_type: self.input_type_id,
        }
    }
}

impl<S, O, I> WorthQueryInstalledApplicationOperation<S, O, I> {
    #[doc(hidden)]
    pub fn installed_view(&self) -> InstalledOperationView<'_> {
        self.core.view()
    }
}
