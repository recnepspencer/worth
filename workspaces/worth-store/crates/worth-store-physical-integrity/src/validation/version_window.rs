use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;

use super::{
    PhysicalArtifactScope, PhysicalIntegrityVersionAxis, UnsupportedPhysicalIntegrityVersion,
};

/// Descriptive adapter for the artifact family's current persisted version.
///
/// The adapter carries no bytes and cannot open a decoder. Current C.5-C.8
/// families each expose one supported version; a later coexistence window must
/// add a real version-specific adapter rather than reinterpret old bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalIntegrityArtifactVersionAdapter {
    scope: PhysicalArtifactScope,
    axis: PhysicalIntegrityVersionAxis,
    supported: u32,
}

/// Descriptive adapter for the durable-frame envelope surrounding a family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalIntegrityEnvelopeVersionAdapter {
    scope: PhysicalArtifactScope,
    supported: u32,
}

/// In-window version observation. This is not validation or admission proof.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalIntegritySupportedVersion {
    scope: PhysicalArtifactScope,
    axis: PhysicalIntegrityVersionAxis,
    observed: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalIntegrityVersionWindowOutcome {
    Supported(PhysicalIntegritySupportedVersion),
    Unsupported(UnsupportedPhysicalIntegrityVersion),
}

impl PhysicalIntegrityArtifactVersionAdapter {
    pub const fn for_scope(scope: PhysicalArtifactScope) -> Self {
        let version = scope.format_version().format_version();
        Self {
            scope,
            axis: artifact_version_axis(scope.artifact_family()),
            supported: version as u32,
        }
    }

    pub const fn axis(self) -> PhysicalIntegrityVersionAxis {
        self.axis
    }

    pub const fn supported_version(self) -> u32 {
        self.supported
    }

    pub const fn observe(self, observed: u32) -> PhysicalIntegrityVersionWindowOutcome {
        observe(self.scope, self.axis, self.supported, observed)
    }
}

impl PhysicalIntegrityEnvelopeVersionAdapter {
    pub const fn for_scope(scope: PhysicalArtifactScope) -> Option<Self> {
        match scope.format_version().envelope_schema() {
            Some(supported) => Some(Self {
                scope,
                supported: supported as u32,
            }),
            None => None,
        }
    }

    pub const fn supported_version(self) -> u32 {
        self.supported
    }

    pub const fn observe(self, observed: u32) -> PhysicalIntegrityVersionWindowOutcome {
        observe(
            self.scope,
            PhysicalIntegrityVersionAxis::EnvelopeSchema,
            self.supported,
            observed,
        )
    }
}

impl PhysicalIntegritySupportedVersion {
    pub const fn scope(self) -> PhysicalArtifactScope {
        self.scope
    }

    pub const fn axis(self) -> PhysicalIntegrityVersionAxis {
        self.axis
    }

    pub const fn observed(self) -> u32 {
        self.observed
    }
}

const fn observe(
    scope: PhysicalArtifactScope,
    axis: PhysicalIntegrityVersionAxis,
    supported: u32,
    observed: u32,
) -> PhysicalIntegrityVersionWindowOutcome {
    if observed == supported {
        PhysicalIntegrityVersionWindowOutcome::Supported(PhysicalIntegritySupportedVersion {
            scope,
            axis,
            observed,
        })
    } else {
        PhysicalIntegrityVersionWindowOutcome::Unsupported(
            UnsupportedPhysicalIntegrityVersion::new(scope, axis, observed),
        )
    }
}

const fn artifact_version_axis(
    family: PhysicalIntegrityArtifactFamily,
) -> PhysicalIntegrityVersionAxis {
    use PhysicalIntegrityArtifactFamily as Family;
    match family {
        Family::PhysicalWorkObligation => PhysicalIntegrityVersionAxis::PhysicalWorkObligation,
        Family::WalFrame => PhysicalIntegrityVersionAxis::WalFrame,
        Family::CheckpointStreamHeader
        | Family::CheckpointDirtyBasis
        | Family::CheckpointBindingCompaction
        | Family::CheckpointBinding
        | Family::CheckpointFooter => PhysicalIntegrityVersionAxis::CheckpointRecordSchema,
        Family::BootstrapCatalog
        | Family::CurrentRootSelector
        | Family::PreviousRootSelector
        | Family::RootManifest
        | Family::RootRoutingBlock
        | Family::SegmentMembership
        | Family::PageFrame
        | Family::ExtentManifest
        | Family::ExtentChunk
        | Family::FreeSpaceHeader
        | Family::FreeSpaceMembershipBlock => PhysicalIntegrityVersionAxis::PhysicalFormat,
        Family::NamespaceIdentity => {
            panic!("namespace identity remains under the C.4 version owner")
        }
    }
}

#[cfg(test)]
#[path = "version_window_family_matrix.rs"]
mod version_window_family_matrix;

#[cfg(test)]
mod tests {
    use core::num::NonZeroU64;

    use worth_store_physical_format::store_namespace::{
        ProposedStoreIdentity, StableStoreIdentity, StoreNamespaceIdentityRecord,
        StoreNamespaceVersion,
    };
    use worth_store_physical_format::{
        PhysicalCheckpointIdentity, PhysicalRecordFormatDeclaration,
        PhysicalWorkObligationIdentity, WalSegmentIdentity,
    };

    use super::*;
    use crate::{CheckpointStreamHeaderScopeIdentity, PhysicalByteRange};

    #[test]
    fn current_c5_c8_families_select_their_explicit_version_axes() {
        super::version_window_family_matrix::assert_current_family_version_matrix();

        let store = store();
        let range = PhysicalByteRange::new(0, 160).unwrap();
        let work = PhysicalArtifactScope::physical_work_obligation(
            store,
            PhysicalWorkObligationIdentity::new(nz(1), nz(2), nz(3)),
            range,
        );
        let wal =
            PhysicalArtifactScope::wal_frame(store, WalSegmentIdentity::new(4, 5).unwrap(), range);
        let checkpoint = PhysicalArtifactScope::checkpoint_stream_header(
            CheckpointStreamHeaderScopeIdentity::known(PhysicalCheckpointIdentity::new(
                store,
                nz(6),
            )),
            range,
        );
        let durable = PhysicalArtifactScope::current_root_selector(
            store,
            PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
            range,
        );

        assert_adapter(
            work,
            PhysicalIntegrityVersionAxis::PhysicalWorkObligation,
            6,
        );
        assert_adapter(wal, PhysicalIntegrityVersionAxis::WalFrame, 1);
        assert_adapter(
            checkpoint,
            PhysicalIntegrityVersionAxis::CheckpointRecordSchema,
            1,
        );
        assert_adapter(durable, PhysicalIntegrityVersionAxis::PhysicalFormat, 1);
        assert!(PhysicalIntegrityEnvelopeVersionAdapter::for_scope(work).is_none());
        assert!(PhysicalIntegrityEnvelopeVersionAdapter::for_scope(wal).is_none());
        assert!(PhysicalIntegrityEnvelopeVersionAdapter::for_scope(checkpoint).is_none());
        let envelope = PhysicalIntegrityEnvelopeVersionAdapter::for_scope(durable).unwrap();
        assert_eq!(envelope.supported_version(), 2);
    }

    #[test]
    fn outside_window_is_unsupported_not_corruption() {
        let scope = PhysicalArtifactScope::current_root_selector(
            store(),
            PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
            PhysicalByteRange::new(0, 107).unwrap(),
        );
        let adapter = PhysicalIntegrityArtifactVersionAdapter::for_scope(scope);

        let PhysicalIntegrityVersionWindowOutcome::Unsupported(unsupported) = adapter.observe(2)
        else {
            panic!("future format version must remain unsupported");
        };
        assert_eq!(unsupported.scope(), scope);
        assert_eq!(
            unsupported.axis(),
            PhysicalIntegrityVersionAxis::PhysicalFormat
        );
        assert_eq!(unsupported.observed(), 2);

        let PhysicalIntegrityVersionWindowOutcome::Supported(supported) = adapter.observe(1) else {
            panic!("current format version must be supported");
        };
        assert_eq!(supported.scope(), scope);
        assert_eq!(
            supported.axis(),
            PhysicalIntegrityVersionAxis::PhysicalFormat
        );
        assert_eq!(supported.observed(), 1);
    }

    fn assert_adapter(
        scope: PhysicalArtifactScope,
        axis: PhysicalIntegrityVersionAxis,
        supported: u32,
    ) {
        let adapter = PhysicalIntegrityArtifactVersionAdapter::for_scope(scope);
        assert_eq!(adapter.axis(), axis);
        assert_eq!(adapter.supported_version(), supported);
        assert!(matches!(
            adapter.observe(supported),
            PhysicalIntegrityVersionWindowOutcome::Supported(_)
        ));
    }

    fn store() -> StableStoreIdentity {
        StoreNamespaceIdentityRecord::new(
            StoreNamespaceVersion::CURRENT,
            ProposedStoreIdentity::from_nonzero_bytes([9; 16]).unwrap(),
        )
        .published_identity()
    }

    fn nz(value: u64) -> NonZeroU64 {
        NonZeroU64::new(value).unwrap()
    }
}
