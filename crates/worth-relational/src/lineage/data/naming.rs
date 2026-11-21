use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::identity::data::{EntityId, LineageId, RelationId};

/// Fixed 32-byte canonical persistent name identity minted exclusively by
/// Relational's naming authority.
///
/// Never carries allocation, partition, slot, generation, or commit-position
/// coordinates. Callers outside `worth-relational` have no raw-bytes
/// constructor.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PersistentNameIdentity([u8; Self::BYTE_LEN]);

impl PersistentNameIdentity {
    /// Fixed byte size of every persistent name identity at any lineage depth.
    pub const BYTE_LEN: usize = 32;

    pub(crate) const fn from_minted_digest(bytes: [u8; Self::BYTE_LEN]) -> Self {
        Self(bytes)
    }

    #[inline]
    pub const fn as_bytes(&self) -> &[u8; Self::BYTE_LEN] {
        &self.0
    }

    pub fn render_hex(&self) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut out = String::with_capacity(Self::BYTE_LEN * 2);
        for byte in self.0 {
            out.push(HEX[(byte >> 4) as usize] as char);
            out.push(HEX[(byte & 0x0f) as usize] as char);
        }
        out
    }
}

impl fmt::Debug for PersistentNameIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("PersistentNameIdentity")
            .field(&self.render_hex())
            .finish()
    }
}

/// Subject class governed by a persistent name scheme in Relational.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PersistentNameSubjectKind {
    Entity,
    Relation,
    Group,
}

/// Canonical birth provenance recorded on a `Create` lineage event.
///
/// Parent names enter strictly as fixed 32-byte [`PersistentNameIdentity`]
/// digests in sorted order so birth-record size stays O(1) at arbitrary split
/// depth.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistentNameBirthParts {
    scheme_id: String,
    occurrence_key: String,
    role_path: String,
    parent_identities: Vec<PersistentNameIdentity>,
    canonical_discriminator: Vec<u8>,
}

impl PersistentNameBirthParts {
    pub(crate) fn new(
        scheme_id: String,
        occurrence_key: String,
        role_path: String,
        parent_identities: Vec<PersistentNameIdentity>,
        canonical_discriminator: Vec<u8>,
    ) -> Self {
        Self {
            scheme_id,
            occurrence_key,
            role_path,
            parent_identities,
            canonical_discriminator,
        }
    }

    pub fn scheme_id(&self) -> &str {
        &self.scheme_id
    }

    pub fn occurrence_key(&self) -> &str {
        &self.occurrence_key
    }

    pub fn role_path(&self) -> &str {
        &self.role_path
    }

    pub fn parent_identities(&self) -> &[PersistentNameIdentity] {
        &self.parent_identities
    }

    pub fn canonical_discriminator(&self) -> &[u8] {
        &self.canonical_discriminator
    }

    /// Logical byte footprint of the canonical birth parts.
    pub fn logical_byte_len(&self) -> usize {
        self.scheme_id.len()
            + self.occurrence_key.len()
            + self.role_path.len()
            + self.parent_identities.len() * PersistentNameIdentity::BYTE_LEN
            + self.canonical_discriminator.len()
    }
}

/// Authoritative birth record carried on a canonical `Create` lineage event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistentNameBirthRecord {
    identity: PersistentNameIdentity,
    subject_kind: PersistentNameSubjectKind,
    parts: PersistentNameBirthParts,
}

impl PersistentNameBirthRecord {
    pub(crate) fn new(
        identity: PersistentNameIdentity,
        subject_kind: PersistentNameSubjectKind,
        parts: PersistentNameBirthParts,
    ) -> Self {
        Self {
            identity,
            subject_kind,
            parts,
        }
    }

    pub const fn identity(&self) -> PersistentNameIdentity {
        self.identity
    }

    pub const fn subject_kind(&self) -> PersistentNameSubjectKind {
        self.subject_kind
    }

    pub fn parts(&self) -> &PersistentNameBirthParts {
        &self.parts
    }
}

/// Identifies a live or pending holder involved in a persistent-name collision.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PersistentNameHolder {
    Entity(EntityId),
    Relation(RelationId),
    Lineage(LineageId),
    PendingBirth {
        occurrence_key: String,
        role_path: String,
        ordinal: usize,
    },
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
    /// A discriminator referenced a same-occurrence role that is not strictly
    /// earlier in the scheme's acyclic role order.
    InvalidDiscriminatorBasis {
        scheme_id: String,
        role_path: String,
        referenced_role: String,
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
                referenced_role,
            } => write!(
                formatter,
                "invalid discriminator basis in scheme `{scheme_id}`: role `{role_path}` cannot reference `{referenced_role}`"
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
        }
    }
}

impl std::error::Error for LineageNameDenial {}

/// Immutable installed name scheme contract lowered into Relational for commit
/// finalization minting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledPersistentNameScheme {
    scheme_id: String,
    subject_kind: PersistentNameSubjectKind,
    role_ranks: BTreeMap<String, u32>,
    split_capable: bool,
    require_authored_occurrence: bool,
}

impl InstalledPersistentNameScheme {
    /// Constructs a Relational scheme contract from topologically ordered roles
    /// lowered by installation.
    pub fn from_ordered_roles(
        scheme_id: impl Into<String>,
        subject_kind: PersistentNameSubjectKind,
        ordered_roles: impl IntoIterator<Item = impl Into<String>>,
        split_capable: bool,
        require_authored_occurrence: bool,
    ) -> Self {
        let mut role_ranks = BTreeMap::new();
        for (rank, role) in ordered_roles.into_iter().enumerate() {
            role_ranks.insert(
                role.into(),
                u32::try_from(rank).expect("role count fits u32"),
            );
        }
        Self {
            scheme_id: scheme_id.into(),
            subject_kind,
            role_ranks,
            split_capable,
            require_authored_occurrence,
        }
    }

    pub fn scheme_id(&self) -> &str {
        &self.scheme_id
    }

    pub const fn subject_kind(&self) -> PersistentNameSubjectKind {
        self.subject_kind
    }

    pub fn role_rank(&self, role: &str) -> Option<u32> {
        self.role_ranks.get(role).copied()
    }

    pub const fn split_capable(&self) -> bool {
        self.split_capable
    }

    pub const fn require_authored_occurrence(&self) -> bool {
        self.require_authored_occurrence
    }
}

/// Immutable registry of installed persistent name schemes visible during
/// Relational commit finalization.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InstalledPersistentNameSchemeRegistry {
    schemes: BTreeMap<String, InstalledPersistentNameScheme>,
}

impl InstalledPersistentNameSchemeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, scheme: InstalledPersistentNameScheme) -> Result<(), String> {
        let id = scheme.scheme_id.clone();
        if self.schemes.contains_key(&id) {
            return Err(format!("duplicate installed persistent name scheme `{id}`"));
        }
        self.schemes.insert(id, scheme);
        Ok(())
    }

    pub fn get(&self, scheme_id: &str) -> Option<&InstalledPersistentNameScheme> {
        self.schemes.get(scheme_id)
    }

    pub fn is_empty(&self) -> bool {
        self.schemes.is_empty()
    }
}

/// Input supplied at commit finalization for one named birth.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistentNameBirthRequest {
    pub scheme_id: String,
    pub occurrence_key: String,
    pub role_path: String,
    pub parent_identities: Vec<PersistentNameIdentity>,
    pub referenced_same_occurrence_roles: Vec<String>,
    pub canonical_discriminator: Vec<u8>,
    pub holder: PersistentNameHolder,
}
