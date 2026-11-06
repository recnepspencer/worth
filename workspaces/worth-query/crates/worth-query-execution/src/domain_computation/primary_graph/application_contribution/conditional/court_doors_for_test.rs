//! Sealed test doors over this module's private members.
use super::*;

impl DeclaredConditionalBinding {
    #[doc(hidden)]
    pub fn binding_type_for_test(&self) -> &TypeId {
        &self.binding_type
    }
}

impl DeclaredConditionalBinding {
    #[doc(hidden)]
    pub fn contract_for_test(&self) -> &WorthQueryApplicationConditionalPackageContract {
        &self.contract
    }
}

impl DeclaredConditionalBinding {
    #[doc(hidden)]
    pub fn installed_type_for_test(&self) -> &TypeId {
        &self.installed_type
    }
}

impl DeclaredConditionalBinding {
    #[doc(hidden)]
    pub fn owner_for_test(&self) -> &String {
        &self.owner
    }
}

impl DeclaredConditionalBinding {
    #[doc(hidden)]
    pub fn required_producers_for_test(&self) -> &Vec<String> {
        &self.required_producers
    }
}

impl DeclaredConditionalBinding {
    #[doc(hidden)]
    pub fn new_for_test(
        owner: String,
        contract: WorthQueryApplicationConditionalPackageContract,
        required_producers: Vec<String>,
        binding_type: TypeId,
        operation_type: TypeId,
        installed_type: TypeId,
    ) -> Self {
        Self {
            owner,
            contract,
            required_producers,
            binding_type,
            operation_type,
            installed_type,
        }
    }
}
