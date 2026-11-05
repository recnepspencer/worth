//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_contribution::producer::WorthQueryProducerApplicability;
use crate::domain_computation::primary_graph::vocabulary::application_contribution::producer::WorthQueryProducerInvariantRequirement;

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
    ) -> &mut Option<crate::domain_computation::vocabulary::primary_graph::input_reuse_contract::WorthQueryProducerInputReuseContract> {
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

impl DeclaredProducerBinding {
    #[doc(hidden)]
    pub fn installed_output_equivalence_into_for_test(
        self,
    ) -> Option<Arc<super::super::output_equivalence_contract::InstalledProducerOutputEquivalence>>
    {
        self.installed_output_equivalence
    }
}

impl DeclaredProducerBinding {
    #[doc(hidden)]
    pub fn new_for_test(
        ownership: worth_foundational::facade::ProducerPublicationOwnership,
        owned_scope_entity: Option<String>,
        owner: String,
        identity: String,
        source_selector: String,
        output_family: String,
        output_family_type: TypeId,
        output_role_descriptors: Vec<worth_query_declaration::facade::application_operation::ApplicationMutationOutputRoleDescriptor>,
        output_role_families: Vec<ApplicationMutationOutputRoleFamilyDescriptor>,
        output_role: String,
        operation: String,
        handler_identity: String,
        provider_identity: String,
        applicability: Vec<WorthQueryProducerApplicability>,
        supported: Vec<WorthQueryProducerApplicability>,
        required_invariants: Vec<WorthQueryProducerInvariantRequirement>,
        reuse_policy: String,
        input_reuse: Option<crate::domain_computation::vocabulary::primary_graph::input_reuse_contract::WorthQueryProducerInputReuseContract>,
        output_equivalence: Option<
            super::super::super::native_field_contract::NativeFieldBindingExpectation,
        >,
        installed_output_equivalence: Option<
            Arc<super::super::output_equivalence_contract::InstalledProducerOutputEquivalence>,
        >,
        binding_type: TypeId,
        source_type: TypeId,
        provider_type: TypeId,
        operation_binding_type: TypeId,
        operation_type: TypeId,
        output_contract_type: TypeId,
    ) -> Self {
        Self {
            ownership,
            owned_scope_entity,
            owner,
            identity,
            source_selector,
            output_family,
            output_family_type,
            output_role_descriptors,
            output_role_families,
            output_role,
            operation,
            handler_identity,
            provider_identity,
            applicability,
            supported,
            required_invariants,
            reuse_policy,
            input_reuse,
            output_equivalence,
            installed_output_equivalence,
            binding_type,
            source_type,
            provider_type,
            operation_binding_type,
            operation_type,
            output_contract_type,
        }
    }
}
