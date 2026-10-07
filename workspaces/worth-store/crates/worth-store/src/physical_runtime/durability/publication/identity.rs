use sha2::{Digest, Sha256};
use worth_store_physical_format::{store_namespace::StableStoreIdentity, RecordArtifactFile};

use crate::physical_runtime::{
    PhysicalDurabilityGroupBasis, PhysicalDurabilityGroupIdentity,
    PhysicalDurabilityGroupMemberBinding, PhysicalDurabilityPolicyIdentity,
    PhysicalMutationIdempotencyKeyIdentity, PhysicalMutationIdentity, PhysicalWalMemberIdentity,
    RuntimeIdentity,
};

/// Exact identity shared by every effect in one current-root transition.
///
/// Construction remains inside the durability owner. Work declarations may
/// carry this identity but cannot mint or alter it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) struct PhysicalRootPublicationIdentity {
    store: StableStoreIdentity,
    runtime: RuntimeIdentity,
    policy: PhysicalDurabilityPolicyIdentity,
    basis: PublicationBasis,
    source_generation: u64,
    candidate_generation: u64,
    catalog_candidate: RecordArtifactFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PublicationBasis {
    Group {
        identity: PhysicalDurabilityGroupIdentity,
        membership: [u8; 32],
        count: u32,
    },
    Retirement {
        operation: PhysicalMutationIdentity,
        release: crate::physical_runtime::durability::RetirementReleaseProjection,
        wal_digest: [u8; 32],
    },
    ManifestResidue {
        operation: PhysicalMutationIdentity,
        intent: worth_store_physical_format::BlobManifestResidueCleanup,
        wal_digest: [u8; 32],
    },
    #[cfg(feature = "certification-test-authority")]
    TierEpoch {
        operation: PhysicalMutationIdentity,
        intent: worth_store_physical_format::TierEpochActivationV1,
        wal_digest: [u8; 32],
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalRootPublicationMemberIdentity {
    mutation: PhysicalMutationIdentity,
    wal_member: PhysicalWalMemberIdentity,
    idempotency: PhysicalMutationIdempotencyKeyIdentity,
    binding: PhysicalDurabilityGroupMemberBinding,
}

impl PhysicalRootPublicationIdentity {
    pub(in crate::physical_runtime) fn from_settled_group(
        store: StableStoreIdentity,
        runtime: RuntimeIdentity,
        policy: PhysicalDurabilityPolicyIdentity,
        group: PhysicalDurabilityGroupBasis,
        source_generation: u64,
        candidate_publication: u64,
    ) -> Option<Self> {
        let candidate_generation = source_generation.checked_add(1)?;
        (source_generation != 0 && candidate_publication != 0).then_some(Self {
            store,
            runtime,
            policy,
            basis: PublicationBasis::Group {
                identity: group.identity(),
                membership: group.membership_digest(),
                count: group.member_count().get(),
            },
            source_generation,
            candidate_generation,
            catalog_candidate: RecordArtifactFile::CatalogCandidate {
                publication: candidate_publication,
            },
        })
    }

    pub(in crate::physical_runtime) fn matches_group(
        self,
        group: PhysicalDurabilityGroupBasis,
        count: usize,
    ) -> bool {
        matches!(self.basis, PublicationBasis::Group { identity, membership, count: members }
            if identity == group.identity() && membership == group.membership_digest()
                && members == group.member_count().get() && usize::try_from(members).ok() == Some(count))
    }

    pub(in crate::physical_runtime) fn from_retirement(
        policy: PhysicalDurabilityPolicyIdentity,
        operation: PhysicalMutationIdentity,
        release: crate::physical_runtime::durability::RetirementReleaseProjection,
        wal_digest: [u8; 32],
        candidate_publication: u64,
    ) -> Option<Self> {
        (candidate_publication == release.publication() && wal_digest != [0; 32]).then_some(Self {
            store: operation.store_identity(),
            runtime: operation.runtime_identity(),
            policy,
            basis: PublicationBasis::Retirement {
                operation,
                release,
                wal_digest,
            },
            source_generation: release.source_generation(),
            candidate_generation: release.candidate_generation(),
            catalog_candidate: RecordArtifactFile::CatalogCandidate {
                publication: candidate_publication,
            },
        })
    }

    pub(in crate::physical_runtime) fn retirement_basis(
        self,
    ) -> Option<(
        PhysicalMutationIdentity,
        crate::physical_runtime::durability::RetirementReleaseProjection,
        [u8; 32],
    )> {
        match self.basis {
            PublicationBasis::Retirement {
                operation,
                release,
                wal_digest,
            } => Some((operation, release, wal_digest)),
            _ => None,
        }
    }

    pub(in crate::physical_runtime) fn from_manifest_residue(
        policy: PhysicalDurabilityPolicyIdentity,
        operation: PhysicalMutationIdentity,
        intent: worth_store_physical_format::BlobManifestResidueCleanup,
    ) -> Option<Self> {
        let wal_digest: [u8; 32] = Sha256::digest(intent.encode()).into();
        (intent.store() == operation.store_identity().bytes()
            && intent.phase()
                == worth_store_physical_format::BlobManifestResidueCleanupPhaseV1::Intent)
            .then_some(Self {
                store: operation.store_identity(),
                runtime: operation.runtime_identity(),
                policy,
                basis: PublicationBasis::ManifestResidue {
                    operation,
                    intent,
                    wal_digest,
                },
                source_generation: intent.source_root_generation(),
                candidate_generation: intent.candidate_root_generation(),
                catalog_candidate: RecordArtifactFile::CatalogCandidate {
                    publication: intent.publication(),
                },
            })
    }

    pub(in crate::physical_runtime) fn manifest_residue_basis(
        self,
    ) -> Option<(
        PhysicalMutationIdentity,
        worth_store_physical_format::BlobManifestResidueCleanup,
        [u8; 32],
    )> {
        match self.basis {
            PublicationBasis::ManifestResidue {
                operation,
                intent,
                wal_digest,
            } => Some((operation, intent, wal_digest)),
            _ => None,
        }
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn from_tier_epoch(
        policy: PhysicalDurabilityPolicyIdentity,
        operation: PhysicalMutationIdentity,
        intent: worth_store_physical_format::TierEpochActivationV1,
    ) -> Option<Self> {
        let wal_digest: [u8; 32] = Sha256::digest(intent.encode()).into();
        (intent.store() == operation.store_identity().bytes()
            && intent.phase() == worth_store_physical_format::TierEpochActivationPhaseV1::Intent)
            .then_some(Self {
                store: operation.store_identity(),
                runtime: operation.runtime_identity(),
                policy,
                basis: PublicationBasis::TierEpoch {
                    operation,
                    intent,
                    wal_digest,
                },
                source_generation: intent.source_root_generation(),
                candidate_generation: intent.candidate_root_generation(),
                catalog_candidate: RecordArtifactFile::CatalogCandidate {
                    publication: intent.publication(),
                },
            })
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn tier_epoch_basis(
        self,
    ) -> Option<(
        PhysicalMutationIdentity,
        worth_store_physical_format::TierEpochActivationV1,
        [u8; 32],
    )> {
        match self.basis {
            PublicationBasis::TierEpoch {
                operation,
                intent,
                wal_digest,
            } => Some((operation, intent, wal_digest)),
            _ => None,
        }
    }

    pub(in crate::physical_runtime) const fn source_generation(self) -> u64 {
        self.source_generation
    }

    pub(in crate::physical_runtime) const fn candidate_generation(self) -> u64 {
        self.candidate_generation
    }

    pub(in crate::physical_runtime) const fn catalog_candidate(self) -> RecordArtifactFile {
        self.catalog_candidate
    }

    pub(in crate::physical_runtime) fn stable_digest(self) -> [u8; 32] {
        let mut digest = Sha256::new();
        digest.update(b"worth-store.root-publication-identity.v2");
        digest.update(self.store.bytes());
        digest.update(self.runtime.get().to_le_bytes());
        digest.update(self.policy.bytes());
        match self.basis {
            PublicationBasis::Group {
                identity,
                membership,
                count,
            } => {
                digest.update([1]);
                digest.update(identity.bytes());
                digest.update(membership);
                digest.update(count.to_le_bytes());
            }
            PublicationBasis::Retirement {
                operation,
                release,
                wal_digest,
            } => {
                digest.update([2]);
                digest.update(operation.operation_identity().get().to_le_bytes());
                digest.update(operation.lifecycle_generation().to_le_bytes());
                digest.update(release.candidate_digest());
                digest.update(release.metadata_bytes().to_le_bytes());
                digest.update(wal_digest);
            }
            PublicationBasis::ManifestResidue {
                operation,
                intent,
                wal_digest,
            } => {
                digest.update([3]);
                digest.update(operation.operation_identity().get().to_le_bytes());
                digest.update(operation.lifecycle_generation().to_le_bytes());
                digest.update(intent.encode());
                digest.update(wal_digest);
            }
            #[cfg(feature = "certification-test-authority")]
            PublicationBasis::TierEpoch {
                operation,
                intent,
                wal_digest,
            } => {
                digest.update([4]);
                digest.update(operation.operation_identity().get().to_le_bytes());
                digest.update(operation.lifecycle_generation().to_le_bytes());
                digest.update(intent.encode());
                digest.update(wal_digest);
            }
        }
        digest.update(self.source_generation.to_le_bytes());
        digest.update(self.candidate_generation.to_le_bytes());
        let RecordArtifactFile::CatalogCandidate { publication } = self.catalog_candidate else {
            unreachable!("root identity construction fixes the catalog candidate family")
        };
        digest.update(publication.to_le_bytes());
        digest.finalize().into()
    }
}

impl PhysicalRootPublicationMemberIdentity {
    pub(in crate::physical_runtime) const fn new(
        mutation: PhysicalMutationIdentity,
        wal_member: PhysicalWalMemberIdentity,
        idempotency: PhysicalMutationIdempotencyKeyIdentity,
        binding: PhysicalDurabilityGroupMemberBinding,
    ) -> Self {
        Self {
            mutation,
            wal_member,
            idempotency,
            binding,
        }
    }

    pub const fn mutation_identity(self) -> PhysicalMutationIdentity {
        self.mutation
    }

    pub const fn wal_member_identity(self) -> PhysicalWalMemberIdentity {
        self.wal_member
    }

    pub const fn idempotency_identity(self) -> PhysicalMutationIdempotencyKeyIdentity {
        self.idempotency
    }

    pub const fn group_binding(self) -> PhysicalDurabilityGroupMemberBinding {
        self.binding
    }
}
