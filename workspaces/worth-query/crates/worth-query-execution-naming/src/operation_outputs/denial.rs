//! Typed refusals of an installed operation's declared output names before
//! Relational sees them. Every refusal precedes the commit: zero effects.

use std::fmt;

use worth_query_declaration::facade::application_naming::ApplicationNameSubjectKind;

use crate::denial::ExecutionNamingDenial;

/// Why an operation's declared persistent-name birth or posture was refused
/// before its commit. Relational's own commit-time refusals (`MissingBirth`,
/// `DuplicateBirth`, `LineagePostureMismatch`, `InvalidMergeKeeper`, ...)
/// arrive as `LineageNameDenial` on the commit denial instead.
#[derive(Debug)]
pub enum WorthQueryOutputNameDenial {
    /// The named record's kind has no installed persistent-name scheme.
    UnschemedOutput {
        subject: ApplicationNameSubjectKind,
        subject_name: String,
    },
    /// The birth target is not a record this program created.
    NotCreatedOutput {
        subject: ApplicationNameSubjectKind,
        subject_name: String,
    },
    /// The created output named by a birth was not lowered into the commit.
    UnboundOutput {
        subject: ApplicationNameSubjectKind,
        key: String,
    },
    /// A held name has no birth visible at the commit basis.
    UnknownHeldName {
        subject: ApplicationNameSubjectKind,
        digest: [u8; 32],
    },
    /// A held name no longer designates one live record at the commit basis,
    /// so no record can be the subject of its split, merge or retirement.
    HeldNameNotLive {
        subject: ApplicationNameSubjectKind,
        digest: [u8; 32],
    },
    /// The commit basis's name index could not be read.
    NameIndexUnavailable { detail: String },
    /// Scheme, role, occurrence or discriminator validation refused the birth.
    Assembly(ExecutionNamingDenial),
}

impl fmt::Display for WorthQueryOutputNameDenial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnschemedOutput {
                subject,
                subject_name,
            } => write!(
                formatter,
                "{} `{subject_name}` has no installed persistent-name scheme",
                subject.canonical_name()
            ),
            Self::NotCreatedOutput {
                subject,
                subject_name,
            } => write!(
                formatter,
                "named {} `{subject_name}` is not an output this operation created",
                subject.canonical_name()
            ),
            Self::UnboundOutput { subject, key } => write!(
                formatter,
                "named {} output `{key}` is absent from the lowered commit",
                subject.canonical_name()
            ),
            Self::UnknownHeldName { subject, .. } => write!(
                formatter,
                "held {} name has no birth at the commit basis",
                subject.canonical_name()
            ),
            Self::HeldNameNotLive { subject, .. } => write!(
                formatter,
                "held {} name designates no single live record at the commit basis",
                subject.canonical_name()
            ),
            Self::NameIndexUnavailable { detail } => {
                write!(formatter, "persistent-name index unavailable: {detail}")
            }
            Self::Assembly(denial) => write!(formatter, "persistent-name output refused: {denial}"),
        }
    }
}

impl std::error::Error for WorthQueryOutputNameDenial {}
