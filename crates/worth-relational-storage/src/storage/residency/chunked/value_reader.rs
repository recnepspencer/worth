//! Bounded streaming of large out-of-line field values.
//!
//! A [`ValueReader`] owns one fixed window of at most `streaming_window_bytes`
//! for its whole life, funded by an in-flight fill reservation. It lends each
//! window as a slice borrowed from the reader, so no caller can retain value
//! bytes past the next read, and it never allocates the whole value.
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::member_io::ChunkMemberSource;
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::residency::working_set::{WorkingSetAccountant, WorkingSetCharge};

/// A large field value: a byte range of a committed member and the digest
/// of exactly those bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValueLocator {
    pub member: RelationalDurableMemberLocator,
    pub offset: u64,
    pub length: u64,
    pub value_sha256: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueReadError {
    /// The in-flight fill ledger cannot fund one more window right now.
    MemoryExhausted {
        requested: u64,
        available: u64,
    },
    /// The value range lies outside its member.
    OutOfBounds,
    /// The streamed bytes are not the value the locator names.
    IntegrityMismatch,
    Member(Cause),
}

pub struct ValueReader<'loan> {
    source: &'loan dyn ChunkMemberSource,
    locator: ValueLocator,
    window: Vec<u8>,
    _fill: WorkingSetCharge,
    position: u64,
    digest: Sha256,
    /// Set once the streamed bytes failed verification; never cleared.
    corrupt: bool,
}

/// Opens a reader over `locator` whose window is charged before allocation.
pub fn open_value_reader<'loan>(
    accountant: &Arc<WorkingSetAccountant>,
    source: &'loan dyn ChunkMemberSource,
    locator: ValueLocator,
) -> Result<ValueReader<'loan>, ValueReadError> {
    let in_member = locator
        .offset
        .checked_add(locator.length)
        .is_some_and(|end| end <= locator.member.length);
    if !in_member {
        return Err(ValueReadError::OutOfBounds);
    }
    let window_limit = u64::from(accountant.budget().streaming_window_bytes).max(1);
    let window_bytes = locator.length.clamp(1, window_limit);
    let fill = accountant.reserve_fill(window_bytes).map_err(|exhausted| {
        ValueReadError::MemoryExhausted {
            requested: exhausted.requested,
            available: exhausted.available,
        }
    })?;
    let window_len =
        usize::try_from(window_bytes).map_err(|_| ValueReadError::MemoryExhausted {
            requested: window_bytes,
            available: 0,
        })?;
    let mut window = Vec::new();
    window
        .try_reserve_exact(window_len)
        .map_err(|_| ValueReadError::Member(Cause::MemoryExhausted))?;
    window.resize(window_len, 0);
    Ok(ValueReader {
        source,
        locator,
        window,
        _fill: fill,
        position: 0,
        digest: Sha256::new(),
        corrupt: false,
    })
}

impl ValueReader<'_> {
    pub fn window_bytes(&self) -> usize {
        self.window.len()
    }

    pub fn remaining(&self) -> u64 {
        self.locator.length - self.position
    }

    /// The next window of the value, or `None` after the last one. The read
    /// that reaches the end verifies the whole value's digest first, so the
    /// final window is lent only for a value proven intact.
    pub fn next_window(&mut self) -> Result<Option<&[u8]>, ValueReadError> {
        if self.corrupt {
            return Err(ValueReadError::IntegrityMismatch);
        }
        let remaining = self.remaining();
        if remaining == 0 {
            return Ok(None);
        }
        let take = remaining.min(self.window.len() as u64) as usize;
        let mut filled = 0;
        while filled < take {
            let offset = self.locator.offset + self.position + filled as u64;
            match self
                .source
                .read_range(self.locator.member, offset, &mut self.window[filled..take])
                .map_err(ValueReadError::Member)?
            {
                0 => return Err(ValueReadError::Member(Cause::MissingMember)),
                read => filled += read,
            }
        }
        self.digest.update(&self.window[..take]);
        self.position += take as u64;
        if self.position == self.locator.length {
            let digest = std::mem::take(&mut self.digest).finalize();
            if <[u8; 32]>::from(digest) != self.locator.value_sha256 {
                self.corrupt = true;
                return Err(ValueReadError::IntegrityMismatch);
            }
        }
        Ok(Some(&self.window[..take]))
    }
}
