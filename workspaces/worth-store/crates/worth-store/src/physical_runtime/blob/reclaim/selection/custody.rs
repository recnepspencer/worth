use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_blob_record, BlobRecordV1, BlobSessionDeclarationV1, PersistedRecordIdentity,
};

use crate::physical_runtime::{durability::CompletedDurableCheckpointWitness, BlobResumeToken};

use super::super::super::ingest::selected_session::selected_abandonment_matches;
use super::super::BlobReclaimFailure;

pub(in crate::physical_runtime::blob::reclaim) struct ResidueCandidate {
    pub(super) record: PersistedRecordIdentity,
    pub(super) chunk_ordinal: Option<u64>,
    pub(super) referenced: bool,
}

pub(super) fn observe_custody(
    record: PersistedRecordIdentity,
    bytes: &[u8],
    token: BlobResumeToken,
    declaration: BlobSessionDeclarationV1,
    completed: Option<CompletedDurableCheckpointWitness>,
    declaration_seen: &mut bool,
    abandoned: &mut Option<(PersistedRecordIdentity, [u8; 32])>,
    candidates: &mut Vec<ResidueCandidate>,
) -> Result<(), BlobReclaimFailure> {
    if !bytes.starts_with(b"WRC11BLB") {
        return Ok(());
    }
    let selected = decode_blob_record(bytes).map_err(BlobReclaimFailure::Format)?;
    let mut chunk_ordinal = None;
    let candidate = match selected {
        BlobRecordV1::SessionDeclared(value) => {
            if value.session() == token.session {
                if *declaration_seen || record != token.declaration_record || value != declaration {
                    return Err(BlobReclaimFailure::ConflictingSelectedFate);
                }
                *declaration_seen = true;
            } else if value.object() == declaration.object() {
                return Err(BlobReclaimFailure::ConflictingSelectedFate);
            }
            false
        }
        BlobRecordV1::SessionAbandoned(value) => {
            if value.session() == token.session
                || value.declaration_record() == token.declaration_record
            {
                if !selected_abandonment_matches(value, &token, declaration, completed)
                    || abandoned
                        .replace((record, Sha256::digest(bytes).into()))
                        .is_some()
                {
                    return Err(BlobReclaimFailure::ConflictingSelectedFate);
                }
            }
            false
        }
        BlobRecordV1::GenerationPublished(value) => {
            if value.session() == token.session || value.object() == declaration.object() {
                return Err(BlobReclaimFailure::AlreadyPublished);
            }
            false
        }
        BlobRecordV1::Chunk(value) => {
            let occurrence = value.occurrence();
            if occurrence.session() != token.session {
                false
            } else {
                let start = occurrence
                    .ordinal()
                    .checked_mul(u64::from(declaration.chunk_size()))
                    .ok_or(BlobReclaimFailure::ConflictingSelectedFate)?;
                let expected = declaration
                    .declared_bytes()
                    .checked_sub(start)
                    .filter(|bytes| *bytes > 0)
                    .ok_or(BlobReclaimFailure::ConflictingSelectedFate)?
                    .min(u64::from(declaration.chunk_size()));
                if occurrence.store() != token.store
                    || value.chunk_size() != declaration.chunk_size()
                    || value.bytes().len() as u64 != expected
                {
                    return Err(BlobReclaimFailure::ConflictingSelectedFate);
                }
                chunk_ordinal = Some(occurrence.ordinal());
                true
            }
        }
        BlobRecordV1::TreeNode(value) => {
            if value.occurrence().session() != token.session {
                false
            } else {
                if value.occurrence().store() != token.store {
                    return Err(BlobReclaimFailure::ConflictingSelectedFate);
                }
                true
            }
        }
        BlobRecordV1::SessionFrontier(value) => {
            if value.session() != token.session
                && value.declaration_record() != token.declaration_record
            {
                false
            } else {
                if value.store() != token.store
                    || value.session() != token.session
                    || value.declaration_record() != token.declaration_record
                    || value.declaration_digest() != token.declaration_digest
                {
                    return Err(BlobReclaimFailure::ConflictingSelectedFate);
                }
                true
            }
        }
        // Reclaim metadata has its own recovery-frontier lifetime. It is not
        // payload residue merely because it records this failed session.
        BlobRecordV1::DropSetManifest(_)
        | BlobRecordV1::DropSetManifestV2(_)
        | BlobRecordV1::DropSetManifestV3(_)
        | BlobRecordV1::OriginalDropReserved(_)
        | BlobRecordV1::ReclaimDescriptor(_)
        | BlobRecordV1::ReclaimDescriptorV2(_)
        | BlobRecordV1::ReclaimDescriptorV3(_)
        | BlobRecordV1::DedupeQuarantine(_) => false,
        value @ (BlobRecordV1::ChunkReuseClaim(_) | BlobRecordV1::ChunkReuseClaimV2(_)) => {
            let reuse = match value {
                BlobRecordV1::ChunkReuseClaim(value) => value,
                BlobRecordV1::ChunkReuseClaimV2(value) => value.claim(),
                _ => unreachable!(),
            };
            if reuse.destination_session() != token.session {
                false
            } else {
                let start = reuse
                    .destination_ordinal()
                    .checked_mul(u64::from(declaration.chunk_size()))
                    .ok_or(BlobReclaimFailure::ConflictingSelectedFate)?;
                let expected = declaration
                    .declared_bytes()
                    .checked_sub(start)
                    .filter(|bytes| *bytes > 0)
                    .ok_or(BlobReclaimFailure::ConflictingSelectedFate)?
                    .min(u64::from(declaration.chunk_size()));
                if reuse.store() != token.store
                    || reuse.scope() != declaration.key_scope()
                    || reuse.chunk_size() != declaration.chunk_size()
                    || u64::from(reuse.chunk_length()) != expected
                {
                    return Err(BlobReclaimFailure::ConflictingSelectedFate);
                }
                chunk_ordinal = Some(reuse.destination_ordinal());
                true
            }
        }
    };
    if candidate {
        candidates.push(ResidueCandidate {
            record,
            chunk_ordinal,
            referenced: false,
        });
    }
    Ok(())
}
