use crate::keyspace::{AdmittedPhysicalKeyDomain, PhysicalKeyDomainWitness};
use crate::strategy::registry::LayoutRequestedCapability;
use crate::strategy::LayoutStrategyFamily;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FutureLayoutCapabilityRequest {
    PointLookup {
        key_domain: AdmittedPhysicalKeyDomain,
    },
    OrderedRange {
        key_domain: AdmittedPhysicalKeyDomain,
    },
    PrefixTraversal {
        key_domain: AdmittedPhysicalKeyDomain,
    },
    BlobStreaming {
        key_domain: AdmittedPhysicalKeyDomain,
    },
    VerifierDeclaredScan {
        key_domain: AdmittedPhysicalKeyDomain,
    },
    RebuildableProjection {
        key_domain: AdmittedPhysicalKeyDomain,
    },
}

impl FutureLayoutCapabilityRequest {
    pub const fn point_lookup(key_domain: AdmittedPhysicalKeyDomain) -> Self {
        Self::PointLookup { key_domain }
    }

    pub const fn ordered_range(key_domain: AdmittedPhysicalKeyDomain) -> Self {
        Self::OrderedRange { key_domain }
    }

    pub const fn prefix_traversal(key_domain: AdmittedPhysicalKeyDomain) -> Self {
        Self::PrefixTraversal { key_domain }
    }

    pub const fn blob_streaming(key_domain: AdmittedPhysicalKeyDomain) -> Self {
        Self::BlobStreaming { key_domain }
    }

    pub const fn verifier_declared_scan(key_domain: AdmittedPhysicalKeyDomain) -> Self {
        Self::VerifierDeclaredScan { key_domain }
    }

    pub const fn rebuildable_projection(key_domain: AdmittedPhysicalKeyDomain) -> Self {
        Self::RebuildableProjection { key_domain }
    }

    pub const fn admitted_key_domain(self) -> AdmittedPhysicalKeyDomain {
        match self {
            Self::PointLookup { key_domain }
            | Self::OrderedRange { key_domain }
            | Self::PrefixTraversal { key_domain }
            | Self::BlobStreaming { key_domain }
            | Self::VerifierDeclaredScan { key_domain }
            | Self::RebuildableProjection { key_domain } => key_domain,
        }
    }

    pub const fn key_domain(self) -> PhysicalKeyDomainWitness {
        self.admitted_key_domain().witness()
    }

    pub const fn requested_capability(self) -> Option<LayoutRequestedCapability> {
        match self {
            Self::PointLookup { .. } => Some(LayoutRequestedCapability::PointLookup),
            Self::OrderedRange { .. } => Some(LayoutRequestedCapability::OrderedRange),
            Self::PrefixTraversal { .. } => Some(LayoutRequestedCapability::PrefixTraversal),
            Self::BlobStreaming { .. } => Some(LayoutRequestedCapability::BlobStreaming),
            Self::VerifierDeclaredScan { .. } => Some(LayoutRequestedCapability::ExactScan),
            Self::RebuildableProjection { .. } => None,
        }
    }

    pub(crate) const fn admitted_strategy_family(self) -> Option<LayoutStrategyFamily> {
        match self {
            Self::PointLookup { key_domain }
            | Self::OrderedRange { key_domain }
            | Self::PrefixTraversal { key_domain } => match key_domain.domain() {
                crate::PhysicalKeyDomain::PageAddressKey
                | crate::PhysicalKeyDomain::SegmentAddressKey
                | crate::PhysicalKeyDomain::ExtentAddressKey
                | crate::PhysicalKeyDomain::PhysicalReferenceKey => {
                    Some(LayoutStrategyFamily::BTreeRange)
                }
                crate::PhysicalKeyDomain::WalRecordKey
                | crate::PhysicalKeyDomain::BlobIdentityKey => {
                    Some(LayoutStrategyFamily::BaselineLsmWriteOptimized)
                }
                crate::PhysicalKeyDomain::RootManifestKey => None,
            },
            Self::BlobStreaming { key_domain } | Self::VerifierDeclaredScan { key_domain } => {
                match key_domain.domain() {
                    crate::PhysicalKeyDomain::WalRecordKey
                    | crate::PhysicalKeyDomain::BlobIdentityKey => {
                        Some(LayoutStrategyFamily::BaselineLsmWriteOptimized)
                    }
                    crate::PhysicalKeyDomain::RootManifestKey
                    | crate::PhysicalKeyDomain::PageAddressKey
                    | crate::PhysicalKeyDomain::SegmentAddressKey
                    | crate::PhysicalKeyDomain::ExtentAddressKey
                    | crate::PhysicalKeyDomain::PhysicalReferenceKey => None,
                }
            }
            Self::RebuildableProjection { .. } => None,
        }
    }
}
