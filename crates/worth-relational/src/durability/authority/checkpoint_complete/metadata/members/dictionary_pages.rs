//! ID ranges and spelling buckets reuse exact borrowed dictionary entries.
use super::super::encode_dictionary;
use super::*;
use crate::symbols::data::{StoredSymbolText, Symbol};
use sha2::{Digest, Sha256};
use worth_execution::ExecutionRequestArrayBuilder;
const CAPACITY: usize = crate::durability::checkpoints::symbol_pages::SYMBOL_PAGE_CAPACITY;
pub(super) fn capture(
    dictionary: &OriginalSymbolDictionary<'_>,
    request: &ExecutionRequest<'_, '_>,
    members: &mut ExecutionOrderedMap<RelationalDurableMemberLocator, RelationalCheckpointMember>,
) -> Result<(CheckpointPages, CheckpointSymbolBuckets), Denial> {
    let mut entries =
        ExecutionRequestArrayBuilder::allocate(*request, dictionary.len()).map_err(array_stop)?;
    let mut buckets = ExecutionOrderedMap::<(u8, Symbol), &StoredSymbolText>::default();
    let mut counts = [0usize; 256];
    for (symbol, text) in dictionary.entries() {
        crate::execution::check_request(request).map_err(Denial::Request)?;
        let mut hash = Sha256::new();
        for bytes in text.as_str().as_bytes().chunks(64 * 1024) {
            crate::execution::check_request(request).map_err(Denial::Request)?;
            hash.update(bytes);
        }
        let bucket = hash.finalize()[0];
        counts[bucket as usize] = counts[bucket as usize]
            .checked_add(1)
            .ok_or_else(super::super::overflow)?;
        entries.push((symbol, text)).map_err(array_stop)?;
        buckets
            .insert_cloned_native_for_request(&(bucket, symbol), &text, *request)
            .map_err(map_stop)?;
    }
    let entries = entries.seal().map_err(array_stop)?;
    let ranges = || {
        entries.chunk_by(|a, b| {
            RelationalCheckpointPage::range_of(u64::from(a.0 .0))
                == RelationalCheckpointPage::range_of(u64::from(b.0 .0))
        })
    };
    let mut by_id = CheckpointPages::allocate_for_request(ranges().count(), request)
        .map_err(Denial::Request)?;
    for page in ranges() {
        let member = add(
            members,
            encode_dictionary(page.len(), || page.iter().copied(), request).map_err(stop)?,
            request,
        )?;
        let row =
            RelationalCheckpointPage::from_ids(page.iter().map(|(id, _)| u64::from(id.0)), member)
                .ok_or(Denial::OriginalAuthorityUnavailable)?;
        by_id.push(row).map_err(Denial::Request)?;
    }
    let mut by_text = CheckpointSymbolBuckets::allocate_for_request(
        counts.iter().filter(|n| **n != 0).count(),
        request,
    )
    .map_err(Denial::Request)?;
    let mut sorted = buckets.iter().peekable();
    for (bucket, count) in counts
        .into_iter()
        .enumerate()
        .filter(|(_, count)| *count != 0)
    {
        let page_count = count.div_ceil(CAPACITY);
        let mut pages =
            ExecutionRequestArrayBuilder::allocate(*request, page_count).map_err(array_stop)?;
        let mut remaining = count;
        while remaining != 0 {
            let size = remaining.min(CAPACITY);
            // Fixed borrowed scratch: no owned text or tuple heap is created.
            let mut page = [None; CAPACITY];
            for slot in &mut page[..size] {
                let ((actual, symbol), text) =
                    sorted.next().ok_or(Denial::OriginalAuthorityUnavailable)?;
                if usize::from(*actual) != bucket {
                    return Err(Denial::OriginalAuthorityUnavailable);
                }
                *slot = Some((*symbol, *text));
            }
            let member = add(
                members,
                encode_dictionary(
                    size,
                    || {
                        page[..size]
                            .iter()
                            .map(|row| row.expect(/* Proof: the immediately preceding loop fills each of these size slots from the immutable sorted dictionary before either serialization pass. */ "filled fixed dictionary page"))
                    },
                    request,
                )
                .map_err(stop)?,
                request,
            )?;
            pages.push(member).map_err(array_stop)?;
            remaining -= size;
        }
        let pages = pages.seal().map_err(array_stop)?;
        by_text
            .push(bucket as u8, &pages, request)
            .map_err(Denial::Request)?;
    }
    if sorted.next().is_some() {
        return Err(Denial::OriginalAuthorityUnavailable);
    }
    Ok((by_id.seal(), by_text.seal()))
}
