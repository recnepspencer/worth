//! The installed producer edition a retained run is the same run for.
//!
//! The edition is plain data. The producer registry derives it from a
//! declaration and admits a typed binding against it through
//! `application_contribution::producer::registry::semantic_edition::ProducerEditionAdmission`.

use crate::equality_type_id::EqualityTypeId as TypeId;

/// The installed declaration's framed semantic meaning. Rust type identities
/// are runtime guards; the digest contains the portable declared contract.
/// `retains_computation` is fixed at installation: the producer's operation
/// runs a partitioned computation whose state its runs retain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InstalledProducerEdition {
    digest: [u8; 32],
    build_identity: worth_foundational::facade::ApplicationBuildIdentity,
    binding_type: TypeId,
    source_type: TypeId,
    operation_binding_type: TypeId,
    provider_type: TypeId,
    retains_computation: bool,
}

impl InstalledProducerEdition {
    /// The edition the registry derived from a declaration, with its runtime
    /// type guards. Only the registry's admission builds one from parts.
    pub(in crate::domain_computation::primary_graph) const fn from_parts(
        digest: [u8; 32],
        build_identity: worth_foundational::facade::ApplicationBuildIdentity,
        binding_type: TypeId,
        source_type: TypeId,
        operation_binding_type: TypeId,
        provider_type: TypeId,
        retains_computation: bool,
    ) -> Self {
        Self {
            digest,
            build_identity,
            binding_type,
            source_type,
            operation_binding_type,
            provider_type,
            retains_computation,
        }
    }

    /// Whether the edition was installed for exactly these binding, source,
    /// operation binding and provider types: the one concrete comparison the
    /// registry's generic admission delegates to.
    pub(in crate::domain_computation::primary_graph) fn matches_types(
        &self,
        binding: TypeId,
        source: TypeId,
        operation_binding: TypeId,
        provider: TypeId,
    ) -> bool {
        self.binding_type == binding
            && self.source_type == source
            && self.operation_binding_type == operation_binding
            && self.provider_type == provider
    }

    pub(in crate::domain_computation::primary_graph) const fn retaining_computation(
        self,
        retains_computation: bool,
    ) -> Self {
        Self {
            retains_computation,
            ..self
        }
    }

    /// Whether a run of this producer looks up and retains a partitioned
    /// computation's state.
    pub(in crate::domain_computation::primary_graph) const fn retains_computation(self) -> bool {
        self.retains_computation
    }

    pub(in crate::domain_computation::primary_graph) const fn digest(self) -> [u8; 32] {
        self.digest
    }

    pub(in crate::domain_computation::primary_graph) const fn build_identity(
        self,
    ) -> worth_foundational::facade::ApplicationBuildIdentity {
        self.build_identity
    }

    pub(in crate::domain_computation::primary_graph) fn for_test(digest: [u8; 32]) -> Self {
        let unit = TypeId::of::<()>();
        Self {
            digest,
            build_identity: worth_foundational::facade::ApplicationBuildIdentity::parse(
                "11111111-1111-1111-1111-111111111111",
            )
            .expect("fixture build identity"),
            binding_type: unit,
            source_type: unit,
            operation_binding_type: unit,
            provider_type: unit,
            retains_computation: true,
        }
    }
}

mod court_doors_for_test;
