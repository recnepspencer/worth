//! Durable opaque Home-MAC-authenticated [`RetainedBasisToken`] and canonical
//! `whc1_...` wire codec sealed strictly inside `worth-store-application`.
//!
//! Minting and advancing tokens are crate-private through
//! [`super::authority::HistoricalCursorAuthority`] and the Home's private
//! 256-bit HMAC key. External safe Rust cannot forge tokens, forge valid wire
//! seals, or arbitrarily advance cursor positions.

use worth_foundational::facade::{HomeIdentity, HomeIncarnation};
use worth_relational::facade::history::{BranchId, CommitId};
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::schema::{SchemaId, SchemaVersionId};
use worth_runtime_world::facade::{
    BranchOriginIdentity, CanonicalCursorKey, CanonicalKeyDomain, RetainedBasisTokenDecodeError,
    RetainedBasisTokenFields, SchemaVersionIdentity, MAX_BRANCH_ID_BYTES,
    MAX_CANONICAL_CURSOR_KEY_BYTES, MAX_SCHEMA_ID_BYTES, RETAINED_BASIS_TOKEN_PREFIX,
    RETAINED_BASIS_TOKEN_VERSION,
};

use super::wire_codec::{
    ct_eq_32, decode_base64url, encode_base64url, hmac_sha256, ByteReader, MAX_WIRE_BASE64_LEN,
    MAX_WIRE_DECODED_BYTES,
};

const INTEGRITY_DOMAIN: &[u8] = b"worth.historical.cursor.token.hmac.v1";
const AUTH_TAG_BYTES: usize = 32;

/// Durable resumable cursor token (NOT an unbounded retention pin).
///
/// All fields and all minting/advancement methods are crate-private to
/// `worth-store-application` so external callers cannot forge or mutate basis
/// commitments, incarnations, MAC tags, or resume positions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetainedBasisToken {
    format_version: u16,
    home_identity: HomeIdentity,
    home_incarnation: HomeIncarnation,
    branch_origin: BranchOriginIdentity,
    commit_id: CommitId,
    root_semantic_commitment: [u8; 32],
    schema_version: SchemaVersionIdentity,
    query_digest: [u8; 32],
    canonical_key_position: CanonicalCursorKey,
    auth_tag: [u8; 32],
}

impl RetainedBasisToken {
    /// Crate-private minting invoked solely through [`super::authority::HistoricalCursorAuthority`].
    pub(super) fn issue_with_key(
        fields: RetainedBasisTokenFields,
        mac_key: &[u8; 32],
    ) -> Option<Self> {
        if let CanonicalCursorKey::After { logical_key, .. } = &fields.canonical_key_position
            && logical_key.len() > MAX_CANONICAL_CURSOR_KEY_BYTES
        {
            return None;
        }
        let mut token = Self {
            format_version: RETAINED_BASIS_TOKEN_VERSION,
            home_identity: fields.home_identity,
            home_incarnation: fields.home_incarnation,
            branch_origin: fields.branch_origin,
            commit_id: fields.commit_id,
            root_semantic_commitment: fields.root_semantic_commitment,
            schema_version: fields.schema_version,
            query_digest: fields.query_digest,
            canonical_key_position: fields.canonical_key_position,
            auth_tag: [0; 32],
        };
        token.auth_tag = hmac_sha256(mac_key, INTEGRITY_DOMAIN, &token.encode_body());
        Some(token)
    }

    /// Crate-private strictly monotone position advancement invoked solely by
    /// [`super::cursor::HistoricalCursor::next_page`] using its Home's MAC key.
    pub(super) fn advance_with_key(
        &self,
        next_position: CanonicalCursorKey,
        mac_key: &[u8; 32],
    ) -> Option<Self> {
        if next_position <= self.canonical_key_position {
            return None;
        }
        Self::issue_with_key(
            RetainedBasisTokenFields {
                home_identity: self.home_identity,
                home_incarnation: self.home_incarnation,
                branch_origin: self.branch_origin.clone(),
                commit_id: self.commit_id,
                root_semantic_commitment: self.root_semantic_commitment,
                schema_version: self.schema_version.clone(),
                query_digest: self.query_digest,
                canonical_key_position: next_position,
            },
            mac_key,
        )
    }

    /// Verifies the Home-keyed HMAC-SHA256 tag in constant time.
    pub(super) fn verify_mac(&self, mac_key: &[u8; 32]) -> bool {
        let expected = hmac_sha256(mac_key, INTEGRITY_DOMAIN, &self.encode_body());
        ct_eq_32(&self.auth_tag, &expected)
    }

    pub const fn format_version(&self) -> u16 {
        self.format_version
    }

    pub const fn home_identity(&self) -> HomeIdentity {
        self.home_identity
    }

    pub const fn home_incarnation(&self) -> HomeIncarnation {
        self.home_incarnation
    }

    pub const fn branch_origin(&self) -> &BranchOriginIdentity {
        &self.branch_origin
    }

    pub const fn commit_id(&self) -> CommitId {
        self.commit_id
    }

    pub const fn root_semantic_commitment(&self) -> &[u8; 32] {
        &self.root_semantic_commitment
    }

    pub const fn schema_version(&self) -> &SchemaVersionIdentity {
        &self.schema_version
    }

    pub const fn query_digest(&self) -> &[u8; 32] {
        &self.query_digest
    }

    pub const fn canonical_key_position(&self) -> &CanonicalCursorKey {
        &self.canonical_key_position
    }

    /// Encodes this token into its canonical opaque `whc1_...` wire representation.
    pub fn encode_wire(&self) -> String {
        let mut raw = self.encode_body();
        raw.extend_from_slice(&self.auth_tag);
        let mut out =
            String::with_capacity(RETAINED_BASIS_TOKEN_PREFIX.len() + raw.len() * 4 / 3 + 4);
        out.push_str(RETAINED_BASIS_TOKEN_PREFIX);
        encode_base64url(&raw, &mut out);
        out
    }

    /// Decodes an opaque `whc1_...` token string, rejecting oversized inputs
    /// BEFORE allocating. Cryptographic authenticity of the carried Home-keyed
    /// MAC is verified on `resume_cursor` by the owning Home's authority.
    pub fn decode_wire(wire: &str) -> Result<Self, RetainedBasisTokenDecodeError> {
        use RetainedBasisTokenDecodeError as ErrKind;
        let payload = wire
            .strip_prefix(RETAINED_BASIS_TOKEN_PREFIX)
            .ok_or(ErrKind::InvalidPrefix)?;
        if payload.len() > MAX_WIRE_BASE64_LEN {
            return Err(ErrKind::MalformedField);
        }
        let raw = decode_base64url(payload)?;
        if raw.len() > MAX_WIRE_DECODED_BYTES {
            return Err(ErrKind::MalformedField);
        }
        if raw.len() < 2 + AUTH_TAG_BYTES {
            return Err(ErrKind::Truncated);
        }
        let body_len = raw.len() - AUTH_TAG_BYTES;
        let (body, tag_slice) = raw.split_at(body_len);
        let auth_tag: [u8; 32] = tag_slice.try_into().expect("32-byte MAC");
        let mut reader = ByteReader { bytes: body, at: 0 };
        let format_version = reader.u16()?;
        if format_version != RETAINED_BASIS_TOKEN_VERSION {
            return Err(ErrKind::UnsupportedVersion {
                found: format_version,
            });
        }
        let home_identity = HomeIdentity::from_observed_bytes(reader.array_16()?)
            .map_err(|_| ErrKind::MalformedField)?;
        let home_incarnation =
            HomeIncarnation::from_recorded(reader.u64()?).ok_or(ErrKind::MalformedField)?;
        let branch_id_str = reader.utf8(MAX_BRANCH_ID_BYTES)?;
        let branch_origin_digest = reader.array_32()?;
        let branch_origin =
            BranchOriginIdentity::from_owner(BranchId(branch_id_str), branch_origin_digest)
                .ok_or(ErrKind::MalformedField)?;
        let commit_id = CommitId(reader.u64()?);
        let root_semantic_commitment = reader.array_32()?;
        let schema_id_str = reader.utf8(MAX_SCHEMA_ID_BYTES)?;
        let schema_version_id = SchemaVersionId(reader.u32()?);
        let schema_commitment = reader.array_32()?;
        let schema_version = SchemaVersionIdentity::new(
            SchemaId(schema_id_str),
            schema_version_id,
            schema_commitment,
        )
        .ok_or(ErrKind::MalformedField)?;
        let query_digest = reader.array_32()?;
        let canonical_key_position = match reader.u8()? {
            0 => CanonicalCursorKey::Start,
            1 => {
                let partition_id = PartitionId(reader.u32()?);
                let domain =
                    CanonicalKeyDomain::from_tag(reader.u8()?).ok_or(ErrKind::MalformedField)?;
                let key_len = usize::from(reader.u16()?);
                if key_len > MAX_CANONICAL_CURSOR_KEY_BYTES {
                    return Err(ErrKind::MalformedField);
                }
                let logical_key = reader.take(key_len)?.to_vec();
                CanonicalCursorKey::after(partition_id, domain, logical_key)
                    .ok_or(ErrKind::MalformedField)?
            }
            _ => return Err(ErrKind::MalformedField),
        };
        if reader.at != body.len() {
            return Err(ErrKind::TrailingBytes);
        }
        Ok(Self {
            format_version,
            home_identity,
            home_incarnation,
            branch_origin,
            commit_id,
            root_semantic_commitment,
            schema_version,
            query_digest,
            canonical_key_position,
            auth_tag,
        })
    }

    fn encode_body(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(256);
        out.extend_from_slice(&self.format_version.to_le_bytes());
        out.extend_from_slice(&self.home_identity.bytes());
        out.extend_from_slice(&self.home_incarnation.get().to_le_bytes());
        let branch_bytes = self.branch_origin.branch_id().0.as_bytes();
        out.extend_from_slice(&(branch_bytes.len() as u16).to_le_bytes());
        out.extend_from_slice(branch_bytes);
        out.extend_from_slice(&self.branch_origin.origin_digest());
        out.extend_from_slice(&self.commit_id.0.to_le_bytes());
        out.extend_from_slice(&self.root_semantic_commitment);
        let schema_bytes = self.schema_version.schema_id().0.as_bytes();
        out.extend_from_slice(&(schema_bytes.len() as u16).to_le_bytes());
        out.extend_from_slice(schema_bytes);
        out.extend_from_slice(&self.schema_version.schema_version_id().0.to_le_bytes());
        out.extend_from_slice(&self.schema_version.schema_commitment());
        out.extend_from_slice(&self.query_digest);
        match &self.canonical_key_position {
            CanonicalCursorKey::Start => out.push(0),
            CanonicalCursorKey::After {
                partition_id,
                domain,
                logical_key,
            } => {
                out.push(1);
                out.extend_from_slice(&partition_id.to_le_bytes());
                out.push(domain.tag());
                out.extend_from_slice(&(logical_key.len() as u16).to_le_bytes());
                out.extend_from_slice(logical_key);
            }
        }
        out
    }
}
