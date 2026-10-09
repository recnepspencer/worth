use worth_store_physical_format::{BlobRecordKind, BlobSessionFrontierV1, PersistedRecordIdentity};

use super::super::append::append_blob_record;
use super::{
    append_pressure::BlobAppendPressure, BlobIngestFailure, BlobIngestSession, BlobResumeToken,
};

const FRONTIER_INTERVAL: u64 = 64;

/// The durable chunk prefix observed by this attempt, excluding buffered input.
/// A later resume may discover additional selected claims after a failed write.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BlobIngestFrontier {
    bytes: u64,
    next_chunk_ordinal: u64,
}

impl BlobIngestFrontier {
    pub const fn bytes(self) -> u64 {
        self.bytes
    }
    pub const fn next_chunk_ordinal(self) -> u64 {
        self.next_chunk_ordinal
    }
}

pub(super) struct BlobIngestProgress {
    token: BlobResumeToken,
    frontier: BlobIngestFrontier,
    last_chunk: Option<(PersistedRecordIdentity, [u8; 32])>,
    persisted_ordinal: u64,
}

impl BlobIngestProgress {
    pub(super) const fn frontier(&self) -> BlobIngestFrontier {
        self.frontier
    }

    pub(super) fn new(token: BlobResumeToken) -> Self {
        Self {
            token,
            frontier: BlobIngestFrontier::default(),
            last_chunk: None,
            persisted_ordinal: 0,
        }
    }

    pub(super) fn record_chunk(
        &mut self,
        record: PersistedRecordIdentity,
        digest: [u8; 32],
        bytes: u64,
    ) {
        // Counts are already bounded by the admitted declaration before append.
        self.frontier.bytes += bytes;
        self.frontier.next_chunk_ordinal += 1;
        self.last_chunk = Some((record, digest));
    }

    pub(super) fn restore_persisted_frontier(&mut self, ordinal: u64) {
        self.persisted_ordinal = ordinal;
    }
}

impl BlobIngestSession<'_> {
    pub fn resume_token(&self) -> BlobResumeToken {
        self.progress.token
    }

    pub fn frontier(&self) -> BlobIngestFrontier {
        self.progress.frontier
    }

    /// Persists completed chunks through ordinary C5/C8 publication. Does not
    /// flush a partial chunk; an unchanged or empty prefix has no write effect.
    pub fn checkpoint(&mut self) -> Result<BlobIngestFrontier, BlobIngestFailure> {
        if self.poisoned {
            return Err(BlobIngestFailure::Poisoned);
        }
        if let Err(failure) = self.persist_frontier() {
            self.poisoned = true;
            return Err(failure);
        }
        Ok(self.frontier())
    }

    pub(super) fn checkpoint_if_due(&mut self) -> Result<(), BlobIngestFailure> {
        if self.progress.frontier.next_chunk_ordinal - self.progress.persisted_ordinal
            >= FRONTIER_INTERVAL
        {
            self.persist_frontier()?;
        }
        Ok(())
    }

    fn persist_frontier(&mut self) -> Result<(), BlobIngestFailure> {
        let progress = &self.progress;
        let ordinal = progress.frontier.next_chunk_ordinal;
        if ordinal == progress.persisted_ordinal {
            return Ok(());
        }
        let (last_record, last_digest) =
            progress.last_chunk.expect("nonempty prefix has last chunk");
        let token = progress.token;
        let encoded = BlobSessionFrontierV1::new(
            token.store,
            token.session,
            token.declaration_record,
            token.declaration_digest,
            ordinal,
            progress.frontier.bytes,
            last_record,
            last_digest,
        )
        .map_err(BlobIngestFailure::Format)?
        .encode();
        let _pressure = BlobAppendPressure::admit(&mut self.allocation, encoded.len() as u64)
            .map_err(BlobIngestFailure::Memory)?;
        append_blob_record(
            self.runtime,
            self.placement,
            self.session,
            BlobRecordKind::SessionFrontier,
            ordinal,
            self.declaration.deadline(),
            encoded,
        )
        .map_err(|cause| BlobIngestFailure::Append {
            session: self.session,
            kind: BlobRecordKind::SessionFrontier,
            ordinal,
            cause,
        })?;
        self.progress.persisted_ordinal = ordinal;
        Ok(())
    }
}
