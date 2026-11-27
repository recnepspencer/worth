//! The naming tables one branch's page run interleaves, by leading tag.
//!
//! A row key is the table tag followed by the table key's canonical bytes,
//! so each table is one contiguous key range and a scheme-scoped group table
//! is one contiguous prefix range.
use super::codec::{NamingCodec, Writer};

/// Naming basis: lineage to holder.
pub const LINEAGE_HOLDER: u8 = 1;
/// Naming basis: holder to the name it last held.
pub const HOLDER_NAME: u8 = 2;
/// Naming basis: live name to holder.
pub const LIVE_HOLDER: u8 = 3;
/// Naming basis: live holder to lineage.
pub const LIVE_LINEAGE: u8 = 4;
/// Naming basis: retired name to its retired lineage and commit.
pub const RETIRED_LINEAGE: u8 = 5;
/// Name index: per-name history.
pub const INDEX_NAME: u8 = 6;
/// Name index: lineage to the name it was created for.
pub const INDEX_LINEAGE_NAME: u8 = 7;
/// Name index: lineage to its record.
pub const INDEX_LINEAGE_HOLDER: u8 = 8;
/// Name index: record to its binding epochs.
pub const INDEX_HOLDER_LINEAGES: u8 = 9;
/// Name index: member name to its group tenures.
pub const INDEX_MEMBER_GROUPS: u8 = 10;
/// Group state: `(scheme, group)` to the live group.
pub const GROUP_LIVE: u8 = 11;
/// Group state: `(scheme, group)` of a retired group name.
pub const GROUP_RETIRED: u8 = 12;
/// Group state: `(scheme, member)` to its live owner group.
pub const GROUP_MEMBER_OWNER: u8 = 13;
/// Published group lineage: `((ordinal, commit), branch)` to the canonical frame.
pub const GROUP_LOG: u8 = 14;
/// Value continuation chunk: `(base_key_len_be, base_key, chunk_idx_be)` -> bytes.
pub const VALUE_CHUNK: u8 = 15;

/// Logical component domain of a naming table tag.
///
/// Pages never mix rows from different domains so read-free manifest
/// validation can verify each component's page presence from descriptors
/// alone.
pub const fn table_domain(tag: u8) -> Option<u8> {
    match tag {
        LINEAGE_HOLDER..=RETIRED_LINEAGE => Some(1),
        INDEX_NAME..=INDEX_MEMBER_GROUPS => Some(2),
        GROUP_LIVE..=GROUP_MEMBER_OWNER => Some(3),
        GROUP_LOG => Some(4),
        VALUE_CHUNK => Some(5),
        _ => None,
    }
}

/// Envelope tag for an inline value stored directly at its row key.
pub const VALUE_INLINE_TAG: u8 = 0;
/// Envelope tag for a chunked value whose body spans `VALUE_CHUNK` rows.
pub const VALUE_CHUNKED_TAG: u8 = 1;
/// Header length for a chunked value: `[tag: 1, chunks: 4, total_len: 4]`.
pub const CHUNKED_HEADER_BYTES: usize = 9;
/// Maximum payload stored inline (or in one continuation chunk), keeping every
/// physical naming page safely within `NAMING_PAGE_BYTES`.
pub const MAX_INLINE_VALUE_BYTES: usize = 24 * 1024;
/// Upper bound on a reassembled chunked value to reject hostile headers.
pub const MAX_REASSEMBLED_VALUE_BYTES: usize = 64 * 1024 * 1024;
/// Upper bound on the chunk count implied by `MAX_REASSEMBLED_VALUE_BYTES`.
pub const MAX_CHUNK_COUNT: u32 =
    MAX_REASSEMBLED_VALUE_BYTES.div_ceil(MAX_INLINE_VALUE_BYTES) as u32;

/// Validates and decodes an envelope header shared identically by reader and
/// writer. Returns `Ok(None)` for an inline value, `Ok(Some((count, total)))`
/// for a valid chunked header, and `Err(Cause::Damaged)` on any malformed
/// tag, length, count, or arithmetic inconsistency.
pub(crate) fn parse_value_envelope(
    raw: &[u8],
) -> Result<
    Option<(u32, usize)>,
    worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause,
>{
    use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
    match raw.first().copied() {
        Some(VALUE_INLINE_TAG) => {
            if raw.len() - 1 > MAX_INLINE_VALUE_BYTES {
                return Err(Cause::Damaged);
            }
            Ok(None)
        }
        Some(VALUE_CHUNKED_TAG) if raw.len() == CHUNKED_HEADER_BYTES => {
            let count = u32::from_be_bytes([raw[1], raw[2], raw[3], raw[4]]);
            let total = u32::from_be_bytes([raw[5], raw[6], raw[7], raw[8]]) as usize;
            let expected = total.div_ceil(MAX_INLINE_VALUE_BYTES);
            if total <= MAX_INLINE_VALUE_BYTES
                || total > MAX_REASSEMBLED_VALUE_BYTES
                || count == 0
                || count > MAX_CHUNK_COUNT
                || count as usize != expected
            {
                return Err(Cause::Damaged);
            }
            Ok(Some((count, total)))
        }
        _ => Err(Cause::Damaged),
    }
}

/// Validates and encodes a chunked continuation header, enforcing the exact
/// same `MAX_REASSEMBLED_VALUE_BYTES` and `MAX_CHUNK_COUNT` bounds as
/// [`parse_value_envelope`].
pub(crate) fn encode_chunked_header(
    total_len: usize,
) -> Result<
    (u32, Vec<u8>),
    worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause,
>{
    use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
    if total_len > MAX_REASSEMBLED_VALUE_BYTES {
        return Err(Cause::LimitExceeded {
            length: total_len as u64,
            limit: MAX_REASSEMBLED_VALUE_BYTES as u64,
        });
    }
    let count = total_len.div_ceil(MAX_INLINE_VALUE_BYTES) as u32;
    let total = total_len as u32;
    let mut header = Vec::with_capacity(CHUNKED_HEADER_BYTES);
    header.push(VALUE_CHUNKED_TAG);
    header.extend_from_slice(&count.to_be_bytes());
    header.extend_from_slice(&total.to_be_bytes());
    Ok((count, header))
}

/// The row key of `key` in `table`.
pub fn row_key<K: NamingCodec>(table: u8, key: &K) -> Vec<u8> {
    let mut out = Writer::default();
    out.u8(table);
    key.write(&mut out);
    out.into_bytes()
}

/// Continuation chunk row key for `base_key` at `chunk_index`.
pub fn chunk_key(base_key: &[u8], chunk_index: u32) -> Vec<u8> {
    let mut key = Vec::with_capacity(1 + 2 + base_key.len() + 4);
    key.push(VALUE_CHUNK);
    key.extend_from_slice(&(base_key.len() as u16).to_be_bytes());
    key.extend_from_slice(base_key);
    key.extend_from_slice(&chunk_index.to_be_bytes());
    key
}

/// Parses a `VALUE_CHUNK` row key into `(base_key, chunk_index)`.
pub fn parse_chunk_key(key: &[u8]) -> Option<(&[u8], u32)> {
    if key.len() < 8 || key[0] != VALUE_CHUNK {
        return None;
    }
    let len = u16::from_be_bytes([key[1], key[2]]) as usize;
    if len == 0 || key.len() != 3 + len + 4 {
        return None;
    }
    let base = &key[3..3 + len];
    let idx = u32::from_be_bytes(key[3 + len..].try_into().ok()?);
    (base[0] != VALUE_CHUNK && table_domain(base[0]).is_some()).then_some((base, idx))
}

/// Fixed 32-byte digest for `BranchId` in `GROUP_LOG` keys.
pub fn branch_key_digest(
    branch: &crate::contracts::history::data::coordinates::BranchId,
) -> [u8; 32] {
    worth_relational_storage::internal_port::storage::residency::chunked::member_frame::domain_sha256(
        b"worth.naming.group_log.branch.v1\0",
        branch.0.as_bytes(),
    )
}

/// Decodes a `GROUP_LOG` row key into `(commit, branch_digest)`.
pub fn parse_group_log_key(
    key: &[u8],
) -> Result<
    (
        crate::contracts::history::data::coordinates::CommitId,
        [u8; 32],
    ),
    worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause,
>{
    use crate::contracts::history::data::coordinates::CommitId;
    use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
    if key.first() != Some(&GROUP_LOG) {
        return Err(Cause::Damaged);
    }
    let ((_ordinal, commit), digest) = <((u64, CommitId), [u8; 32])>::from_bytes(&key[1..])?;
    Ok((commit, digest))
}

/// Computes sorted per-branch `(first_commit, last_commit)` spans for a
/// primary `GROUP_LOG` page (empty for all non-`GROUP_LOG` pages).
pub fn group_log_branch_spans(
    rows: &[(Vec<u8>, Vec<u8>)],
) -> Result<
    Vec<super::GroupLogBranchSpan>,
    worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause,
>{
    use std::collections::BTreeMap;
    use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
    if rows
        .first()
        .is_none_or(|(key, _)| key.first() != Some(&GROUP_LOG))
    {
        return Ok(Vec::new());
    }
    let mut spans: BTreeMap<[u8; 32], (u64, u64)> = BTreeMap::new();
    for (key, _) in rows {
        let (commit, digest) = parse_group_log_key(key)?;
        if commit.0 == 0 {
            return Err(Cause::Damaged);
        }
        spans
            .entry(digest)
            .and_modify(|(lo, hi)| {
                *lo = (*lo).min(commit.0);
                *hi = (*hi).max(commit.0);
            })
            .or_insert((commit.0, commit.0));
    }
    Ok(spans
        .into_iter()
        .map(
            |(branch_digest, (first_commit, last_commit))| super::GroupLogBranchSpan {
                branch_digest,
                first_commit,
                last_commit,
            },
        )
        .collect())
}

/// Cryptographic commitment to a primary page's ascending `(base_key, chunks)` pairs.
pub fn continuation_digest(pairs: &[(&[u8], u32)]) -> Option<[u8; 32]> {
    if pairs.is_empty() {
        return None;
    }
    let mut buf = Vec::new();
    for &(key, count) in pairs {
        buf.extend_from_slice(&(key.len() as u32).to_be_bytes());
        buf.extend_from_slice(key);
        buf.extend_from_slice(&count.to_be_bytes());
    }
    Some(
        worth_relational_storage::internal_port::storage::residency::chunked::member_frame::domain_sha256(
            b"worth.naming.page.continuations.v1\0",
            &buf,
        ),
    )
}

/// Computes `(continuation_chunks, continuation_digest)` once over a page's rows.
pub fn page_continuation_summary(
    rows: &[(Vec<u8>, Vec<u8>)],
) -> Result<
    (u32, Option<[u8; 32]>),
    worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause,
>{
    use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
    let mut total: u32 = 0;
    let mut pairs: Vec<(&[u8], u32)> = Vec::new();
    for (key, value) in rows {
        if key[0] != VALUE_CHUNK
            && let Some((count, _)) = parse_value_envelope(value)?
        {
            total = total.checked_add(count).ok_or(Cause::Damaged)?;
            pairs.push((key.as_slice(), count));
        }
    }
    Ok((total, continuation_digest(&pairs)))
}

/// The prefix every row of `table` scoped by `scope` starts with.
pub fn scoped_prefix(table: u8, scope: &str) -> Vec<u8> {
    row_key(table, &scope.to_owned())
}
