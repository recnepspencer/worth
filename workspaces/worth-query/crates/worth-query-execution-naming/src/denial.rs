//! Typed refusal vocabulary for execution-time persistent naming lowering,
//! birth-plan assembly, admission, and Relational commit finalization.

use std::fmt;

use worth_query_declaration::facade::application_naming::ApplicationNameSubjectKind;
use worth_relational::facade::lineage::{
    LineageNameDenial, PersistentNameHolder, PersistentNameIdentity, PersistentNameSubjectKind,
};
use worth_relational::facade::transactions::TransactionCommitError;

/// Typed refusal produced by execution-time persistent naming.
#[derive(Debug)]
pub enum ExecutionNamingDenial {
    /// A lowered scheme identifier was duplicated or invalid during lowering.
    SchemeLowering { scheme_id: String, detail: String },
    /// The birth plan referenced a scheme identifier not present in the
    /// installed catalog.
    UnknownScheme { scheme_id: String },
    /// The birth subject kind does not match the installed scheme's subject kind.
    SubjectKindMismatch {
        scheme_id: String,
        expected: ApplicationNameSubjectKind,
        actual: ApplicationNameSubjectKind,
    },
    /// The declared role path is not present in the installed scheme's DAG.
    UndeclaredRole {
        scheme_id: String,
        role_path: String,
    },
    /// The scheme requires an authored occurrence key, but the supplied key was
    /// empty or whitespace-only.
    MissingAuthoredOccurrenceKey { scheme_id: String },
    /// A split-capable child birth omitted its required canonical discriminator.
    MissingSplitDiscriminator {
        scheme_id: String,
        role_path: String,
    },
    /// A same-occurrence discriminator reference targets a role that is not
    /// strictly earlier in the scheme's installed acyclic role order.
    NonPrecedingSameOccurrenceRole {
        scheme_id: String,
        role_path: String,
        referenced_role: String,
    },
    /// A birth referenced a raw name digest that was neither declared in the
    /// same occurrence nor supplied as an admitted basis reference.
    UnresolvedReferencedName {
        scheme_id: String,
        role_path: String,
        digest: [u8; 32],
    },
    /// The requested record or subject had no committed birth record in the
    /// sealed commit envelope.
    UnadmittedBirth { detail: String },
    /// Attempted to elevate a minted record into the wrong subject kind.
    AdmissionSubjectKindMismatch {
        expected: PersistentNameSubjectKind,
        actual: PersistentNameSubjectKind,
    },
    /// Attempted to admit a committed birth under a declared Rust entity or
    /// relation marker whose kind identifier does not match the birth scheme's
    /// installed subject name.
    KindIdentityMismatch {
        scheme_id: String,
        expected_subject: String,
        requested_subject: String,
    },
    /// Relational commit finalization refused the naming/lineage plan (for
    /// example `DuplicateBirth` or `InvalidDiscriminatorBasis`) with zero
    /// committed effects.
    RelationalNaming(LineageNameDenial),
    /// Relational rejected the enclosing commit before publication.
    RelationalCommit(Box<TransactionCommitError>),
}

impl ExecutionNamingDenial {
    /// Returns the underlying `LineageNameDenial` when Relational refused
    /// persistent name minting or posture validation.
    pub fn lineage_name_denial(&self) -> Option<&LineageNameDenial> {
        match self {
            Self::RelationalNaming(denial) => Some(denial),
            Self::RelationalCommit(err) => err.lineage_name_denial_ref(),
            _ => None,
        }
    }

    /// Returns the colliding `(identity, first, second)` when commit
    /// finalization refused a `DuplicateBirth`.
    pub fn duplicate_birth(
        &self,
    ) -> Option<(
        PersistentNameIdentity,
        &PersistentNameHolder,
        &PersistentNameHolder,
    )> {
        match self.lineage_name_denial()? {
            LineageNameDenial::DuplicateBirth {
                identity,
                first,
                second,
            } => Some((*identity, first, second)),
            _ => None,
        }
    }
}

impl fmt::Display for ExecutionNamingDenial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SchemeLowering { scheme_id, detail } => {
                write!(formatter, "failed lowering name scheme `{scheme_id}`: {detail}")
            }
            Self::UnknownScheme { scheme_id } => {
                write!(formatter, "unknown installed persistent name scheme `{scheme_id}`")
            }
            Self::SubjectKindMismatch {
                scheme_id,
                expected,
                actual,
            } => write!(
                formatter,
                "name scheme `{scheme_id}` governs {} but birth targeted {}",
                expected.canonical_name(),
                actual.canonical_name()
            ),
            Self::UndeclaredRole {
                scheme_id,
                role_path,
            } => write!(
                formatter,
                "role `{role_path}` is not declared in name scheme `{scheme_id}`"
            ),
            Self::MissingAuthoredOccurrenceKey { scheme_id } => write!(
                formatter,
                "name scheme `{scheme_id}` requires a non-empty authored occurrence key"
            ),
            Self::MissingSplitDiscriminator {
                scheme_id,
                role_path,
            } => write!(
                formatter,
                "split child under scheme `{scheme_id}` role `{role_path}` requires a canonical discriminator"
            ),
            Self::NonPrecedingSameOccurrenceRole {
                scheme_id,
                role_path,
                referenced_role,
            } => write!(
                formatter,
                "same-occurrence reference from `{role_path}` to `{referenced_role}` in scheme `{scheme_id}` violates installed acyclic role order"
            ),
            Self::UnresolvedReferencedName {
                scheme_id,
                role_path,
                ..
            } => write!(
                formatter,
                "birth in scheme `{scheme_id}` role `{role_path}` embedded an unresolved persistent name digest"
            ),
            Self::UnadmittedBirth { detail } => {
                write!(formatter, "unadmitted persistent name birth: {detail}")
            }
            Self::AdmissionSubjectKindMismatch { expected, actual } => write!(
                formatter,
                "persistent name admission expected subject {expected:?}, found {actual:?}"
            ),
            Self::KindIdentityMismatch {
                scheme_id,
                expected_subject,
                requested_subject,
            } => write!(
                formatter,
                "birth under scheme `{scheme_id}` belongs to subject `{expected_subject}`, not `{requested_subject}`"
            ),
            Self::RelationalNaming(denial) => write!(formatter, "relational naming refusal: {denial}"),
            Self::RelationalCommit(err) => write!(formatter, "relational commit failed: {err:?}"),
        }
    }
}

impl std::error::Error for ExecutionNamingDenial {}
