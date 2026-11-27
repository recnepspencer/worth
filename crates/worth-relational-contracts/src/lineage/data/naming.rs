use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::identity::data::{EntityId, LineageId, RelationId};

/// Fixed 32-byte canonical persistent name identity minted exclusively by
/// Relational's naming authority.
///
/// Never carries allocation, partition, slot, generation, or commit-position
/// coordinates. Intentionally does NOT implement `Deserialize` so untrusted
/// wire bytes cannot forge an identity outside Relational's verified birth and
/// readmission doors.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct PersistentNameIdentity([u8; Self::BYTE_LEN]);

impl PersistentNameIdentity {
    /// Fixed byte size of every persistent name identity at any lineage depth.
    pub const BYTE_LEN: usize = 32;

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PersistentNameBirthParts {
    scheme_id: String,
    occurrence_key: String,
    role_path: String,
    parent_identities: Vec<PersistentNameIdentity>,
    canonical_discriminator: Vec<u8>,
    structural_identity: bool,
}

impl PersistentNameBirthParts {
    pub const fn structural_identity(&self) -> bool {
        self.structural_identity
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PersistentNameBirthRecord {
    identity: PersistentNameIdentity,
    subject_kind: PersistentNameSubjectKind,
    parts: PersistentNameBirthParts,
}

impl PersistentNameBirthRecord {
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

/// Identifies a live or pending holder involved in a persistent-name refusal.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
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

/// Where a persistent name referenced by a birth's discriminator must exist.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum PersistentNameReferenceBasis {
    /// Recorded on the branch at the basis: live, a retired ancestor, or retired
    /// by this commit (a consumed cutter). Never-recorded names refuse.
    Basis,
    /// Must be minted earlier in the same commit batch under the exact same
    /// `occurrence_key` with a strictly lower role rank in the scheme DAG.
    SameOccurrence { role: String },
}

/// One typed persistent-name reference carried inside a birth's discriminator.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct PersistentNameDiscriminatorReference {
    pub identity: PersistentNameIdentity,
    pub basis: PersistentNameReferenceBasis,
}

/// Immutable installed name scheme contract lowered into Relational for commit
/// finalization minting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledPersistentNameScheme {
    scheme_id: String,
    subject_kind: PersistentNameSubjectKind,
    subject_name: String,
    role_ranks: BTreeMap<String, u32>,
    split_capable: bool,
    require_authored_occurrence: bool,
    structural_identity: bool,
}

impl InstalledPersistentNameScheme {
    /// Validates and constructs a Relational scheme contract from topologically
    /// ordered roles lowered by installation, binding the declared subject name.
    pub fn try_from_ordered_roles_for_subject(
        scheme_id: impl Into<String>,
        subject_kind: PersistentNameSubjectKind,
        subject_name: impl Into<String>,
        ordered_roles: impl IntoIterator<Item = impl Into<String>>,
        split_capable: bool,
        require_authored_occurrence: bool,
        structural_identity: bool,
    ) -> Result<Self, String> {
        let scheme_id = scheme_id.into();
        if !is_valid_dotted_scheme_id(&scheme_id) {
            return Err(format!(
                "invalid persistent name scheme identifier `{scheme_id}`"
            ));
        }
        let subject_name = subject_name.into();
        if subject_name.trim().is_empty() {
            return Err(format!(
                "persistent name scheme `{scheme_id}` requires a non-empty subject name"
            ));
        }
        let mut role_ranks = BTreeMap::new();
        for (rank, role) in ordered_roles.into_iter().enumerate() {
            let role = role.into();
            if role.trim().is_empty() {
                return Err(format!(
                    "persistent name scheme `{scheme_id}` contains an empty role name"
                ));
            }
            let rank = u32::try_from(rank)
                .map_err(|_| format!("persistent name scheme `{scheme_id}` exceeded u32 roles"))?;
            if role_ranks.insert(role.clone(), rank).is_some() {
                return Err(format!(
                    "persistent name scheme `{scheme_id}` contains duplicate role `{role}`"
                ));
            }
        }
        if role_ranks.is_empty() {
            return Err(format!(
                "persistent name scheme `{scheme_id}` must declare at least one role"
            ));
        }
        Ok(Self {
            scheme_id,
            subject_kind,
            subject_name,
            role_ranks,
            split_capable,
            require_authored_occurrence,
            structural_identity,
        })
    }

    /// Mints names from the discriminator alone (`canonical_birth_digest`).
    pub const fn structural_identity(&self) -> bool {
        self.structural_identity
    }

    pub fn scheme_id(&self) -> &str {
        &self.scheme_id
    }

    pub const fn subject_kind(&self) -> PersistentNameSubjectKind {
        self.subject_kind
    }

    pub fn subject_name(&self) -> &str {
        &self.subject_name
    }

    pub fn role_rank(&self, role: &str) -> Option<u32> {
        self.role_ranks.get(role).copied()
    }

    /// Returns declared roles in canonical ascending topological rank order.
    pub fn ordered_roles(&self) -> Vec<String> {
        let mut pairs: Vec<(u32, &str)> = self
            .role_ranks
            .iter()
            .map(|(role, &rank)| (rank, role.as_str()))
            .collect();
        pairs.sort_by_key(|(rank, _)| *rank);
        pairs.into_iter().map(|(_, role)| role.to_owned()).collect()
    }

    pub const fn split_capable(&self) -> bool {
        self.split_capable
    }

    pub const fn require_authored_occurrence(&self) -> bool {
        self.require_authored_occurrence
    }
}

fn is_valid_dotted_scheme_id(id: &str) -> bool {
    !id.is_empty()
        && id.split('.').all(|segment| {
            !segment.is_empty()
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
        })
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

    pub fn len(&self) -> usize {
        self.schemes.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &InstalledPersistentNameScheme> {
        self.schemes.values()
    }
}

/// Input supplied at commit finalization for one named birth.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistentNameBirthRequest {
    pub scheme_id: String,
    pub occurrence_key: String,
    pub role_path: String,
    pub parent_identities: Vec<PersistentNameIdentity>,
    pub referenced_names: Vec<PersistentNameDiscriminatorReference>,
    pub canonical_discriminator: Vec<u8>,
    pub holder: PersistentNameHolder,
}

/// Operations used by the Relational owners.
pub trait PersistentNameIdentityOwnerPort: Sized {
    fn from_minted_digest(bytes: [u8; PersistentNameIdentity::BYTE_LEN]) -> Self;
}

impl PersistentNameIdentityOwnerPort for PersistentNameIdentity {
    fn from_minted_digest(bytes: [u8; Self::BYTE_LEN]) -> Self {
        Self(bytes)
    }
}

/// Operations used by the Relational owners.
pub trait PersistentNameBirthPartsOwnerPort: Sized {
    fn new(
        scheme_id: String,
        occurrence_key: String,
        role_path: String,
        parent_identities: Vec<PersistentNameIdentity>,
        canonical_discriminator: Vec<u8>,
        structural_identity: bool,
    ) -> Self;
}

impl PersistentNameBirthPartsOwnerPort for PersistentNameBirthParts {
    fn new(
        scheme_id: String,
        occurrence_key: String,
        role_path: String,
        parent_identities: Vec<PersistentNameIdentity>,
        canonical_discriminator: Vec<u8>,
        structural_identity: bool,
    ) -> Self {
        Self {
            scheme_id,
            occurrence_key,
            role_path,
            parent_identities,
            canonical_discriminator,
            structural_identity,
        }
    }
}

/// Operations used by the Relational owners.
pub trait PersistentNameBirthRecordOwnerPort: Sized {
    fn new(
        identity: PersistentNameIdentity,
        subject_kind: PersistentNameSubjectKind,
        parts: PersistentNameBirthParts,
    ) -> Self;
}

impl PersistentNameBirthRecordOwnerPort for PersistentNameBirthRecord {
    fn new(
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
}

mod role_order_fixture;
pub use role_order_fixture::InstalledPersistentNameSchemeOwnerPort;
