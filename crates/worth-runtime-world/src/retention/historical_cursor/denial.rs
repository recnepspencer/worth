//! Exhaustive typed historical read and resume refusal implementing
//! [`worth_execution::report::Reportable`] (`plans/database-foundation/milestone-4.md` §§3.4, 8).

use core::fmt;

use worth_execution::report::{DenialClass, Disposition, ReportCode, Reportable, RetryHint};
use worth_relational::facade::history::CommitId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::storage::RelationalStorageSubject;

use super::key::SchemaVersionIdentity;

const CODE_HISTORY_UNAVAILABLE: ReportCode =
    match ReportCode::new("worth.store.historical_read.history_unavailable") {
        Ok(code) => code,
        Err(_) => panic!("valid static report code"),
    };
const CODE_UNSUPPORTED_MEANING: ReportCode =
    match ReportCode::new("worth.store.historical_read.unsupported_meaning") {
        Ok(code) => code,
        Err(_) => panic!("valid static report code"),
    };
const CODE_MALFORMED_TOKEN: ReportCode =
    match ReportCode::new("worth.store.historical_read.malformed_token") {
        Ok(code) => code,
        Err(_) => panic!("valid static report code"),
    };
const CODE_CORRUPT_STORAGE: ReportCode =
    match ReportCode::new("worth.store.historical_read.corrupt_storage") {
        Ok(code) => code,
        Err(_) => panic!("valid static report code"),
    };
const CODE_MEMORY_EXHAUSTED: ReportCode =
    match ReportCode::new("worth.store.historical_read.memory_exhausted") {
        Ok(code) => code,
        Err(_) => panic!("valid static report code"),
    };
const CODE_REQUEST_STOPPED: ReportCode =
    match ReportCode::new("worth.store.historical_read.request_stopped") {
        Ok(code) => code,
        Err(_) => panic!("valid static report code"),
    };
const CODE_TOKEN_INTEGRITY: ReportCode =
    match ReportCode::new("worth.store.retained_basis_token.integrity_mismatch") {
        Ok(code) => code,
        Err(_) => panic!("valid static report code"),
    };

/// Typed refusal returned when decoding an untrusted `whc1_...` token string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetainedBasisTokenDecodeError {
    InvalidPrefix,
    InvalidEncoding,
    Truncated,
    TrailingBytes,
    UnsupportedVersion { found: u16 },
    IntegrityMismatch,
    MalformedField,
}

impl fmt::Display for RetainedBasisTokenDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.diagnostic_summary())
    }
}

impl std::error::Error for RetainedBasisTokenDecodeError {}

/// Exhaustive typed historical read/resume refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoricalReadDenial {
    /// Basis was legitimately pruned past the retention horizon (or unretained
    /// across restart). Disposition: [`Disposition::Final`] — never silently
    /// rebinds to `HEAD`.
    HistoryUnavailable { commit_id: CommitId },
    /// Schema/version boundary, retention conflict, or query contract makes the
    /// requested historical meaning unsupported. Disposition: [`Disposition::CorrectInput`].
    UnsupportedHistoricalMeaning {
        schema_version: SchemaVersionIdentity,
    },
    /// Caller presented a syntactically malformed, wrong-prefix, oversized, or
    /// unsupported-version wire token. Disposition: [`Disposition::CorrectInput`]
    /// (never escalates malformed untrusted wire input as storage corruption).
    MalformedToken(RetainedBasisTokenDecodeError),
    /// Persisted bytes, Merkle commitment, token MAC/integrity, or home
    /// incarnation failed validation. Disposition: [`Disposition::Escalate`].
    CorruptStorage { subject: RelationalStorageSubject },
    /// Active resident borrows or a single row's streaming window genuinely
    /// saturated the configured historical working-set/window budget.
    /// Disposition: [`Disposition::Retry`] with [`RetryHint::AfterBackoff`].
    MemoryExhausted {
        requested_bytes: u64,
        allowance_bytes: u64,
    },
    /// Caller cancelled or exceeded execution deadline before completing read.
    /// Disposition: [`Disposition::Retry`] with [`RetryHint::Immediate`].
    RequestStopped,
}

impl fmt::Display for HistoricalReadDenial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.diagnostic_summary())
    }
}

impl std::error::Error for HistoricalReadDenial {}

impl Reportable for HistoricalReadDenial {
    fn denial_class(&self) -> DenialClass {
        match self {
            Self::HistoryUnavailable { .. } => DenialClass::NotFound,
            Self::UnsupportedHistoricalMeaning { .. } => DenialClass::InvalidInput,
            Self::MalformedToken(error) => error.denial_class(),
            Self::CorruptStorage { .. } => DenialClass::Defect,
            Self::MemoryExhausted { .. } => DenialClass::Exhausted,
            Self::RequestStopped => DenialClass::Cancelled,
        }
    }

    fn disposition(&self) -> Disposition {
        match self {
            Self::HistoryUnavailable { .. } => Disposition::Final,
            Self::UnsupportedHistoricalMeaning { .. } => Disposition::CorrectInput,
            Self::MalformedToken(error) => error.disposition(),
            Self::CorruptStorage { .. } => Disposition::Escalate,
            Self::MemoryExhausted { .. } => Disposition::Retry(RetryHint::AfterBackoff),
            Self::RequestStopped => Disposition::Retry(RetryHint::Immediate),
        }
    }

    fn report_code(&self) -> ReportCode {
        match self {
            Self::HistoryUnavailable { .. } => CODE_HISTORY_UNAVAILABLE,
            Self::UnsupportedHistoricalMeaning { .. } => CODE_UNSUPPORTED_MEANING,
            Self::MalformedToken(error) => error.report_code(),
            Self::CorruptStorage { .. } => CODE_CORRUPT_STORAGE,
            Self::MemoryExhausted { .. } => CODE_MEMORY_EXHAUSTED,
            Self::RequestStopped => CODE_REQUEST_STOPPED,
        }
    }

    fn diagnostic_summary(&self) -> &'static str {
        match self {
            Self::HistoryUnavailable { .. } => "The requested historical basis is unavailable.",
            Self::UnsupportedHistoricalMeaning { .. } => {
                "The requested historical meaning is unsupported for this contract."
            }
            Self::MalformedToken(error) => error.diagnostic_summary(),
            Self::CorruptStorage { .. } => "Persisted historical storage failed validation.",
            Self::MemoryExhausted { .. } => {
                "Historical read working-set or streaming budget is exhausted."
            }
            Self::RequestStopped => "The historical read request was stopped before completion.",
        }
    }
}

impl Reportable for RetainedBasisTokenDecodeError {
    fn denial_class(&self) -> DenialClass {
        match self {
            Self::IntegrityMismatch => DenialClass::Defect,
            Self::InvalidPrefix
            | Self::InvalidEncoding
            | Self::Truncated
            | Self::TrailingBytes
            | Self::UnsupportedVersion { .. }
            | Self::MalformedField => DenialClass::InvalidInput,
        }
    }

    fn disposition(&self) -> Disposition {
        match self {
            Self::IntegrityMismatch => Disposition::Escalate,
            Self::InvalidPrefix
            | Self::InvalidEncoding
            | Self::Truncated
            | Self::TrailingBytes
            | Self::UnsupportedVersion { .. }
            | Self::MalformedField => Disposition::CorrectInput,
        }
    }

    fn report_code(&self) -> ReportCode {
        match self {
            Self::IntegrityMismatch => CODE_TOKEN_INTEGRITY,
            Self::InvalidPrefix
            | Self::InvalidEncoding
            | Self::Truncated
            | Self::TrailingBytes
            | Self::UnsupportedVersion { .. }
            | Self::MalformedField => CODE_MALFORMED_TOKEN,
        }
    }

    fn diagnostic_summary(&self) -> &'static str {
        match self {
            Self::IntegrityMismatch => "Retained basis cursor token failed integrity verification.",
            Self::InvalidPrefix
            | Self::InvalidEncoding
            | Self::Truncated
            | Self::TrailingBytes
            | Self::UnsupportedVersion { .. }
            | Self::MalformedField => "Retained basis cursor token is malformed.",
        }
    }
}

impl From<RetainedBasisTokenDecodeError> for HistoricalReadDenial {
    fn from(error: RetainedBasisTokenDecodeError) -> Self {
        match error {
            RetainedBasisTokenDecodeError::IntegrityMismatch => Self::CorruptStorage {
                subject: RelationalStorageSubject::Region(PartitionId(0)),
            },
            other => Self::MalformedToken(other),
        }
    }
}
