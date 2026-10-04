use std::num::NonZeroU64;

use worth_store_physical_format::{BlobRecordDenial, PersistedRecordIdentity};

use crate::physical_runtime::{
    BlobSessionId, RecordReadError, RecordScanError, RecordStreamFailure,
};

use super::super::selected_session::SelectedSessionFailure;
use super::super::BlobIngestClaimDenial;
use crate::physical_runtime::blob::BlobAppendFailure;

/// A bounded selected-root inspection; exhaustion is never absence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobTerminalLimits {
    maximum_scanned_records: NonZeroU64,
}

impl BlobTerminalLimits {
    pub const fn new(maximum_scanned_records: NonZeroU64) -> Self {
        Self {
            maximum_scanned_records,
        }
    }

    pub const fn maximum_scanned_records(self) -> NonZeroU64 {
        self.maximum_scanned_records
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobTerminalDisposition {
    NewlyAbandoned,
    AlreadyAbandoned,
}

/// The selected C.5 terminal record is the durable result. The disposition
/// distinguishes this call's append from an already selected terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobTerminalReceipt {
    session: BlobSessionId,
    record: PersistedRecordIdentity,
    disposition: BlobTerminalDisposition,
}

impl BlobTerminalReceipt {
    pub(super) const fn new(
        session: BlobSessionId,
        record: PersistedRecordIdentity,
        disposition: BlobTerminalDisposition,
    ) -> Self {
        Self {
            session,
            record,
            disposition,
        }
    }

    pub const fn session(self) -> BlobSessionId {
        self.session
    }

    pub const fn record(self) -> PersistedRecordIdentity {
        self.record
    }

    pub const fn disposition(self) -> BlobTerminalDisposition {
        self.disposition
    }
}

#[derive(Debug)]
pub enum BlobTerminalFailure {
    Claim(BlobIngestClaimDenial),
    ServingRequiresInspection,
    PendingPublication,
    Format(BlobRecordDenial),
    Read(RecordReadError),
    Stream(RecordStreamFailure),
    Scan(RecordScanError),
    ForeignStore,
    DeclarationMismatch,
    ScopeMismatch,
    AlreadyPublished,
    /// Store released the generation this session published. Its fate is
    /// that release, never an abandonment.
    AlreadyReleased,
    ConflictingSelectedFate,
    NotExpired {
        selected_checkpoint_sequence: u64,
        maximum_checkpoint_sequence: u64,
    },
    ScanBoundExhausted,
    Append(BlobAppendFailure),
}

impl From<SelectedSessionFailure> for BlobTerminalFailure {
    fn from(cause: SelectedSessionFailure) -> Self {
        match cause {
            SelectedSessionFailure::Format(cause) => Self::Format(cause),
            SelectedSessionFailure::Read(cause) => Self::Read(cause),
            SelectedSessionFailure::Stream(cause) => Self::Stream(cause),
            SelectedSessionFailure::ForeignStore => Self::ForeignStore,
            SelectedSessionFailure::DeclarationMismatch => Self::DeclarationMismatch,
            SelectedSessionFailure::ScopeMismatch => Self::ScopeMismatch,
        }
    }
}
