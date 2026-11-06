//! Sealed test doors over this module's private members.
use crate::domain_computation::vocabulary::provider_session::declared_role_closure::WorthQueryProviderPlanDeclaredClosure;

impl WorthQueryProviderPlanDeclaredClosure {
    #[doc(hidden)]
    pub fn invariant_for_test(&self) -> &Vec<String> {
        &self.invariant
    }
}

impl WorthQueryProviderPlanDeclaredClosure {
    #[doc(hidden)]
    pub fn read_mut_for_test(&mut self) -> &mut Vec<String> {
        &mut self.read
    }
}

impl WorthQueryProviderPlanDeclaredClosure {
    #[doc(hidden)]
    pub fn touch_mut_for_test(&mut self) -> &mut Vec<String> {
        &mut self.touch
    }
}
