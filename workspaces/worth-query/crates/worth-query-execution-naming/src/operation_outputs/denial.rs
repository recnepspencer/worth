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
    /// A discriminator references the same-commit name of an output this
    /// candidate did not name before the referencing birth.
    ForeignOutputReference { role_path: String },
    /// A same-commit name was requested for a created output this candidate
    /// has not named.
    UnnamedOutputReference {
        subject: ApplicationNameSubjectKind,
        subject_name: String,
    },
    /// A discriminator references a same-commit name of another occurrence.
    CrossOccurrenceReference {
        role_path: String,
        occurrence_key: String,
        referenced_occurrence_key: String,
    },
    /// A `merge_least` listed no parents, so no least parent can keep.
    EmptyMergeParents { subject: ApplicationNameSubjectKind },
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
            Self::ForeignOutputReference { role_path } => write!(
                formatter,
                "role `{role_path}` references a same-commit name this candidate did not issue for an earlier named output"
            ),
            Self::UnnamedOutputReference {
                subject,
                subject_name,
            } => write!(
                formatter,
                "created {} `{subject_name}` has no declared name to reference",
                subject.canonical_name()
            ),
            Self::CrossOccurrenceReference {
                role_path,
                occurrence_key,
                referenced_occurrence_key,
            } => write!(
                formatter,
                "role `{role_path}` of occurrence `{occurrence_key}` references a same-commit name of occurrence `{referenced_occurrence_key}`"
            ),
            Self::EmptyMergeParents { subject } => write!(
                formatter,
                "a least-keeper {} merge lists no parents",
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
