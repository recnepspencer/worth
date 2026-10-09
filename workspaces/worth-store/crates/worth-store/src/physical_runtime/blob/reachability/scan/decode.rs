use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_blob_record, BTreeNodeV1, BlobReclaimSourceBasisV1, BlobRecordV1,
    BlobSessionDeclarationV1, DerivedFamilyRootDirectoryV1, PersistedRecordIdentity,
    SelectedRecordContentClass,
};

use crate::physical_runtime::blob::ingest::SelectedResumeClaim;

use super::{
    closure::{ClosureFact, TreeLink},
    control::ControlFact,
    reuse::ReuseSource,
    Role,
};
use crate::physical_runtime::blob::reachability::BlobReachabilityFailure as Failure;

pub(super) fn decode_selected(
    class: SelectedRecordContentClass,
    bytes: &[u8],
    record: PersistedRecordIdentity,
) -> Result<
    (
        Role,
        Vec<PersistedRecordIdentity>,
        Option<BlobSessionDeclarationV1>,
        Option<SelectedResumeClaim>,
        Option<ClosureFact>,
        Option<ControlFact>,
        Option<ReuseSource>,
    ),
    Failure,
> {
    match class {
        SelectedRecordContentClass::Blob(expected) => {
            let decoded = decode_blob_record(bytes).map_err(Failure::Format)?;
            if decoded.kind() != expected {
                return Err(Failure::ConflictingSelectedFate);
            }
            let mut declaration = None;
            let mut claim = None;
            let mut closure = None;
            let mut control = None;
            let mut reuse = None;
            let (role, edges) = match decoded {
                BlobRecordV1::SessionDeclared(value) => {
                    declaration = Some(value);
                    (Role::Declaration(value.session()), vec![])
                }
                BlobRecordV1::Chunk(value) => {
                    closure = Some(ClosureFact::Chunk {
                        store: value.occurrence().store(),
                        session: value.occurrence().session(),
                        ordinal: value.occurrence().ordinal(),
                        content_digest: value.stored_digest(),
                        covered_bytes: value.bytes().len() as u64,
                        chunk_size: value.chunk_size(),
                        scope: None,
                    });
                    claim = Some(SelectedResumeClaim::Chunk {
                        ordinal: value.occurrence().ordinal(),
                        record,
                        digest: value.stored_digest(),
                        bytes: value.bytes().len() as u64,
                    });
                    (
                        Role::Chunk {
                            session: value.occurrence().session(),
                            ordinal: value.occurrence().ordinal(),
                        },
                        vec![],
                    )
                }
                BlobRecordV1::TreeNode(value) => {
                    let occurrence = value.occurrence();
                    let mut links = Vec::new();
                    links
                        .try_reserve_exact(value.entries().len())
                        .map_err(|_| Failure::MetadataUnavailable)?;
                    links.extend(value.entries().iter().map(|entry| TreeLink {
                        record: entry.record(),
                        digest: entry.digest(),
                        covered_bytes: entry.covered_bytes(),
                    }));
                    closure = Some(ClosureFact::Tree {
                        store: occurrence.store(),
                        session: occurrence.session(),
                        kind: occurrence.kind(),
                        level: occurrence.level(),
                        covered_bytes: value.covered_bytes(),
                        frame_digest: Sha256::digest(bytes).into(),
                        canonical_digest: value.canonical_digest(),
                        links,
                    });
                    claim = Some(SelectedResumeClaim::Node {
                        ordinal: (u64::from(occurrence.level()) << 56) | occurrence.index(),
                        record,
                        frame_digest: Sha256::digest(bytes).into(),
                    });
                    (
                        Role::Tree(occurrence.session()),
                        value.entries().iter().map(|entry| entry.record()).collect(),
                    )
                }
                BlobRecordV1::GenerationPublished(value) => {
                    closure = Some(ClosureFact::Publication {
                        store: value.store(),
                        session: value.session(),
                        root: value.root_record(),
                        root_digest: value.root_digest(),
                        frame_digest: Sha256::digest(bytes).into(),
                        total_bytes: value.total_bytes(),
                        chunk_size: value.chunk_size(),
                        scope: value.key_scope(),
                    });
                    (
                        Role::Publication(value.session()),
                        vec![value.root_record()],
                    )
                }
                BlobRecordV1::SessionFrontier(value) => {
                    claim = Some(SelectedResumeClaim::Frontier {
                        ordinal: value.next_chunk_ordinal(),
                        durable_bytes: value.durable_bytes(),
                        last_record: value.last_chunk_record(),
                        last_digest: value.last_chunk_digest(),
                    });
                    (
                        Role::Frontier {
                            session: value.session(),
                            next_ordinal: value.next_chunk_ordinal(),
                        },
                        vec![value.last_chunk_record()],
                    )
                }
                BlobRecordV1::SessionAbandoned(value) => {
                    (Role::Abandonment(value.session()), vec![])
                }
                BlobRecordV1::ChunkReuseClaim(value) => {
                    reuse = Some(ReuseSource {
                        publication_record: value.source_publication(),
                        selected_chunk: value.selected_chunk(),
                        source_ordinal: value.source_ordinal(),
                        store: value.store(),
                        scope: value.scope(),
                        chunk_size: value.chunk_size(),
                        chunk_length: value.chunk_length(),
                        stored_digest: value.stored_digest(),
                        witnessed_publication: None,
                        witnessed_publication_digest: None,
                    });
                    closure = Some(ClosureFact::Chunk {
                        store: value.store(),
                        session: value.destination_session(),
                        ordinal: value.destination_ordinal(),
                        content_digest: value.stored_digest(),
                        covered_bytes: u64::from(value.chunk_length()),
                        chunk_size: value.chunk_size(),
                        scope: Some(value.scope()),
                    });
                    claim = Some(SelectedResumeClaim::ReusedChunk {
                        ordinal: value.destination_ordinal(),
                        record,
                        digest: value.stored_digest(),
                        bytes: u64::from(value.chunk_length()),
                    });
                    (
                        Role::ReuseClaim {
                            session: value.destination_session(),
                            ordinal: value.destination_ordinal(),
                        },
                        vec![value.source_publication(), value.selected_chunk()],
                    )
                }
                BlobRecordV1::ChunkReuseClaimV2(value) => {
                    let claim_value = value.claim();
                    let source = ReuseSource {
                        publication_record: claim_value.source_publication(),
                        selected_chunk: claim_value.selected_chunk(),
                        source_ordinal: claim_value.source_ordinal(),
                        store: claim_value.store(),
                        scope: claim_value.scope(),
                        chunk_size: claim_value.chunk_size(),
                        chunk_length: claim_value.chunk_length(),
                        stored_digest: claim_value.stored_digest(),
                        witnessed_publication: Some(value.source_publication()),
                        witnessed_publication_digest: Some(value.source_publication_frame_sha256()),
                    };
                    reuse = Some(source);
                    closure = Some(ClosureFact::Chunk {
                        store: claim_value.store(),
                        session: claim_value.destination_session(),
                        ordinal: claim_value.destination_ordinal(),
                        content_digest: claim_value.stored_digest(),
                        covered_bytes: u64::from(claim_value.chunk_length()),
                        chunk_size: claim_value.chunk_size(),
                        scope: Some(claim_value.scope()),
                    });
                    claim = Some(SelectedResumeClaim::ReusedChunk {
                        ordinal: claim_value.destination_ordinal(),
                        record,
                        digest: claim_value.stored_digest(),
                        bytes: u64::from(claim_value.chunk_length()),
                    });
                    (
                        Role::ReuseClaim {
                            session: claim_value.destination_session(),
                            ordinal: claim_value.destination_ordinal(),
                        },
                        vec![
                            claim_value.source_publication(),
                            claim_value.selected_chunk(),
                        ],
                    )
                }
                BlobRecordV1::DedupeQuarantine(value) => (
                    Role::Control,
                    vec![
                        value.source_publication(),
                        value.source_chunk(),
                        value.conflicting_chunk(),
                    ],
                ),
                BlobRecordV1::DropSetManifestV3(value) => match value.source_basis() {
                    BlobReclaimSourceBasisV1::ReleasedGeneration(basis) => {
                        control = Some(ControlFact::ReleasedManifest {
                            attempt: value.reclaim_attempt(),
                            basis,
                            frame_digest: Sha256::digest(bytes).into(),
                            source_basis_digest: value.source_basis_digest(),
                        });
                        (Role::ReleasedControl(basis.session()), vec![])
                    }
                    BlobReclaimSourceBasisV1::FailedIngest(_) => (Role::Control, vec![]),
                },
                BlobRecordV1::ReclaimDescriptorV3(value) => {
                    let base = value.base();
                    control = Some(ControlFact::ReleasedDescriptor {
                        store: base.store(),
                        attempt: base.reclaim_attempt(),
                        manifest: base.manifest_record(),
                        manifest_digest: base.manifest_frame_sha256(),
                        source_basis_digest: base.source_basis_digest(),
                        request: value.custody().request(),
                    });
                    (Role::Control, vec![])
                }
                BlobRecordV1::OriginalDropReserved(value) => {
                    control = Some(ControlFact::Reservation {
                        store: value.store(),
                        attempt: value.reclaim_attempt(),
                        manifest: value.manifest_record(),
                        manifest_digest: value.manifest_frame_sha256(),
                        source_basis_digest: value.source_basis_digest(),
                        request: value.request(),
                    });
                    (Role::Control, vec![])
                }
                _ => (Role::Control, vec![]),
            };
            Ok((role, edges, declaration, claim, closure, control, reuse))
        }
        SelectedRecordContentClass::DerivedDirectory => {
            let directory =
                DerivedFamilyRootDirectoryV1::decode(bytes).map_err(Failure::DerivedDirectory)?;
            Ok((
                Role::Derived,
                directory
                    .entries()
                    .iter()
                    .map(|entry| entry.root_record())
                    .collect(),
                None,
                None,
                None,
                None,
                None,
            ))
        }
        SelectedRecordContentClass::BTreeNode { family_code } => {
            let node = BTreeNodeV1::decode(bytes).map_err(Failure::BTree)?;
            if node.family_code() != family_code {
                return Err(Failure::ConflictingSelectedFate);
            }
            let mut edges = Vec::new();
            if let Some(first) = node.first_child() {
                edges.push(first);
            }
            edges.extend(node.cells().iter().filter_map(|cell| cell.child()));
            Ok((Role::Derived, edges, None, None, None, None, None))
        }
        SelectedRecordContentClass::Opaque => {
            if bytes.starts_with(b"WRC11BLB") {
                return Err(Failure::ConflictingSelectedFate);
            }
            Ok((Role::Opaque, vec![], None, None, None, None, None))
        }
        SelectedRecordContentClass::UnknownLegacy => Err(Failure::ConflictingSelectedFate),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opaque_route_cannot_hide_a_blob_frame() {
        let record = PersistedRecordIdentity::new([7; 16], 1).unwrap();
        assert!(matches!(
            decode_selected(SelectedRecordContentClass::Opaque, b"WRC11BLBspoof", record),
            Err(Failure::ConflictingSelectedFate)
        ));
        assert!(matches!(
            decode_selected(
                SelectedRecordContentClass::Opaque,
                b"foreign-record",
                record
            ),
            Ok((Role::Opaque, _, _, _, _, _, _))
        ));
    }
}
