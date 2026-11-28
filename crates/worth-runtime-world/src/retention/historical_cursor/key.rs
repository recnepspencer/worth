//! Canonical logical cursor key, branch origin, schema version, and admitted
//! query contract identities bound inside [`super::RetainedBasisToken`].
//!
//! [`CanonicalCursorKey`] encodes strictly canonical logical key order — NEVER a
//! physical index offset or tree slot — so discarding auxiliary physical indexes
//! never skips or duplicates rows on resume.

use sha2::{Digest, Sha256};
use worth_relational::facade::branch::RelationalBranchBasisDescriptor;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{KindId, PartitionId};
use worth_relational::facade::schema::{SchemaId, SchemaVersionId};

/// Maximum byte length of a canonical logical key encoded inside a cursor token.
pub const MAX_CANONICAL_CURSOR_KEY_BYTES: usize = 4096;
/// Maximum UTF-8 byte length of a branch id encoded inside a cursor token.
pub const MAX_BRANCH_ID_BYTES: usize = 256;
/// Maximum UTF-8 byte length of a schema id encoded inside a cursor token.
pub const MAX_SCHEMA_ID_BYTES: usize = 256;

/// Admitted canonical key-ascending ordering name for canonical chunk streams.
pub const ORDERING_CANONICAL_ASC: &str = "canonical.key.asc";
/// Admitted full-row projection (payload + spilled/inline history versions).
pub const PROJECTION_CANONICAL_FULL: &str = "canonical.full";
/// Admitted head-only projection (payload only, zero continuation reads).
pub const PROJECTION_CANONICAL_HEAD_ONLY: &str = "canonical.head_only";
/// Admitted keys-only projection (key metadata only, zero payload/continuation reads).
pub const PROJECTION_CANONICAL_KEYS_ONLY: &str = "canonical.keys_only";
/// Admitted Query-Host typed query projection executed over a retained revision.
pub const PROJECTION_QUERY_ROWS: &str = "query.rows.v1";

const BRANCH_ORIGIN_DOMAIN: &[u8] = b"worth.historical.branch-origin.v1";
const QUERY_CONTRACT_DOMAIN: &[u8] = b"worth.historical.query-contract.v1";

/// Admitted projection semantics enforced by historical cursor execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdmittedHistoricalProjection {
    /// Emits full row payload and complete reconstructed history versions.
    Full,
    /// Emits row payload only and omits history versions without reading continuation pages.
    HeadOnly,
    /// Emits canonical logical keys only with empty payload and history.
    KeysOnly,
    /// Executes an admitted typed Query-Host query projection over a retained revision.
    QueryRows,
}

/// Logical record domain within a partition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum CanonicalKeyDomain {
    Entity = 0,
    Relation = 1,
}

impl CanonicalKeyDomain {
    pub const fn tag(self) -> u8 {
        self as u8
    }

    pub const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0 => Some(Self::Entity),
            1 => Some(Self::Relation),
            _ => None,
        }
    }
}

/// Strictly canonical logical key-order resume position.
///
/// Ordered totally by `Start < After { partition_id, domain, logical_key }`.
/// Carries zero physical index offsets, tree slots, or memory addresses.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CanonicalCursorKey {
    /// Before the first canonical row of the query.
    Start,
    /// Strictly after the canonical row identified by `(partition_id, domain, logical_key)`.
    After {
        partition_id: u32,
        domain: CanonicalKeyDomain,
        logical_key: Vec<u8>,
    },
}

impl CanonicalCursorKey {
    pub const fn start() -> Self {
        Self::Start
    }

    /// Constructs a bounded canonical position strictly after `(partition_id, domain, logical_key)`.
    pub fn after(
        partition_id: PartitionId,
        domain: CanonicalKeyDomain,
        logical_key: Vec<u8>,
    ) -> Option<Self> {
        if logical_key.len() > MAX_CANONICAL_CURSOR_KEY_BYTES {
            return None;
        }
        Some(Self::After {
            partition_id: partition_id.0,
            domain,
            logical_key,
        })
    }

    /// Encodes a canonical entity key from `(partition_id, kind_id, logical_key_bytes)`.
    pub fn after_entity_key(
        partition_id: PartitionId,
        kind_id: KindId,
        client_key: &[u8],
    ) -> Option<Self> {
        let mut logical_key = Vec::with_capacity(4 + client_key.len());
        logical_key.extend_from_slice(&kind_id.0.to_be_bytes());
        logical_key.extend_from_slice(client_key);
        Self::after(partition_id, CanonicalKeyDomain::Entity, logical_key)
    }

    pub const fn is_start(&self) -> bool {
        matches!(self, Self::Start)
    }
}

/// Durable branch origin provenance bound inside a [`super::RetainedBasisToken`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BranchOriginIdentity {
    branch_id: BranchId,
    origin_digest: [u8; 32],
}

impl BranchOriginIdentity {
    /// Derives durable branch origin identity from a [`RelationalBranchBasisDescriptor`].
    pub fn from_basis_descriptor(descriptor: &RelationalBranchBasisDescriptor) -> Option<Self> {
        let branch_bytes = descriptor.branch_id().0.as_bytes();
        if branch_bytes.is_empty() || branch_bytes.len() > MAX_BRANCH_ID_BYTES {
            return None;
        }
        let mut hasher = Sha256::new();
        hasher.update(BRANCH_ORIGIN_DOMAIN);
        hasher.update((branch_bytes.len() as u16).to_le_bytes());
        hasher.update(branch_bytes);
        hasher.update(descriptor.truth_version().as_u64().to_le_bytes());
        hasher.update(descriptor.root_identity().to_le_bytes());
        hasher.update(descriptor.visibility_commitment());
        Some(Self {
            branch_id: descriptor.branch_id().clone(),
            origin_digest: hasher.finalize().into(),
        })
    }

    /// Constructs a validated branch origin identity for an owning retention runtime.
    pub fn from_owner(branch_id: BranchId, origin_digest: [u8; 32]) -> Option<Self> {
        let bytes = branch_id.0.as_bytes();
        if bytes.is_empty() || bytes.len() > MAX_BRANCH_ID_BYTES {
            return None;
        }
        Some(Self {
            branch_id,
            origin_digest,
        })
    }

    pub fn branch_id(&self) -> &BranchId {
        &self.branch_id
    }

    pub const fn origin_digest(&self) -> [u8; 32] {
        self.origin_digest
    }
}

/// Durable schema version boundary identity bound inside a [`super::RetainedBasisToken`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchemaVersionIdentity {
    schema_id: SchemaId,
    schema_version_id: SchemaVersionId,
    schema_commitment: [u8; 32],
}

impl SchemaVersionIdentity {
    pub fn new(
        schema_id: SchemaId,
        schema_version_id: SchemaVersionId,
        schema_commitment: [u8; 32],
    ) -> Option<Self> {
        let bytes = schema_id.0.as_bytes();
        if bytes.is_empty() || bytes.len() > MAX_SCHEMA_ID_BYTES {
            return None;
        }
        Some(Self {
            schema_id,
            schema_version_id,
            schema_commitment,
        })
    }

    /// Binds schema version metadata with the schema commitment carried by a
    /// [`RelationalBranchBasisDescriptor`].
    pub fn from_basis_descriptor(
        schema_id: SchemaId,
        schema_version_id: SchemaVersionId,
        descriptor: &RelationalBranchBasisDescriptor,
    ) -> Option<Self> {
        Self::new(schema_id, schema_version_id, descriptor.schema_commitment())
    }

    pub fn schema_id(&self) -> &SchemaId {
        &self.schema_id
    }

    pub const fn schema_version_id(&self) -> SchemaVersionId {
        self.schema_version_id
    }

    pub const fn schema_commitment(&self) -> [u8; 32] {
        self.schema_commitment
    }
}

/// Canonical query contract hashed into [`super::RetainedBasisToken::query_digest`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HistoricalQueryContract {
    schema_version: SchemaVersionIdentity,
    domain: CanonicalKeyDomain,
    kind_filter: Option<KindId>,
    projection_name: String,
    ordering_name: String,
}

impl HistoricalQueryContract {
    pub fn new(
        schema_version: SchemaVersionIdentity,
        domain: CanonicalKeyDomain,
        kind_filter: Option<KindId>,
        projection_name: impl Into<String>,
        ordering_name: impl Into<String>,
    ) -> Self {
        Self {
            schema_version,
            domain,
            kind_filter,
            projection_name: projection_name.into(),
            ordering_name: ordering_name.into(),
        }
    }

    pub fn schema_version(&self) -> &SchemaVersionIdentity {
        &self.schema_version
    }

    pub const fn domain(&self) -> CanonicalKeyDomain {
        self.domain
    }

    pub const fn kind_filter(&self) -> Option<KindId> {
        self.kind_filter
    }

    pub fn projection_name(&self) -> &str {
        &self.projection_name
    }

    pub fn ordering_name(&self) -> &str {
        &self.ordering_name
    }

    /// Resolves the admitted projection semantics, or `None` for any unsupported
    /// projection name (which MUST be refused rather than silently ignored).
    pub fn admitted_projection(&self) -> Option<AdmittedHistoricalProjection> {
        match self.projection_name.as_str() {
            PROJECTION_CANONICAL_FULL => Some(AdmittedHistoricalProjection::Full),
            PROJECTION_CANONICAL_HEAD_ONLY => Some(AdmittedHistoricalProjection::HeadOnly),
            PROJECTION_CANONICAL_KEYS_ONLY => Some(AdmittedHistoricalProjection::KeysOnly),
            PROJECTION_QUERY_ROWS => Some(AdmittedHistoricalProjection::QueryRows),
            _ => None,
        }
    }

    /// Whether this query contract is admitted for canonical key-order chunk streaming.
    pub fn admitted_canonical_projection(&self) -> Option<AdmittedHistoricalProjection> {
        if self.ordering_name != ORDERING_CANONICAL_ASC {
            return None;
        }
        match self.admitted_projection()? {
            p @ (AdmittedHistoricalProjection::Full
            | AdmittedHistoricalProjection::HeadOnly
            | AdmittedHistoricalProjection::KeysOnly) => Some(p),
            AdmittedHistoricalProjection::QueryRows => None,
        }
    }

    /// Canonical SHA-256 digest of the query projection, filter, schema, and ordering contract.
    pub fn query_digest(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(QUERY_CONTRACT_DOMAIN);
        let schema_bytes = self.schema_version.schema_id().0.as_bytes();
        hasher.update((schema_bytes.len() as u32).to_le_bytes());
        hasher.update(schema_bytes);
        hasher.update(self.schema_version.schema_version_id().0.to_le_bytes());
        hasher.update(self.schema_version.schema_commitment());
        hasher.update([self.domain.tag()]);
        match self.kind_filter {
            Some(kind) => {
                hasher.update([1]);
                hasher.update(kind.0.to_le_bytes());
            }
            None => hasher.update([0]),
        }
        let proj = self.projection_name.as_bytes();
        hasher.update((proj.len() as u32).to_le_bytes());
        hasher.update(proj);
        let ord = self.ordering_name.as_bytes();
        hasher.update((ord.len() as u32).to_le_bytes());
        hasher.update(ord);
        hasher.finalize().into()
    }
}
