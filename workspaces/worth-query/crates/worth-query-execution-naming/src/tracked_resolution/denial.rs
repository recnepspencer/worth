//! Typed refusals of tracked persistent-name reads.

use std::fmt;

use worth_query_declaration::facade::application_naming::NameResolutionDenial;
use worth_relational::facade::lineage::{
    LineageRecordTarget, PersistentNameSubjectKind, RelationalPersistentNameReadDenial,
};

/// Why a tracked name read refused, apart from resolution's own outcomes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutionNameReadDenial {
    /// Relational refused the read at the reader's observation.
    Relational(RelationalPersistentNameReadDenial),
    /// No persistent name was bound to the record at the reader's basis.
    RecordNotNamed { record: LineageRecordTarget },
    /// The record's name no longer designates it at the reader's basis: the
    /// record was retired, replaced, split or merged away. Resolve the held
    /// name instead.
    RecordNoLongerNamed { record: LineageRecordTarget },
    /// The name was born under a scheme the runtime has not installed.
    UnknownScheme { scheme_id: Option<String> },
    /// The name belongs to another subject class than the one requested.
    SubjectKindMismatch {
        expected: PersistentNameSubjectKind,
        actual: PersistentNameSubjectKind,
    },
    /// The name's installed scheme governs another declared subject than the
    /// requested kind `K`.
    KindIdentityMismatch {
        scheme_id: String,
        expected_subject: String,
        requested_subject: String,
    },
    /// A carrier or side filter lowered to an empty payload; the typed
    /// [`NameSelection`](worth_query_declaration::facade::application_naming::NameSelection)
    /// constructors never build one.
    EmptySelectionFilter,
}

impl fmt::Display for ExecutionNameReadDenial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Relational(denial) => write!(formatter, "relational name read refused: {denial:?}"),
            Self::RecordNotNamed { record } => {
                write!(formatter, "record {record:?} carries no persistent name at this basis")
            }
            Self::RecordNoLongerNamed { record } => write!(
                formatter,
                "record {record:?} is no longer designated by its persistent name at this basis"
            ),
            Self::UnknownScheme { scheme_id } => {
                write!(formatter, "persistent name scheme {scheme_id:?} is not installed")
            }
            Self::SubjectKindMismatch { expected, actual } => write!(
                formatter,
                "persistent name read expected subject {expected:?}, found {actual:?}"
            ),
            Self::KindIdentityMismatch {
                scheme_id,
                expected_subject,
                requested_subject,
            } => write!(
                formatter,
                "name under scheme `{scheme_id}` belongs to subject `{expected_subject}`, not `{requested_subject}`"
            ),
            Self::EmptySelectionFilter => formatter.write_str("selection filter is empty"),
        }
    }
}

impl std::error::Error for ExecutionNameReadDenial {}

/// Why a tracked `resolve` of a held `WorthQueryName<K>` refused: one of
/// resolution's own typed outcomes, or a read refusal.
pub enum ExecutionNameResolveDenial<K> {
    Resolution(NameResolutionDenial<K>),
    Read(ExecutionNameReadDenial),
}

impl<K> Clone for ExecutionNameResolveDenial<K> {
    fn clone(&self) -> Self {
        match self {
            Self::Resolution(denial) => Self::Resolution(denial.clone()),
            Self::Read(denial) => Self::Read(denial.clone()),
        }
    }
}

impl<K> PartialEq for ExecutionNameResolveDenial<K> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Resolution(a), Self::Resolution(b)) => a == b,
            (Self::Read(a), Self::Read(b)) => a == b,
            _ => false,
        }
    }
}

impl<K> Eq for ExecutionNameResolveDenial<K> {}

impl<K> fmt::Debug for ExecutionNameResolveDenial<K> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Resolution(denial) => formatter.debug_tuple("Resolution").field(denial).finish(),
            Self::Read(denial) => formatter.debug_tuple("Read").field(denial).finish(),
        }
    }
}

impl<K> fmt::Display for ExecutionNameResolveDenial<K> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Resolution(denial) => fmt::Display::fmt(denial, formatter),
            Self::Read(denial) => fmt::Display::fmt(denial, formatter),
        }
    }
}

impl<K> std::error::Error for ExecutionNameResolveDenial<K> {}

impl<K> From<ExecutionNameReadDenial> for ExecutionNameResolveDenial<K> {
    fn from(denial: ExecutionNameReadDenial) -> Self {
        Self::Read(denial)
    }
}
