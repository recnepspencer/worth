//! Merge capture of one page run: prior pages plus sorted row changes.
//!
//! A prior page whose key range no change touches is carried unchanged
//! (`O(touched)` capture); a touched page is re-read, merged with its
//! changes and re-paged in place, so the pages after it keep their exact
//! members. An unchanged run re-emits the same content-addressed pages.
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use worth_relational_storage::internal_port::storage::residency::chunked::member_frame::{
    encode_member, ChunkMemberKind,
};
use worth_relational_storage::internal_port::storage::residency::chunked::member_io::ChunkMemberSink;

use super::page::NamingPage;
use super::store::ColdNamingStore;
use super::tables::{
    chunk_key, encode_chunked_header, page_continuation_summary, parse_chunk_key,
    parse_group_log_key, parse_value_envelope, table_domain, verify_group_log_single_branch,
    GROUP_LOG, MAX_INLINE_VALUE_BYTES, VALUE_CHUNK, VALUE_INLINE_TAG,
};
use super::{
    logical_key_limit, physical_key_fits, NamingPageRef, NAMING_PAGE_BYTES, NAMING_PAGE_ENTRIES,
};

/// Frame and array headers of one encoded page.
const PAGE_OVERHEAD: usize = 64;

/// A row's worst-case encoded size: each byte may take two in the frame.
fn row_bytes(key: &[u8], value: &[u8]) -> usize {
    2 * (key.len() + value.len()) + 16
}

fn prior_chunk_count(
    prior: Option<(&ColdNamingStore, &[NamingPageRef])>,
    key: &[u8],
) -> Result<u32, Cause> {
    let Some((store, pages)) = prior else {
        return Ok(0);
    };
    let Some(raw) = store.raw_get(pages, key)? else {
        return Ok(0);
    };
    Ok(parse_value_envelope(&raw)?.map_or(0, |(count, _)| count))
}

fn expand_changes(
    prior: Option<(&ColdNamingStore, &[NamingPageRef])>,
    changes: Vec<(Vec<u8>, Option<Vec<u8>>)>,
) -> Result<Vec<(Vec<u8>, Option<Vec<u8>>)>, Cause> {
    let mut expanded = Vec::with_capacity(changes.len());
    for (key, value) in changes {
        let limit = logical_key_limit(key.first().copied());
        if key.is_empty() || key.len() > limit {
            return Err(Cause::LimitExceeded {
                length: key.len() as u64,
                limit: limit as u64,
            });
        }
        let old_chunks = prior_chunk_count(prior, &key)?;
        let new_chunks = match value {
            None => {
                expanded.push((key.clone(), None));
                0
            }
            Some(bytes) if bytes.len() <= MAX_INLINE_VALUE_BYTES => {
                let mut wrapped = Vec::with_capacity(1 + bytes.len());
                wrapped.push(VALUE_INLINE_TAG);
                wrapped.extend_from_slice(&bytes);
                expanded.push((key.clone(), Some(wrapped)));
                0
            }
            Some(bytes) => {
                let (count, header) = encode_chunked_header(bytes.len())?;
                expanded.push((key.clone(), Some(header)));
                for (index, chunk) in bytes.chunks(MAX_INLINE_VALUE_BYTES).enumerate() {
                    expanded.push((chunk_key(&key, index as u32), Some(chunk.to_vec())));
                }
                count
            }
        };
        for stale in new_chunks..old_chunks {
            expanded.push((chunk_key(&key, stale), None));
        }
    }
    expanded.sort_unstable_by(|left, right| left.0.cmp(&right.0));
    Ok(expanded)
}

/// Whether `key` can share a physical page with a page starting at `page_first`.
///
/// Every physical page stays within one table domain; `VALUE_CHUNK` pages also
/// isolate a single base key, and primary `GROUP_LOG` pages isolate a single
/// branch so endpoint keys alone establish exact branch + commit routing.
fn same_page_bucket(page_first: &[u8], key: &[u8]) -> bool {
    if table_domain(page_first[0]) != table_domain(key[0]) {
        return false;
    }
    if page_first[0] == VALUE_CHUNK || key[0] == VALUE_CHUNK {
        return page_first[0] == VALUE_CHUNK
            && key[0] == VALUE_CHUNK
            && parse_chunk_key(page_first)
                .zip(parse_chunk_key(key))
                .is_some_and(|((a, _), (b, _))| a == b);
    }
    if page_first[0] == GROUP_LOG || key[0] == GROUP_LOG {
        return page_first[0] == GROUP_LOG
            && key[0] == GROUP_LOG
            && parse_group_log_key(page_first)
                .ok()
                .zip(parse_group_log_key(key).ok())
                .is_some_and(|((_, a), (_, b))| a == b);
    }
    true
}

struct PageWriter<'s> {
    sink: &'s mut dyn ChunkMemberSink,
    rows: Vec<(Vec<u8>, Vec<u8>)>,
    bytes: usize,
    pages: Vec<NamingPageRef>,
}

impl PageWriter<'_> {
    fn push(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<(), Cause> {
        let size = row_bytes(&key, &value);
        if !physical_key_fits(&key) || size + PAGE_OVERHEAD > NAMING_PAGE_BYTES {
            return Err(Cause::LimitExceeded {
                length: size as u64,
                limit: NAMING_PAGE_BYTES as u64,
            });
        }
        if self.rows.len() == NAMING_PAGE_ENTRIES as usize
            || self.bytes + size + PAGE_OVERHEAD > NAMING_PAGE_BYTES
            || self
                .rows
                .first()
                .is_some_and(|(first, _)| !same_page_bucket(first, &key))
        {
            self.flush()?;
        }
        self.bytes += size;
        self.rows.push((key, value));
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Cause> {
        let (Some(first), Some(last)) = (self.rows.first(), self.rows.last()) else {
            return Ok(());
        };
        let (first, last) = (first.0.clone(), last.0.clone());
        let rows = std::mem::take(&mut self.rows);
        let entries = rows.len() as u32;
        verify_group_log_single_branch(&rows)?;
        let (continuation_chunks, continuation_digest) = page_continuation_summary(&rows)?;
        let member = encode_member(
            ChunkMemberKind::NamingStatePage,
            &NamingPage { rows },
            NAMING_PAGE_BYTES,
        )?;
        let locator = member.locator;
        self.sink.put(locator, member.bytes)?;
        self.pages.push(NamingPageRef {
            first,
            last,
            entries,
            continuation_chunks,
            continuation_digest,
            member: locator,
        });
        self.bytes = 0;
        Ok(())
    }
}

/// Pages `prior`'s rows with `changes` applied: strictly ascending row keys,
/// each a new value or a removal (`None`). Written pages go to `sink`.
///
/// Complexity: O(prior pages + changes + rows of touched pages).
pub fn seal_pages(
    prior: Option<(&ColdNamingStore, &[NamingPageRef])>,
    changes: Vec<(Vec<u8>, Option<Vec<u8>>)>,
    sink: &mut dyn ChunkMemberSink,
) -> Result<Vec<NamingPageRef>, Cause> {
    if changes.windows(2).any(|pair| pair[0].0 >= pair[1].0) {
        return Err(Cause::Damaged);
    }
    let changes = expand_changes(prior, changes)?;
    let mut writer = PageWriter {
        sink,
        rows: Vec::new(),
        bytes: 0,
        pages: Vec::new(),
    };
    let mut changes = changes.into_iter().peekable();
    if let Some((store, pages)) = prior {
        for (ordinal, page) in pages.iter().enumerate() {
            while let Some((key, _)) = changes.peek() {
                if key.as_slice() < page.first.as_slice() && !same_page_bucket(&page.first, key) {
                    let Some((key, value)) = changes.next() else {
                        break;
                    };
                    if let Some(value) = value {
                        writer.push(key, value)?;
                    }
                } else {
                    break;
                }
            }
            let upper = pages.get(ordinal + 1).map(|next| next.first.as_slice());
            let mut touched = Vec::new();
            while let Some((key, _)) = changes.peek() {
                if upper.is_some_and(|upper| key.as_slice() >= upper) {
                    break;
                }
                if key.as_slice() > page.last.as_slice() && !same_page_bucket(&page.first, key) {
                    break;
                }
                touched.extend(changes.next());
            }
            if touched.is_empty() {
                writer.flush()?;
                writer.pages.push(page.clone());
                continue;
            }
            writer.flush()?;
            let resident = store.page(page)?;
            let mut rows = resident.page.rows.iter().peekable();
            let mut touched = touched.into_iter().peekable();
            loop {
                let take_row = match (rows.peek(), touched.peek()) {
                    (None, None) => break,
                    (Some(_), None) => true,
                    (None, Some(_)) => false,
                    (Some(row), Some(change)) => row.0 < change.0,
                };
                if take_row {
                    let (key, value) = rows.next().cloned().ok_or(Cause::Damaged)?;
                    writer.push(key, value)?;
                    continue;
                }
                let (key, value) = touched.next().ok_or(Cause::Damaged)?;
                if rows.peek().is_some_and(|row| row.0 == key) {
                    rows.next();
                }
                if let Some(value) = value {
                    writer.push(key, value)?;
                }
            }
            writer.flush()?;
        }
    }
    for (key, value) in changes {
        if let Some(value) = value {
            writer.push(key, value)?;
        }
    }
    writer.flush()?;
    Ok(writer.pages)
}
