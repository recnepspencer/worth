use std::any::TypeId;

use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationOutputContract,
};
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_installation::facade::ApplicationSchema;

use super::DeclaredProducerBinding;
use crate::domain_computation::primary_graph::application_contribution::{
    WorthQueryApplicationProducerBinding, WorthQueryApplicationProducerProvider,
    WorthQueryProducerOutputFamily,
};

impl DeclaredProducerBinding {
    pub(in crate::domain_computation::primary_graph::application_contribution) fn of<
        Schema,
        Binding,
    >(
        owner: &str,
    ) -> Self
    where
        Schema: ApplicationSchema,
        Binding: WorthQueryApplicationProducerBinding<Schema>,
    {
        Self {
            owner: owner.to_owned(),
            identity: Binding::IDENTITY.to_owned(),
            source_selector:
                <Binding::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Source::IDENTITY
                    .to_owned(),
            output_family: Binding::OutputFamily::IDENTITY.to_owned(),
            output_family_type: TypeId::of::<Binding::OutputFamily>(),
            output_roles: <Binding::Operation as ApplicationMutationBinding<Schema>>::Output::ROLES
                .iter()
                .map(|role| role.name().to_owned())
                .collect(),
            output_role_descriptors:
                <Binding::Operation as ApplicationMutationBinding<Schema>>::Output::ROLES.to_vec(),
            output_role_families:
                <Binding::Operation as ApplicationMutationBinding<Schema>>::Output::ROLE_FAMILIES
                    .to_vec(),
            output_role: Binding::OUTPUT_ROLE.to_owned(),
            operation: Binding::Operation::IDENTITY.to_owned(),
            provider_identity: Binding::Provider::SEMANTIC_IDENTITY.to_owned(),
            applicability: Binding::APPLICABILITY.to_vec(),
            supported: Binding::OutputFamily::SUPPORTED.to_vec(),
            required_invariants: Binding::REQUIRED_INVARIANTS.to_vec(),
            resource_policy: Binding::RESOURCE_POLICY.to_owned(),
            reuse_policy: Binding::REUSE_POLICY.to_owned(),
            binding_type: TypeId::of::<Binding>(),
            source_type: TypeId::of::<
                <Binding::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Source,
            >(),
            operation_binding_type: TypeId::of::<Binding::Operation>(),
            provider_type: TypeId::of::<Binding::Provider>(),
        }
    }

    pub(in crate::domain_computation::primary_graph::application_contribution) fn meaning_matches<
        Schema,
        Binding,
    >(
        &self,
    ) -> bool
    where
        Schema: ApplicationSchema,
        Binding: WorthQueryApplicationProducerBinding<Schema>,
    {
        let expected = Self::of::<Schema, Binding>(&self.owner);
        self.has_same_meaning_as(&expected)
    }

    pub(super) fn has_same_meaning_as(&self, expected: &Self) -> bool {
        self.identity == expected.identity
            && self.source_selector == expected.source_selector
            && self.output_family == expected.output_family
            && self.output_family_type == expected.output_family_type
            && self.output_roles == expected.output_roles
            && self.output_role_descriptors == expected.output_role_descriptors
            && self.output_role_families == expected.output_role_families
            && self.output_role == expected.output_role
            && self.operation == expected.operation
            && self.provider_identity == expected.provider_identity
            && self.applicability == expected.applicability
            && self.supported == expected.supported
            && self.required_invariants == expected.required_invariants
            && self.resource_policy == expected.resource_policy
            && self.reuse_policy == expected.reuse_policy
            && self.binding_type == expected.binding_type
            && self.source_type == expected.source_type
            && self.operation_binding_type == expected.operation_binding_type
            && self.provider_type == expected.provider_type
    }
}
