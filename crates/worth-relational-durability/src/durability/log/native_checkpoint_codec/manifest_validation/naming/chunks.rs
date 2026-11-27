//! Read-free `VALUE_CHUNK` continuation roster validation for `CHK24` naming runs.
//!
//! Each `VALUE_CHUNK` page belongs to exactly one logical `base_key`, so the
//! manifest descriptors alone reconstruct every `(base_key, chunk_count)` pair
//! in ascending order and verify it against each primary page's cryptographic
//! `continuation_digest` before checkpoint installation (zero page reads).
//!
//! G7 two-stage authenticity law (matching `manifest_validation.rs`):
//! - Stage 1 (here) checks read-free descriptor & `continuation_digest` consistency at reopen (`0` reads).
//! - Stage 2 (`ResidentNamingPage::decode` + O(1) `admit`) authenticates `(entries, first, last, continuation_chunks, continuation_digest)` against the content-addressed page bytes on first fault.

use worth_relational_contracts::internal_port::durability::data::recovery_errors::DurabilityError;
use worth_relational_lineage::internal_port::lineage::naming_pages::{
    page_for, tables, NamingPageRef,
};

use super::super::super::manifest_capture::corrupt;

pub(super) fn validate_continuation_chunks<'a>(
    pages: &'a [NamingPageRef],
    is_group_log: bool,
) -> Result<(), DurabilityError> {
    let split = pages.partition_point(|p| p.first.first() != Some(&tables::VALUE_CHUNK));
    let (primary, chunks) = pages.split_at(split);
    let mut per_primary: Vec<Vec<(&'a [u8], u32)>> = vec![Vec::new(); primary.len()];
    let mut active: Option<(&'a [u8], u32)> = None;
    let tag_ok = |tag: u8| {
        if is_group_log {
            tag == tables::GROUP_LOG
        } else {
            tag != tables::GROUP_LOG && tag != tables::VALUE_CHUNK
        }
    };
    let mut record = |base: &'a [u8], count: u32| -> Result<(), DurabilityError> {
        if !tag_ok(base[0]) {
            return Err(corrupt(
                "checkpoint VALUE_CHUNK page targets foreign naming table",
            ));
        }
        let slot = page_for(primary, base).ok_or_else(|| {
            corrupt("checkpoint VALUE_CHUNK page references missing primary page")
        })?;
        per_primary[slot].push((base, count));
        Ok(())
    };
    for page in chunks {
        let (base_first, idx_first) = tables::parse_chunk_key(&page.first)
            .ok_or_else(|| corrupt("malformed naming VALUE_CHUNK first key"))?;
        let (base_last, idx_last) = tables::parse_chunk_key(&page.last)
            .ok_or_else(|| corrupt("malformed naming VALUE_CHUNK last key"))?;
        let span = idx_last
            .checked_sub(idx_first)
            .and_then(|d| d.checked_add(1));
        if base_first != base_last || span != Some(page.entries) {
            return Err(corrupt(
                "checkpoint VALUE_CHUNK page spans multiple keys or has gapped chunk indices",
            ));
        }
        match active {
            None => {
                if idx_first != 0 {
                    return Err(corrupt(
                        "checkpoint VALUE_CHUNK run does not start at chunk index 0",
                    ));
                }
                active = Some((base_first, page.entries));
            }
            Some((cur_base, cur_count)) if cur_base == base_first => {
                if idx_first != cur_count {
                    return Err(corrupt(
                        "checkpoint VALUE_CHUNK run has non-contiguous chunk indices",
                    ));
                }
                let next_count = cur_count
                    .checked_add(page.entries)
                    .ok_or_else(|| corrupt("checkpoint VALUE_CHUNK count overflow"))?;
                active = Some((cur_base, next_count));
            }
            Some((prev_base, prev_count)) => {
                if (base_first.len(), base_first) <= (prev_base.len(), prev_base) || idx_first != 0
                {
                    return Err(corrupt(
                        "checkpoint VALUE_CHUNK keys are unordered or do not start at chunk 0",
                    ));
                }
                record(prev_base, prev_count)?;
                active = Some((base_first, page.entries));
            }
        }
    }
    if let Some((last_base, last_count)) = active {
        record(last_base, last_count)?;
    }
    for (page, pairs) in primary.iter().zip(per_primary.iter_mut()) {
        pairs.sort_unstable_by(|a, b| a.0.cmp(b.0));
        let sum: u64 = pairs.iter().map(|&(_, c)| u64::from(c)).sum();
        if sum != u64::from(page.continuation_chunks)
            || tables::continuation_digest(pairs) != page.continuation_digest
        {
            return Err(corrupt(
                "checkpoint primary page continuation commitment disagrees with VALUE_CHUNK roster",
            ));
        }
    }
    Ok(())
}
