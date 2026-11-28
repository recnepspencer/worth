//! Typed refusals of tracked persistent-name reads.

use std::fmt;

use worth_query_declaration::facade::application_naming::{DiscriminatorScanError, NameResolutionDenial, NoNameOwner};
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
    /// The record is not one the batch read observed.
    RecordNotObserved { record: LineageRecordTarget },
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
    /// The held name has no birth recorded and visible at the reader's
    /// basis: it is born later, on another branch, or never.
    UnknownName,
    /// A recorded birth carries malformed discriminator bytes.
    MalformedDiscriminator(DiscriminatorScanError),
    /// A recorded birth's parent is not recorded at the reader's basis.
    UnrecordedBirthReference { digest: [u8; 32] },
    /// A birth looked up by its parts references a name only its own
    /// commit mints; only committed references can match a recorded birth.
    SameCommitBirthReference,
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
            Self::RecordNotObserved { record } => {
                write!(formatter, "record {record:?} was not observed by this name read")
            }
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
            Self::UnknownName => {
                formatter.write_str("no birth of this name is recorded at this basis")
            }
            Self::MalformedDiscriminator(error) => error.fmt(formatter),
            Self::UnrecordedBirthReference { .. } => {
                formatter.write_str("a recorded birth's parent is not recorded at this basis")
            }
            Self::SameCommitBirthReference => {
                formatter.write_str("a birth lookup cannot reference a same-commit name")
            }
            Self::EmptySelectionFilter => formatter.write_str("selection filter is empty"),
        }
    }
}

impl std::error::Error for ExecutionNameReadDenial {}

/// Why a tracked `resolve` of a held `WorthQueryName<K>` refused: one of
/// resolution's own typed outcomes, or a read refusal. `O` is the owner
/// handle its candidates carry.
pub enum ExecutionNameResolveDenial<K, O = NoNameOwner> {
    Resolution(NameResolutionDenial<K, O>),
    Read(ExecutionNameReadDenial),
}

impl<K, O: Clone> Clone for ExecutionNameResolveDenial<K, O> {
    fn clone(&self) -> Self {
        match self {
            Self::Resolution(denial) => Self::Resolution(denial.clone()),
            Self::Read(denial) => Self::Read(denial.clone()),
        }
    }
}

impl<K, O: PartialEq> PartialEq for ExecutionNameResolveDenial<K, O> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Resolution(a), Self::Resolution(b)) => a == b,
            (Self::Read(a), Self::Read(b)) => a == b,
            _ => false,
        }
    }
}

impl<K, O: Eq> Eq for ExecutionNameResolveDenial<K, O> {}

impl<K, O: fmt::Debug> fmt::Debug for ExecutionNameResolveDenial<K, O> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Resolution(denial) => formatter.debug_tuple("Resolution").field(denial).finish(),
            Self::Read(denial) => formatter.debug_tuple("Read").field(denial).finish(),
        }
    }
}

impl<K, O> fmt::Display for ExecutionNameResolveDenial<K, O> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Resolution(denial) => fmt::Display::fmt(denial, formatter),
            Self::Read(denial) => fmt::Display::fmt(denial, formatter),
        }
    }
}

impl<K, O: fmt::Debug> std::error::Error for ExecutionNameResolveDenial<K, O> {}

impl<K, O> From<ExecutionNameReadDenial> for ExecutionNameResolveDenial<K, O> {
    fn from(denial: ExecutionNameReadDenial) -> Self {
        Self::Read(denial)
    }
}

impl<K, O> From<NameResolutionDenial<K, O>> for ExecutionNameResolveDenial<K, O> {
    fn from(denial: NameResolutionDenial<K, O>) -> Self {
        Self::Resolution(denial)
    }
}
