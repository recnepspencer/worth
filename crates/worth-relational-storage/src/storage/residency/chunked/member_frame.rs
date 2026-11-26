//! Content-addressed framing of chunk members (head, history, continuation
//! and directory pages). A member's locator is the SHA-256 and length of its
//! whole frame, so any change of encoded bytes is a new member.
use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha256};

use crate::durability::medium::RelationalDurableMemberLocator;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;

pub const MAGIC: &[u8] = b"WORTH-REL-CHUNK1\n";
const HEADER_BYTES: usize = 1;

/// The member kinds one frame may carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkMemberKind {
    Head,
    History,
    Continuation,
    DirectoryPage,
    /// One page of a checkpoint's paged commit index (`CHK24`).
    CommitIndexPage,
}

impl ChunkMemberKind {
    const fn tag(self) -> u8 {
        match self {
            Self::Head => 0,
            Self::History => 1,
            Self::Continuation => 2,
            Self::DirectoryPage => 3,
            Self::CommitIndexPage => 4,
        }
    }
}

/// One encoded member and its content address.
pub struct EncodedChunkMember {
    pub locator: RelationalDurableMemberLocator,
    pub bytes: Vec<u8>,
}

/// Encodes `payload` as a `kind` member no larger than `limit` bytes.
pub fn encode_member(
    kind: ChunkMemberKind,
    payload: &impl Serialize,
    limit: usize,
) -> Result<EncodedChunkMember, Cause> {
    let mut bytes = Vec::with_capacity(MAGIC.len() + HEADER_BYTES);
    bytes.extend_from_slice(MAGIC);
    bytes.push(kind.tag());
    rmp_serde::encode::write(&mut bytes, payload).map_err(|_| Cause::Damaged)?;
    if bytes.len() > limit {
        return Err(Cause::LimitExceeded {
            length: bytes.len() as u64,
            limit: limit as u64,
        });
    }
    Ok(EncodedChunkMember {
        locator: RelationalDurableMemberLocator {
            sha256: Sha256::digest(&bytes).into(),
            length: bytes.len() as u64,
        },
        bytes,
    })
}

/// Decodes a `kind` member after proving it is the content `locator` names.
pub fn decode_member<T: DeserializeOwned>(
    kind: ChunkMemberKind,
    bytes: &[u8],
    locator: RelationalDurableMemberLocator,
) -> Result<T, Cause> {
    if bytes.len() as u64 != locator.length
        || <[u8; 32]>::from(Sha256::digest(bytes)) != locator.sha256
    {
        return Err(Cause::IntegrityMismatch);
    }
    let frame = bytes.strip_prefix(MAGIC).ok_or(Cause::Damaged)?;
    let (header, payload) = frame.split_first().ok_or(Cause::Damaged)?;
    if *header != kind.tag() {
        return Err(Cause::Damaged);
    }
    rmp_serde::from_slice(payload).map_err(|_| Cause::Damaged)
}
