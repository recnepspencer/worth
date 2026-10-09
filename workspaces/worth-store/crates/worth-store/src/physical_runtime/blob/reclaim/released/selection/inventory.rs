use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_blob_record, BlobReclaimDescriptorV2, BlobReclaimSourceBasisV1, BlobRecordV1,
    BlobTreeNodeKind, DropSetManifestV3, OriginalDropReservedV1, PersistedRecordIdentity,
    PhysicalTierClass, ReleasedGenerationReclaimBasisV1, SelectedRecordContentClass,
};

use crate::physical_runtime::PhysicalRecordReader;

use super::super::super::{scan, BlobReclaimFailure, BlobReclaimLimits};
use super::transcript;
use super::{merge_basis, require_known_class};

#[derive(Clone, Copy)]
pub(super) enum SelectedBlobFact {
    Other,
    Tree {
        store: [u8; 16],
        session: [u8; 16],
        kind: BlobTreeNodeKind,
        level: u8,
        canonical_digest: [u8; 32],
        covered_bytes: u64,
    },
    Chunk {
        store: [u8; 16],
        session: [u8; 16],
        ordinal: u64,
        content_digest: [u8; 32],
        covered_bytes: u64,
        chunk_size: u32,
    },
    ReuseClaim {
        store: [u8; 16],
        session: [u8; 16],
        ordinal: u64,
        scope: [u8; 32],
        content_digest: [u8; 32],
        covered_bytes: u64,
        chunk_size: u32,
    },
    /// A resume frontier, owned by the session that wrote it.
    Frontier {
        session: [u8; 16],
    },
}

#[derive(Clone, Copy)]
pub(super) struct SelectedReleaseFact {
    pub(super) record: PersistedRecordIdentity,
    pub(super) class: SelectedRecordContentClass,
    pub(super) tier: PhysicalTierClass,
    pub(super) payload_bytes: u64,
    pub(super) frame_sha256: [u8; 32],
    pub(super) blob: SelectedBlobFact,
    pub(super) reachable: bool,
    pub(super) protected: bool,
}

#[derive(Clone, Copy)]
pub(super) struct ManifestLink {
    pub(super) record: PersistedRecordIdentity,
    pub(super) frame_sha256: [u8; 32],
    pub(super) reclaim_attempt: [u8; 16],
    pub(super) count: u16,
    pub(super) manifest_selected_generation: u64,
}

#[derive(Clone, Copy)]
pub(super) struct DescriptorLink {
    pub(super) record: PersistedRecordIdentity,
    pub(super) frame_sha256: [u8; 32],
    pub(super) descriptor: BlobReclaimDescriptorV2,
}

#[derive(Clone, Copy)]
pub(super) struct ReservationLink {
    pub(super) record: PersistedRecordIdentity,
    pub(super) frame_sha256: [u8; 32],
    pub(super) reservation: OriginalDropReservedV1,
}

pub(super) struct SelectedReleaseInventory {
    pub(super) basis: ReleasedGenerationReclaimBasisV1,
    pub(super) publication_selected: bool,
    pub(super) facts: Vec<SelectedReleaseFact>,
    pub(super) manifests: Vec<ManifestLink>,
    pub(super) descriptors: Vec<DescriptorLink>,
    pub(super) reservations: Vec<ReservationLink>,
    pub(super) occupied_attempts: Vec<[u8; 16]>,
    pub(super) selected_route_inventory_sha256: [u8; 32],
}

impl SelectedReleaseInventory {
    pub(super) fn scan(
        reader: PhysicalRecordReader,
        expected_basis: ReleasedGenerationReclaimBasisV1,
        limits: BlobReclaimLimits,
        scratch: &mut [u8],
        work: &mut scan::ReclaimInspectionWork,
    ) -> Result<(PhysicalRecordReader, Self), BlobReclaimFailure> {
        let capacity = usize::try_from(limits.maximum_selected_records())
            .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
        let mut facts = Vec::new();
        let mut manifests = Vec::new();
        let mut descriptors = Vec::new();
        let mut reservations = Vec::new();
        let mut occupied_attempts = Vec::new();
        for reserve in [
            facts.try_reserve_exact(capacity),
            manifests.try_reserve_exact(capacity),
            descriptors.try_reserve_exact(capacity),
            reservations.try_reserve_exact(capacity),
            occupied_attempts.try_reserve_exact(capacity),
        ] {
            reserve.map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
        }
        if [
            facts.capacity(),
            manifests.capacity(),
            descriptors.capacity(),
            reservations.capacity(),
            occupied_attempts.capacity(),
        ]
        .iter()
        .any(|actual| *actual != capacity)
        {
            return Err(BlobReclaimFailure::ScratchUnavailable);
        }
        let store = expected_basis.publication().store();
        let basis_digest =
            BlobReclaimSourceBasisV1::ReleasedGeneration(expected_basis).digest(store);
        let root_generation = reader.protected_root().root().generation().get();
        let mut selected_basis = None;
        let mut publication_selected = false;
        let reader = scan::walk_classified(
            reader,
            limits,
            scratch,
            work,
            |record, class, tier, declared, bytes| {
                require_known_class(class)?;
                let frame_sha256: [u8; 32] = if matches!(class, SelectedRecordContentClass::Blob(_))
                {
                    Sha256::digest(bytes).into()
                } else {
                    [0; 32]
                };
                let mut fact = SelectedReleaseFact {
                    record,
                    class,
                    tier,
                    payload_bytes: declared,
                    frame_sha256,
                    blob: SelectedBlobFact::Other,
                    reachable: false,
                    protected: false,
                };
                if let SelectedRecordContentClass::Blob(_) = class {
                    match decode_blob_record(bytes).map_err(BlobReclaimFailure::Format)? {
                        BlobRecordV1::TreeNode(node) => {
                            fact.blob = SelectedBlobFact::Tree {
                                store: node.occurrence().store(),
                                session: node.occurrence().session(),
                                kind: node.occurrence().kind(),
                                level: node.occurrence().level(),
                                canonical_digest: node.canonical_digest(),
                                covered_bytes: node.covered_bytes(),
                            };
                        }
                        BlobRecordV1::Chunk(chunk) => {
                            fact.blob = SelectedBlobFact::Chunk {
                                store: chunk.occurrence().store(),
                                session: chunk.occurrence().session(),
                                ordinal: chunk.occurrence().ordinal(),
                                content_digest: chunk.stored_digest(),
                                covered_bytes: chunk.bytes().len() as u64,
                                chunk_size: chunk.chunk_size(),
                            };
                        }
                        BlobRecordV1::ChunkReuseClaim(claim) => {
                            fact.blob = SelectedBlobFact::ReuseClaim {
                                store: claim.store(),
                                session: claim.destination_session(),
                                ordinal: claim.destination_ordinal(),
                                scope: claim.scope(),
                                content_digest: claim.stored_digest(),
                                covered_bytes: u64::from(claim.chunk_length()),
                                chunk_size: claim.chunk_size(),
                            };
                        }
                        BlobRecordV1::ChunkReuseClaimV2(value) => {
                            let claim = value.claim();
                            fact.blob = SelectedBlobFact::ReuseClaim {
                                store: claim.store(),
                                session: claim.destination_session(),
                                ordinal: claim.destination_ordinal(),
                                scope: claim.scope(),
                                content_digest: claim.stored_digest(),
                                covered_bytes: u64::from(claim.chunk_length()),
                                chunk_size: claim.chunk_size(),
                            };
                        }
                        BlobRecordV1::SessionFrontier(frontier) => {
                            fact.blob = SelectedBlobFact::Frontier {
                                session: frontier.session(),
                            };
                        }
                        BlobRecordV1::GenerationPublished(publication)
                            if record == expected_basis.publication_record() =>
                        {
                            if publication != expected_basis.publication()
                                || frame_sha256 != expected_basis.publication_frame_sha256()
                            {
                                return Err(BlobReclaimFailure::DeclarationMismatch);
                            }
                            merge_basis(&mut selected_basis, expected_basis)?;
                            publication_selected = true;
                        }
                        BlobRecordV1::DropSetManifestV3(manifest) => {
                            if let BlobReclaimSourceBasisV1::ReleasedGeneration(candidate) =
                                manifest.source_basis()
                            {
                                if candidate == expected_basis {
                                    merge_basis(&mut selected_basis, candidate)?;
                                    manifests.push(manifest_link(record, frame_sha256, &manifest));
                                    occupied_attempts.push(manifest.reclaim_attempt());
                                }
                            }
                        }
                        BlobRecordV1::ReclaimDescriptorV2(descriptor)
                            if descriptor.store() == store
                                && descriptor.source_basis_digest() == basis_digest =>
                        {
                            occupied_attempts.push(descriptor.reclaim_attempt());
                            descriptors.push(DescriptorLink {
                                record,
                                frame_sha256,
                                descriptor,
                            });
                        }
                        BlobRecordV1::ReclaimDescriptorV3(value)
                            if value.base().store() == store
                                && value.base().source_basis_digest() == basis_digest =>
                        {
                            let descriptor = value.base();
                            occupied_attempts.push(descriptor.reclaim_attempt());
                            descriptors.push(DescriptorLink {
                                record,
                                frame_sha256,
                                descriptor,
                            });
                        }
                        BlobRecordV1::ReclaimDescriptorV3(value) => {
                            occupied_attempts.push(value.base().reclaim_attempt());
                        }
                        BlobRecordV1::OriginalDropReserved(reserved)
                            if reserved.store() == store
                                && reserved.source_basis_digest() == basis_digest =>
                        {
                            reservations.push(ReservationLink {
                                record,
                                frame_sha256,
                                reservation: reserved,
                            });
                            occupied_attempts.push(reserved.reclaim_attempt());
                        }
                        _ => {}
                    }
                }
                facts.push(fact);
                Ok(())
            },
        )?;
        if selected_basis != Some(expected_basis) {
            return Err(BlobReclaimFailure::DeclarationMismatch);
        }
        facts.sort_unstable_by_key(|fact| fact.record);
        let selected_route_inventory_sha256 = transcript::selected_routes(root_generation, &facts);
        manifests.sort_unstable_by_key(|link| link.record);
        descriptors.sort_unstable_by_key(|link| link.record);
        occupied_attempts.sort_unstable();
        occupied_attempts.dedup();
        Ok((
            reader,
            Self {
                basis: expected_basis,
                publication_selected,
                facts,
                manifests,
                descriptors,
                reservations,
                occupied_attempts,
                selected_route_inventory_sha256,
            },
        ))
    }

    pub(super) fn fact(&self, record: PersistedRecordIdentity) -> Option<&SelectedReleaseFact> {
        self.facts
            .binary_search_by_key(&record, |fact| fact.record)
            .ok()
            .map(|index| &self.facts[index])
    }

    pub(super) fn fact_mut(
        &mut self,
        record: PersistedRecordIdentity,
    ) -> Option<&mut SelectedReleaseFact> {
        self.facts
            .binary_search_by_key(&record, |fact| fact.record)
            .ok()
            .map(move |index| &mut self.facts[index])
    }
}

fn manifest_link(
    record: PersistedRecordIdentity,
    frame_sha256: [u8; 32],
    manifest: &DropSetManifestV3,
) -> ManifestLink {
    ManifestLink {
        record,
        frame_sha256,
        reclaim_attempt: manifest.reclaim_attempt(),
        count: manifest.count(),
        manifest_selected_generation: manifest.never_reserved_slot_generation(),
    }
}
