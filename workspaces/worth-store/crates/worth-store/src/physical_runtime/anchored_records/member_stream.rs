//! Caller-owned scratch streaming of one physically protected selected member.
use super::{
    AnchoredRecordMemberDamage as Damage, AnchoredRecordMemberLocator,
    AnchoredRecordMemberReadFailure as Failure,
};
use crate::physical_runtime::RecordReadSession;
use sha2::{Digest, Sha256};

/// Lower-level integrity streaming, not an execution-controlled operation.
/// It neither retains a request nor latches refusals. The owning operation
/// must poll its SAME original request before every read, use bounded caller
/// scratch, and stop on any error. Only verified EOF validates the whole member.
/// Generic disposal uses ordinary unbudgeted physical protection release.
pub struct AnchoredRecordMemberStream {
    session: RecordReadSession,
    locator: AnchoredRecordMemberLocator,
    remaining: u64,
    hash: Sha256,
    complete: bool,
}
impl AnchoredRecordMemberStream {
    pub(super) fn new(session: RecordReadSession, locator: AnchoredRecordMemberLocator) -> Self {
        Self {
            session,
            locator,
            remaining: locator.length(),
            hash: Sha256::new(),
            complete: false,
        }
    }
    /// `Ok(0)` is terminal verified EOF; consume through this result before
    /// treating the bytes as an intact member. Empty output while payload
    /// remains is an error, not EOF. A refusal is not latched by this primitive;
    /// callers must retain their owning operation and stop reading on error.
    pub fn read_next(&mut self, buffer: &mut [u8]) -> Result<usize, Failure> {
        if self.complete {
            return Ok(0);
        }
        if self.remaining == 0 {
            if self
                .session
                .read_next(&mut [0; 1])
                .map_err(Failure::Stream)?
                != 0
            {
                return Err(Failure::Damaged(Damage::TrailingBytes));
            }
            if <[u8; 32]>::from(self.hash.clone().finalize()) != self.locator.sha256() {
                return Err(Failure::Damaged(Damage::DigestMismatch));
            }
            self.complete = true;
            return Ok(0);
        }
        if buffer.is_empty() {
            return Err(Failure::Damaged(Damage::Truncated));
        }
        let length = usize::try_from(self.remaining)
            .unwrap_or(usize::MAX)
            .min(buffer.len());
        let count = self
            .session
            .read_next(&mut buffer[..length])
            .map_err(Failure::Stream)?;
        if count == 0 {
            return Err(Failure::Damaged(Damage::Truncated));
        }
        self.hash.update(&buffer[..count]);
        self.remaining -= count as u64;
        Ok(count)
    }
}
