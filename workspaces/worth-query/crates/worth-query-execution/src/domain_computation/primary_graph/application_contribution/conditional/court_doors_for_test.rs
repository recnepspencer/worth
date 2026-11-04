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
