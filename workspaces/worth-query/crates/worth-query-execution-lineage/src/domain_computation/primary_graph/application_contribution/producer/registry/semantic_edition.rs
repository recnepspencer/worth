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
    pub const fn from_parts(
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

    pub fn matches_types(
        self,
        binding_type: TypeId,
        source_type: TypeId,
        operation_binding_type: TypeId,
        provider_type: TypeId,
    ) -> bool {
        self.binding_type == binding_type
            && self.source_type == source_type
            && self.operation_binding_type == operation_binding_type
            && self.provider_type == provider_type
    }

    pub const fn retaining_computation(self, retains_computation: bool) -> Self {
        Self {
            retains_computation,
            ..self
        }
    }

    /// Whether a run of this producer looks up and retains a partitioned
    /// computation's state.
    pub const fn retains_computation(self) -> bool {
        self.retains_computation
    }

    pub const fn digest(self) -> [u8; 32] {
        self.digest
    }

    pub const fn build_identity(self) -> worth_foundational::facade::ApplicationBuildIdentity {
        self.build_identity
    }

    pub fn for_test(digest: [u8; 32]) -> Self {
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
