use worth_store_physical_format::{
    DropSetManifestV2, FailedIngestReclaimBasisV1, OriginalDropProofV1, PersistedRecordIdentity,
    ReservedDropRecordV1, RootPublicationCell,
};

use crate::physical_runtime::durability::{
    DisplacedArtifact, PhysicalOriginalDropNoEffect, PhysicalRecoveredOriginalDropNoDurableEffect,
};

use super::PhysicalReconciledReclaimDescriptorFate;

pub(in crate::physical_runtime) enum SelectedOriginalDropProof {
    V1(PhysicalOriginalDropNoEffect),
    V2NeverReserved(DropSetManifestV2),
    V2ProvenNoEffect {
        original: PhysicalOriginalDropNoEffect,
        reserved: ReservedDropRecordV1,
        reserved_selected_generation: u64,
    },
    V2RecoveredNoBinding {
        recovered: PhysicalRecoveredOriginalDropNoDurableEffect,
        reserved: ReservedDropRecordV1,
    },
}

#[derive(Clone, Copy)]
pub(in crate::physical_runtime) enum ManifestResidueProof {
    ProvenNoEffect {
        fate: PhysicalReconciledReclaimDescriptorFate,
        reserved: Option<ReservedDropRecordV1>,
    },
    NeverReserved {
        store: [u8; 16],
        attempt: [u8; 16],
        basis_digest: [u8; 32],
        manifest: PersistedRecordIdentity,
        manifest_sha256: [u8; 32],
        selected_root: RootPublicationCell,
        manifest_selected_generation: u64,
    },
    RecoveredNoBinding {
        recovered: PhysicalRecoveredOriginalDropNoDurableEffect,
        reserved: ReservedDropRecordV1,
    },
}

impl ManifestResidueProof {
    pub(in crate::physical_runtime) fn positive(
        fate: PhysicalReconciledReclaimDescriptorFate,
        reserved: Option<ReservedDropRecordV1>,
    ) -> Self {
        Self::ProvenNoEffect { fate, reserved }
    }

    /// The Store selected-root scanner may issue this only after proving the
    /// exact V2 manifest is selected and no matching Reserved record exists.
    pub(in crate::physical_runtime) fn never_reserved(
        manifest: &DropSetManifestV2,
        record: PersistedRecordIdentity,
        sha256: [u8; 32],
        selected_root: RootPublicationCell,
    ) -> Option<Self> {
        (sha256 != [0; 32]
            && manifest.never_reserved_slot_generation() <= selected_root.generation().get())
        .then_some(Self::NeverReserved {
            store: manifest.store(),
            attempt: manifest.reclaim_attempt(),
            basis_digest: manifest.source_basis().digest(manifest.store()),
            manifest: record,
            manifest_sha256: sha256,
            selected_root,
            manifest_selected_generation: manifest.never_reserved_slot_generation(),
        })
    }

    pub(in crate::physical_runtime) fn matches(
        self,
        store: [u8; 16],
        attempt: [u8; 16],
        basis: FailedIngestReclaimBasisV1,
        manifest: PersistedRecordIdentity,
        sha256: [u8; 32],
        selected_root: RootPublicationCell,
    ) -> bool {
        match self {
            Self::ProvenNoEffect { fate, .. } => {
                fate.matches(store, attempt, basis, manifest, sha256, selected_root)
            }
            Self::NeverReserved {
                store: expected_store,
                attempt: expected_attempt,
                basis_digest,
                manifest: expected_manifest,
                manifest_sha256,
                selected_root: expected_root,
                manifest_selected_generation,
            } => {
                expected_store == store
                    && expected_attempt == attempt
                    && basis_digest == basis.digest(store)
                    && expected_manifest == manifest
                    && manifest_sha256 == sha256
                    && expected_root == selected_root
                    && manifest_selected_generation <= selected_root.generation().get()
            }
            Self::RecoveredNoBinding {
                recovered,
                reserved,
            } => {
                let original = recovered.reservation();
                original.store() == store
                    && original.reclaim_attempt() == attempt
                    && original.source_basis_digest() == basis.digest(store)
                    && original.manifest_record() == manifest
                    && original.manifest_frame_sha256() == sha256
                    && recovered.reservation_record() == reserved.record()
                    && recovered.reservation_sha256() == reserved.frame_sha256()
                    && recovered.selected_root() == selected_root
            }
        }
    }

    pub(in crate::physical_runtime) fn attempt(self) -> [u8; 16] {
        match self {
            Self::ProvenNoEffect { fate, .. } => fate.original_attempt,
            Self::NeverReserved { attempt, .. } => attempt,
            Self::RecoveredNoBinding { recovered, .. } => recovered.reservation().reclaim_attempt(),
        }
    }

    pub(in crate::physical_runtime) fn wire_proof(self) -> OriginalDropProofV1 {
        match self {
            Self::ProvenNoEffect { fate, reserved } => OriginalDropProofV1::ProvenNoEffect {
                idempotency: fate.drop_idempotency,
                fingerprint: fate.drop_fingerprint,
                reserved,
            },
            Self::NeverReserved { .. } => OriginalDropProofV1::NeverReserved,
            Self::RecoveredNoBinding {
                recovered,
                reserved,
            } => OriginalDropProofV1::RecoveredNoBinding {
                idempotency: recovered.reservation().request().idempotency(),
                fingerprint: recovered.reservation().request().fingerprint(),
                reserved,
            },
        }
    }

    pub(in crate::physical_runtime) fn reserved(self) -> Option<ReservedDropRecordV1> {
        match self {
            Self::ProvenNoEffect { reserved, .. } => reserved,
            Self::NeverReserved { .. } => None,
            Self::RecoveredNoBinding { reserved, .. } => Some(reserved),
        }
    }

    pub(in crate::physical_runtime) fn registry_runtime(
        self,
    ) -> Option<crate::physical_runtime::RuntimeIdentity> {
        match self {
            Self::RecoveredNoBinding { recovered, .. } => Some(recovered.registry_runtime()),
            _ => None,
        }
    }
}

/// Fixed one- or two-extent displacement avoids allocating another roster
/// before the root-only WAL effect begins.
#[derive(Clone, Copy)]
pub(in crate::physical_runtime) struct ManifestResidueDisplacement {
    pub(in crate::physical_runtime) manifest: DisplacedArtifact,
    pub(in crate::physical_runtime) reserved: Option<DisplacedArtifact>,
}

impl ManifestResidueDisplacement {
    pub(in crate::physical_runtime) fn count(self) -> u32 {
        1 + u32::from(self.reserved.is_some())
    }

    pub(in crate::physical_runtime) fn bytes(self) -> Option<u64> {
        self.manifest
            .bytes
            .checked_add(self.reserved.map_or(0, |value| value.bytes))
    }
}
