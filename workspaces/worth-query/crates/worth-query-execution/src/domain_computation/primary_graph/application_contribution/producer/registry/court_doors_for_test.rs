//! Sealed test doors over this module's private members.
use super::*;

impl DeclaredProducerBinding {
    #[doc(hidden)]
    pub fn identity_for_test(&self) -> &String {
        &self.identity
    }
}

impl DeclaredProducerBinding {
    #[doc(hidden)]
    pub fn installed_output_equivalence_for_test(
        &self,
    ) -> &Option<Arc<super::super::output_equivalence_contract::InstalledProducerOutputEquivalence>>
    {
        &self.installed_output_equivalence
    }
}

impl DeclaredProducerBinding {
    #[doc(hidden)]
    pub fn output_equivalence_for_test(
        &self,
    ) -> &Option<super::super::super::native_field_contract::NativeFieldBindingExpectation> {
        &self.output_equivalence
    }
}

impl DeclaredProducerBinding {
    #[doc(hidden)]
    pub fn output_family_type_for_test(&self) -> &TypeId {
        &self.output_family_type
    }
}

impl DeclaredProducerBinding {
    #[doc(hidden)]
    pub fn output_role_for_test(&self) -> &String {
        &self.output_role
    }
}

impl DeclaredProducerBinding {
    #[doc(hidden)]
    pub fn output_role_descriptors_for_test(&self) -> &Vec<worth_query_declaration::facade::application_operation::ApplicationMutationOutputRoleDescriptor>{
        &self.output_role_descriptors
    }
}

impl DeclaredProducerBinding {
    #[doc(hidden)]
    pub fn supported_for_test(&self) -> &Vec<WorthQueryProducerApplicability> {
        &self.supported
    }
}

impl DeclaredProducerBinding {
    #[doc(hidden)]
    pub fn handler_identity_mut_for_test(&mut self) -> &mut String {
        &mut self.handler_identity
    }
}

impl DeclaredProducerBinding {
    #[doc(hidden)]
    pub fn identity_mut_for_test(&mut self) -> &mut String {
        &mut self.identity
    }
}

impl DeclaredProducerBinding {
    #[doc(hidden)]
    pub fn input_reuse_mut_for_test(
        &mut self,
    ) -> &mut Option<super::super::WorthQueryProducerInputReuseContract> {
        &mut self.input_reuse
    }
}

impl DeclaredProducerBinding {
    #[doc(hidden)]
    pub fn installed_output_equivalence_mut_for_test(
        &mut self,
    ) -> &mut Option<
        Arc<super::super::output_equivalence_contract::InstalledProducerOutputEquivalence>,
    > {
        &mut self.installed_output_equivalence
    }
}

impl DeclaredProducerBinding {
    #[doc(hidden)]
    pub fn operation_binding_type_mut_for_test(&mut self) -> &mut TypeId {
        &mut self.operation_binding_type
    }
}

impl DeclaredProducerBinding {
    #[doc(hidden)]
    pub fn output_equivalence_mut_for_test(
        &mut self,
    ) -> &mut Option<super::super::super::native_field_contract::NativeFieldBindingExpectation>
    {
        &mut self.output_equivalence
    }
}

impl DeclaredProducerBinding {
    #[doc(hidden)]
    pub fn output_family_mut_for_test(&mut self) -> &mut String {
        &mut self.output_family
    }
}

impl DeclaredProducerBinding {
    #[doc(hidden)]
    pub fn output_family_type_mut_for_test(&mut self) -> &mut TypeId {
        &mut self.output_family_type
    }
}

impl DeclaredProducerBinding {
    #[doc(hidden)]
    pub fn output_role_mut_for_test(&mut self) -> &mut String {
        &mut self.output_role
    }
}

impl DeclaredProducerBinding {
    #[doc(hidden)]
    pub fn output_role_descriptors_mut_for_test(&mut self) -> &mut Vec<worth_query_declaration::facade::application_operation::ApplicationMutationOutputRoleDescriptor>{
        &mut self.output_role_descriptors
    }
}

impl DeclaredProducerBinding {
    #[doc(hidden)]
    pub fn output_role_families_mut_for_test(
        &mut self,
    ) -> &mut Vec<ApplicationMutationOutputRoleFamilyDescriptor> {
        &mut self.output_role_families
    }
}

impl DeclaredProducerBinding {
    #[doc(hidden)]
    pub fn owned_scope_entity_mut_for_test(&mut self) -> &mut Option<String> {
        &mut self.owned_scope_entity
    }
}

impl DeclaredProducerBinding {
    #[doc(hidden)]
    pub fn ownership_mut_for_test(
        &mut self,
    ) -> &mut worth_foundational::facade::ProducerPublicationOwnership {
        &mut self.ownership
    }
}

impl DeclaredProducerBinding {
    #[doc(hidden)]
    pub fn provider_identity_mut_for_test(&mut self) -> &mut String {
        &mut self.provider_identity
    }
}

impl DeclaredProducerBinding {
    #[doc(hidden)]
    pub fn supported_mut_for_test(&mut self) -> &mut Vec<WorthQueryProducerApplicability> {
        &mut self.supported
    }
}
