use std::any::TypeId;

use worth_query_declaration::facade::application_operation::{
    application_value_identity, ApplicationMutationOutputPosture, ApplicationValueIdentityDomain,
};

use super::DeclaredProducerBinding;
use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind as DenialKind,
};

/// The installed declaration's framed semantic meaning. Rust type identities
/// are runtime guards; the digest contains the portable declared contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct InstalledProducerEdition {
    digest: [u8; 32],
    binding_type: TypeId,
    source_type: TypeId,
    operation_binding_type: TypeId,
    provider_type: TypeId,
}

impl InstalledProducerEdition {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn admits_binding<
        Schema,
        Binding,
    >(
        self,
    ) -> bool
    where
        Schema: worth_query_installation::facade::ApplicationSchema + 'static,
        Binding: super::super::WorthQueryApplicationProducerBinding<Schema> + 'static,
    {
        self.binding_type == TypeId::of::<Binding>()
            && self.source_type
                == TypeId::of::<
                    <Binding::OutputFamily as super::super::WorthQueryProducerOutputFamily<
                        Schema,
                    >>::Source,
                >()
            && self.operation_binding_type == TypeId::of::<Binding::Operation>()
            && self.provider_type == TypeId::of::<Binding::Provider>()
    }

    pub(super) fn from_declaration(
        declaration: &DeclaredProducerBinding,
    ) -> Result<Self, WorthQueryPrimaryGraphInstallationDenial> {
        let roles: Vec<_> = declaration
            .output_role_descriptors
            .iter()
            .map(|role| (role.name(), role.entity(), posture_code(role.posture())))
            .collect();
        let families: Vec<_> = declaration
            .output_role_families
            .iter()
            .map(|family| {
                (
                    family.prefix(),
                    family.entity(),
                    family.postures().bits(),
                    family.minimum(),
                )
            })
            .collect();
        let applicability: Vec<_> = declaration
            .applicability
            .iter()
            .map(|item| (item.profile_kind(), item.lifecycle() as u8))
            .collect();
        let supported: Vec<_> = declaration
            .supported
            .iter()
            .map(|item| (item.profile_kind(), item.lifecycle() as u8))
            .collect();
        let invariants: Vec<_> = declaration
            .required_invariants
            .iter()
            .map(|item| {
                (
                    item.identifier(),
                    item.major(),
                    item.minor(),
                    item.execution_point().canonical_token(),
                )
            })
            .collect();
        let meaning = (
            declaration.owner.as_str(),
            declaration.identity.as_str(),
            declaration.source_selector.as_str(),
            declaration.output_family.as_str(),
            declaration.output_roles.as_slice(),
            roles,
            families,
            declaration.output_role.as_str(),
            declaration.operation.as_str(),
            declaration.provider_identity.as_str(),
            applicability,
            supported,
            invariants,
            declaration.resource_policy.as_str(),
            declaration.reuse_policy.as_str(),
        );
        let base_digest = application_value_identity(
            ApplicationValueIdentityDomain::ProducerImplementationEdition,
            &declaration.identity,
            &meaning,
        )
        .map_err(|_| {
            WorthQueryPrimaryGraphInstallationDenial::new(
                DenialKind::ProducerBindingMeaningMismatch,
                declaration.identity.clone(),
            )
        })?
        .identity();
        // The None branch preserves the prior edition encoding. An opt-in
        // binds the exact installed handler as well as the producer meaning.
        let digest = match declaration.input_reuse {
            None => base_digest,
            Some(contract) => application_value_identity(
                ApplicationValueIdentityDomain::ProducerImplementationEdition,
                &declaration.identity,
                &(
                    "producer-input-reuse.v1",
                    base_digest,
                    declaration.handler_identity.as_str(),
                    contract.portable_meaning(),
                ),
            )
            .map_err(|_| {
                WorthQueryPrimaryGraphInstallationDenial::new(
                    DenialKind::ProducerBindingMeaningMismatch,
                    declaration.identity.clone(),
                )
            })?
            .identity(),
        };
        Ok(Self {
            digest,
            binding_type: declaration.binding_type,
            source_type: declaration.source_type,
            operation_binding_type: declaration.operation_binding_type,
            provider_type: declaration.provider_type,
        })
    }

    pub(in crate::domain_computation::primary_graph) const fn digest(self) -> [u8; 32] {
        self.digest
    }
}

fn posture_code(posture: ApplicationMutationOutputPosture) -> u8 {
    match posture {
        ApplicationMutationOutputPosture::Preserve => 1,
        ApplicationMutationOutputPosture::Create => 2,
        ApplicationMutationOutputPosture::Retire => 3,
    }
}
