//! How the registry derives a producer's installed edition from its
//! declaration, and admits a typed binding against it.
//!
//! The edition itself is retention vocabulary
//! (`computation_retention::producer_edition`); deriving and admitting it
//! names the registry's declaration and the producer binding traits, so both
//! stay here as `ProducerEditionAdmission`.

use crate::equality_type_id::EqualityTypeId as TypeId;

use worth_query_declaration::facade::application_operation::{
    application_value_identity, ApplicationMutationOutputPosture, ApplicationValueIdentityDomain,
};

use super::DeclaredProducerBinding;
use crate::domain_computation::graph_layout::denial::WorthQueryPrimaryGraphInstallationDenial;
use crate::domain_computation::graph_layout::denial::WorthQueryPrimaryGraphInstallationDenialKind as DenialKind;
use crate::domain_computation::primary_graph::computation_retention::producer_edition::InstalledProducerEdition;

/// The registry's admission of an installed edition: derived from a
/// declaration at installation, and matched against a producer binding's
/// types when the binding runs.
pub(in crate::domain_computation::primary_graph::application_contribution::producer) trait ProducerEditionAdmission:
    Sized
{
    /// Whether the edition was installed for this binding's types. Generic
    /// over the binding; the comparison itself is one concrete call.
    fn admits_binding<Schema, Binding>(self) -> bool
    where
        Schema: worth_query_installation::facade::ApplicationSchema + 'static,
        Binding: super::super::WorthQueryApplicationProducerBinding<Schema> + 'static;

    /// The edition a declaration installs under `build_identity`.
    fn from_declaration(
        declaration: &DeclaredProducerBinding,
        build_identity: worth_foundational::facade::ApplicationBuildIdentity,
    ) -> Result<Self, WorthQueryPrimaryGraphInstallationDenial>;
}

impl ProducerEditionAdmission for InstalledProducerEdition {
    fn admits_binding<Schema, Binding>(self) -> bool
    where
        Schema: worth_query_installation::facade::ApplicationSchema + 'static,
        Binding: super::super::WorthQueryApplicationProducerBinding<Schema> + 'static,
    {
        self.matches_types(
            TypeId::of::<Binding>(),
            TypeId::of::<
                <Binding::OutputFamily as super::super::WorthQueryProducerOutputFamily<
                    Schema,
                >>::Source,
            >(),
            TypeId::of::<Binding::Operation>(),
            TypeId::of::<Binding::Provider>(),
        )
    }

    fn from_declaration(
        declaration: &DeclaredProducerBinding,
        build_identity: worth_foundational::facade::ApplicationBuildIdentity,
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
            roles,
            families,
            declaration.output_role.as_str(),
            declaration.operation.as_str(),
            declaration.provider_identity.as_str(),
            applicability,
            supported,
            invariants,
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
        // Input reuse remains part of declared meaning; every producer below
        // additionally receives the mandatory application build identity.
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
        let digest = match (
            &declaration.output_equivalence,
            &declaration.installed_output_equivalence,
        ) {
            (None, None) => digest,
            (Some(field), Some(installed)) if field == &installed.field.binding => {
                application_value_identity(
                    ApplicationValueIdentityDomain::ProducerImplementationEdition,
                    &declaration.identity,
                    &("producer-output-equivalence.v1", digest, installed.meaning),
                )
                .map_err(|_| {
                    WorthQueryPrimaryGraphInstallationDenial::new(
                        DenialKind::ProducerBindingMeaningMismatch,
                        declaration.identity.clone(),
                    )
                })?
                .identity()
            }
            _ => {
                return Err(WorthQueryPrimaryGraphInstallationDenial::new(
                    DenialKind::ProducerBindingMeaningMismatch,
                    declaration.identity.clone(),
                ))
            }
        };
        let digest = match declaration.ownership {
            worth_foundational::facade::ProducerPublicationOwnership::Independent => digest,
            worth_foundational::facade::ProducerPublicationOwnership::SourceScopeCascade => {
                application_value_identity(
                    ApplicationValueIdentityDomain::ProducerImplementationEdition,
                    &declaration.identity,
                    &(
                        "producer-source-scope-cascade.v1",
                        digest,
                        &declaration.owned_scope_entity,
                    ),
                )
                .map_err(|_| {
                    WorthQueryPrimaryGraphInstallationDenial::new(
                        DenialKind::ProducerBindingMeaningMismatch,
                        declaration.identity.clone(),
                    )
                })?
                .identity()
            }
        };
        let digest = application_value_identity(
            ApplicationValueIdentityDomain::ProducerImplementationEdition,
            &declaration.identity,
            &(
                "application-build-producing-edition.v1",
                build_identity.bytes(),
                digest,
            ),
        )
        .map_err(|_| {
            WorthQueryPrimaryGraphInstallationDenial::new(
                DenialKind::ProducerBindingMeaningMismatch,
                declaration.identity.clone(),
            )
        })?
        .identity();
        Ok(InstalledProducerEdition::from_parts(
            digest,
            build_identity,
            declaration.binding_type,
            declaration.source_type,
            declaration.operation_binding_type,
            declaration.provider_type,
            false,
        ))
    }
}

fn posture_code(posture: ApplicationMutationOutputPosture) -> u8 {
    match posture {
        ApplicationMutationOutputPosture::Preserve => 1,
        ApplicationMutationOutputPosture::Create => 2,
        ApplicationMutationOutputPosture::Retire => 3,
    }
}

mod court_doors_for_test;
