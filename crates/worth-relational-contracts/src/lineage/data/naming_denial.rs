use std::fmt;

use super::events::LineageSubject;
use super::naming::{PersistentNameHolder, PersistentNameIdentity};
use super::postures::LineageRecordTarget;
use crate::aggregate_identity::transition::group::denial::AggregateGroupNamingDenial;

/// Posture kind named in a [`LineageNameDenial::LineagePostureMismatch`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LineagePostureKind {
    Create,
    Preserve,
    Retire,
    Restore,
    Split,
    Merge,
}

/// Specific reason a discriminator's referenced names failed basis validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscriminatorBasisDenialReason {
    /// Referenced basis name was never recorded on the branch at the basis.
    AbsentBasisName { identity: PersistentNameIdentity },
    /// Referenced same-occurrence role is not strictly earlier in scheme order.
    NonPrecedingRole { referenced_role: String },
    /// Referenced same-occurrence name was not minted earlier in this commit
    /// for the same occurrence and role.
    UnbornSameOccurrenceName {
        referenced_role: String,
        identity: PersistentNameIdentity,
    },
    /// Canonical discriminator bytes contain an encoded `TAG_NAME_REF` digest
    /// that is not present in the verified `referenced_names` table.
    UndeclaredEncodedNameReference { raw_digest: [u8; 32] },
    /// Non-empty discriminator bytes failed canonical v1 domain-header or
    /// structural encoding validation.
    MalformedDiscriminator(worth_foundational::DiscriminatorScanError),
    /// A structural-identity discriminator exceeds its wire ceiling.
    UnboundedStructuralDiscriminator { len: usize },
}

/// Typed commit-boundary refusal returned when birth minting violates naming
/// laws or collides with an existing live or same-commit holder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineageNameDenial {
    /// Two live or same-commit births minted the exact same identity.
    DuplicateBirth {
        identity: PersistentNameIdentity,
        first: PersistentNameHolder,
        second: PersistentNameHolder,
    },
    /// A discriminator referenced an absent, retired, unborn, undeclared, or
    /// non-preceding persistent name.
    InvalidDiscriminatorBasis {
        scheme_id: String,
        role_path: String,
        reason: DiscriminatorBasisDenialReason,
    },
    /// Referenced parent identity does not exist at the commit's basis,
    /// retired set, or earlier in this commit batch.
    UnknownParentIdentity {
        scheme_id: String,
        role_path: String,
        parent: PersistentNameIdentity,
    },
    /// A birth request targeted a holder/entity not created in this commit.
    BirthWithoutCreate {
        scheme_id: String,
        occurrence_key: String,
        role_path: String,
        holder: PersistentNameHolder,
    },
    /// Referenced scheme is not installed in Relational's naming registry.
    UnknownScheme { scheme_id: String },
    /// Referenced role path is not declared in the installed scheme.
    UndeclaredRole {
        scheme_id: String,
        role_path: String,
    },
    /// Scheme requires an authored occurrence key, but none was supplied.
    MissingAuthoredOccurrenceKey { scheme_id: String },
    /// Split-capable birth with parent identities supplied an empty discriminator.
    MissingSplitDiscriminator {
        scheme_id: String,
        role_path: String,
    },
    /// Parent identity list contained duplicate identities.
    DuplicateParentIdentity {
        scheme_id: String,
        role_path: String,
        parent: PersistentNameIdentity,
    },
    /// Declared output posture contradicted the actual performed commit
    /// effects (e.g., `Split` on an unretired parent or with zero targets,
    /// `Merge` with an unretired non-keeper, `Preserve` on a fresh create, or
    /// `Retire` on an unretired record).
    LineagePostureMismatch {
        posture: LineagePostureKind,
        subject: LineageSubject,
        holder: PersistentNameHolder,
        detail: String,
    },
    /// `Merge` posture named a `KeepParent` identity that is not a member of
    /// its declared `parents` list.
    InvalidMergeKeeper {
        keeper: PersistentNameIdentity,
        parents: Vec<PersistentNameIdentity>,
    },
    /// Commit created a record whose kind has an installed persistent-name
    /// scheme in Relational's schema registry, but no birth or posture covered
    /// it.
    MissingBirth {
        target: LineageRecordTarget,
        scheme_id: String,
    },
    /// A declared or derived group transition of `scheme_id` was refused.
    GroupNaming {
        scheme_id: String,
        denial: AggregateGroupNamingDenial,
    },
}

impl fmt::Display for LineageNameDenial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateBirth {
                identity,
                first,
                second,
            } => write!(
                formatter,
                "duplicate persistent name birth `{}` between {:?} and {:?}",
                identity.render_hex(),
                first,
                second
            ),
            Self::InvalidDiscriminatorBasis {
                scheme_id,
                role_path,
                reason,
            } => write!(
                formatter,
                "invalid discriminator basis for role `{role_path}` in scheme `{scheme_id}`: {reason:?}"
            ),
            Self::UnknownParentIdentity {
                scheme_id,
                role_path,
                parent,
            } => write!(
                formatter,
                "unknown parent identity `{}` for role `{role_path}` in scheme `{scheme_id}`",
                parent.render_hex()
            ),
            Self::BirthWithoutCreate {
                scheme_id,
                occurrence_key,
                role_path,
                holder,
            } => write!(
                formatter,
                "birth request for `{scheme_id}`/{occurrence_key}/{role_path} ({holder:?}) has no matching Create event"
            ),
            Self::UnknownScheme { scheme_id } => {
                write!(formatter, "unknown installed name scheme `{scheme_id}`")
            }
            Self::UndeclaredRole {
                scheme_id,
                role_path,
            } => write!(
                formatter,
                "undeclared role `{role_path}` in name scheme `{scheme_id}`"
            ),
            Self::MissingAuthoredOccurrenceKey { scheme_id } => write!(
                formatter,
                "missing authored occurrence key for name scheme `{scheme_id}`"
            ),
            Self::MissingSplitDiscriminator {
                scheme_id,
                role_path,
            } => write!(
                formatter,
                "missing split discriminator for role `{role_path}` in split-capable scheme `{scheme_id}`"
            ),
            Self::DuplicateParentIdentity {
                scheme_id,
                role_path,
                parent,
            } => write!(
                formatter,
                "duplicate parent identity `{}` for role `{role_path}` in scheme `{scheme_id}`",
                parent.render_hex()
            ),
            Self::LineagePostureMismatch {
                posture,
                subject,
                holder,
                detail,
            } => write!(
                formatter,
                "lineage posture mismatch for {subject:?} {posture:?} on {holder:?}: {detail}"
            ),
            Self::InvalidMergeKeeper { keeper, parents } => write!(
                formatter,
                "invalid merge keeper `{}` is not among declared parents ({} parents)",
                keeper.render_hex(),
                parents.len()
            ),
            Self::MissingBirth { target, scheme_id } => write!(
                formatter,
                "created record {target:?} requires a persistent-name birth under installed scheme `{scheme_id}`"
            ),
            Self::GroupNaming { scheme_id, denial } => {
                write!(formatter, "group scheme `{scheme_id}` refused: {denial:?}")
            }
        }
    }
}

impl std::error::Error for LineageNameDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::GroupNaming { denial, .. } => Some(denial),
            _ => None,
        }
    }
}
