//! ID ranges and spelling buckets reuse exact borrowed dictionary entries
//! and carry prior cold symbol pages forward without re-reading them.
use super::super::encode_dictionary;
use super::*;
use contracts::symbols::data::stored_text::StoredSymbolText;
use contracts::symbols::data::Symbol;
use sha2::{Digest, Sha256};
use worth_execution::ExecutionRequestArrayBuilder;
use worth_relational_contracts::internal_port as contracts;

const CAPACITY: usize =
    worth_relational_storage::internal_port::storage::data::symbol_page_shape::SYMBOL_PAGE_CAPACITY;

pub(super) fn capture(
    dictionary: &OriginalSymbolDictionary<'_>,
    prior: Option<&RelationalCheckpointManifest>,
    request: &ExecutionRequest<'_, '_>,
    members: &mut ExecutionOrderedMap<RelationalDurableMemberLocator, RelationalCheckpointMember>,
) -> Result<(CheckpointPages, CheckpointSymbolBuckets), Denial> {
    let mut ordered_by_id = ExecutionOrderedMap::<(u64, u64), RelationalCheckpointPage>::default();
    let mut occupied_by_range = ExecutionOrderedMap::<u64, [u64; 4]>::default();
    let mut id_page_members = ExecutionOrderedMap::<RelationalDurableMemberLocator, ()>::default();
    let mut prior_buckets: [Option<&[RelationalDurableMemberLocator]>; 256] = [None; 256];

    if let Some(prior) = prior {
        carry_prior_symbols(
            prior,
            &mut ordered_by_id,
            &mut occupied_by_range,
            &mut id_page_members,
            &mut prior_buckets,
            members,
            request,
        )?;
    }

    let new_count = if occupied_by_range.is_empty() {
        dictionary.len()
    } else {
        let mut count = 0usize;
        for (symbol, _) in dictionary.entries() {
            contracts::execution::request_custody::check_request(request)
                .map_err(Denial::Request)?;
            if !range_contains(&occupied_by_range, symbol) {
                count = count.checked_add(1).ok_or_else(super::super::overflow)?;
            }
        }
        count
    };

    let mut entries =
        ExecutionRequestArrayBuilder::allocate(*request, new_count).map_err(array_stop)?;
    let mut buckets = ExecutionOrderedMap::<(u8, Symbol), &StoredSymbolText>::default();
    let mut counts = [0usize; 256];
    for (symbol, text) in dictionary.entries() {
        contracts::execution::request_custody::check_request(request).map_err(Denial::Request)?;
        if range_contains(&occupied_by_range, symbol) {
            continue;
        }
        let mut hash = Sha256::new();
        for bytes in text.as_str().as_bytes().chunks(64 * 1024) {
            contracts::execution::request_custody::check_request(request)
                .map_err(Denial::Request)?;
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
    for page in entries.chunk_by(|a, b| {
        RelationalCheckpointPage::range_of(u64::from(a.0 .0))
            == RelationalCheckpointPage::range_of(u64::from(b.0 .0))
    }) {
        let member = add(
            members,
            encode_dictionary(page.len(), || page.iter().copied(), request).map_err(stop)?,
            request,
        )?;
        let row =
            RelationalCheckpointPage::from_ids(page.iter().map(|(id, _)| u64::from(id.0)), member)
                .ok_or(Denial::OriginalAuthorityUnavailable)?;
        insert_symbol_page(
            &mut ordered_by_id,
            &mut occupied_by_range,
            &mut id_page_members,
            row,
            request,
        )?;
    }
    let mut by_id = CheckpointPages::allocate_for_request(ordered_by_id.len(), request)
        .map_err(Denial::Request)?;
    for page in ordered_by_id.values() {
        by_id.push(*page).map_err(Denial::Request)?;
    }

    let bucket_count = (0..256)
        .filter(|&bucket| prior_buckets[bucket].is_some() || counts[bucket] != 0)
        .count();
    let mut by_text = CheckpointSymbolBuckets::allocate_for_request(bucket_count, request)
        .map_err(Denial::Request)?;
    let mut sorted = buckets.iter().peekable();
    for bucket in 0..256 {
        let carried = prior_buckets[bucket].unwrap_or(&[]);
        let count = counts[bucket];
        if carried.is_empty() && count == 0 {
            continue;
        }
        let page_count = carried
            .len()
            .checked_add(count.div_ceil(CAPACITY))
            .ok_or_else(super::super::overflow)?;
        let mut pages =
            ExecutionRequestArrayBuilder::allocate(*request, page_count).map_err(array_stop)?;
        let mut seen = ExecutionOrderedMap::<RelationalDurableMemberLocator, ()>::default();
        for &locator in carried {
            contracts::execution::request_custody::check_request(request)
                .map_err(Denial::Request)?;
            if count != 0 {
                seen.insert_cloned_native_for_request(&locator, &(), *request)
                    .map_err(map_stop)?;
            }
            pages.push(locator).map_err(array_stop)?;
        }
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
            if seen.contains_key(&member) {
                return Err(Denial::OriginalAuthorityUnavailable);
            }
            seen.insert_cloned_native_for_request(&member, &(), *request)
                .map_err(map_stop)?;
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

fn carry_prior_symbols<'a>(
    prior: &'a RelationalCheckpointManifest,
    ordered_by_id: &mut ExecutionOrderedMap<(u64, u64), RelationalCheckpointPage>,
    occupied_by_range: &mut ExecutionOrderedMap<u64, [u64; 4]>,
    id_page_members: &mut ExecutionOrderedMap<RelationalDurableMemberLocator, ()>,
    prior_buckets: &mut [Option<&'a [RelationalDurableMemberLocator]>; 256],
    members: &mut ExecutionOrderedMap<RelationalDurableMemberLocator, RelationalCheckpointMember>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    if prior.symbol_identity_frontier == 0
        || prior.symbols.is_empty() != prior.symbol_buckets.is_empty()
    {
        return Err(Denial::OriginalAuthorityUnavailable);
    }
    let mut previous_page = None;
    for page in &prior.symbols {
        contracts::execution::request_custody::check_request(request).map_err(Denial::Request)?;
        if !page.valid() || page.last >= u64::from(prior.symbol_identity_frontier) {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
        let key = (page.range(), page.first);
        if previous_page.is_some_and(|prior_key| key <= prior_key) {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
        previous_page = Some(key);
        carry_member(members, page.member, request)?;
        insert_symbol_page(
            ordered_by_id,
            occupied_by_range,
            id_page_members,
            *page,
            request,
        )?;
    }
    let mut previous_bucket = None;
    for bucket in &prior.symbol_buckets {
        contracts::execution::request_custody::check_request(request).map_err(Denial::Request)?;
        if previous_bucket.is_some_and(|prior_bucket| prior_bucket >= bucket.bucket)
            || bucket.pages.is_empty()
        {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
        previous_bucket = Some(bucket.bucket);
        let mut seen = ExecutionOrderedMap::<RelationalDurableMemberLocator, ()>::default();
        for &locator in &bucket.pages {
            if seen.contains_key(&locator) {
                return Err(Denial::OriginalAuthorityUnavailable);
            }
            seen.insert_cloned_native_for_request(&locator, &(), *request)
                .map_err(map_stop)?;
            carry_member(members, locator, request)?;
        }
        prior_buckets[usize::from(bucket.bucket)] = Some(&bucket.pages);
    }
    Ok(())
}

fn carry_member(
    members: &mut ExecutionOrderedMap<RelationalDurableMemberLocator, RelationalCheckpointMember>,
    locator: RelationalDurableMemberLocator,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    contracts::execution::request_custody::check_request(request).map_err(Denial::Request)?;
    if members.get(&locator).is_none() {
        members
            .insert_cloned_native_for_request(
                &locator,
                &RelationalCheckpointMember::Carried { locator },
                *request,
            )
            .map_err(map_stop)?;
    }
    Ok(())
}

fn insert_symbol_page(
    ordered_by_id: &mut ExecutionOrderedMap<(u64, u64), RelationalCheckpointPage>,
    occupied_by_range: &mut ExecutionOrderedMap<u64, [u64; 4]>,
    id_page_members: &mut ExecutionOrderedMap<RelationalDurableMemberLocator, ()>,
    page: RelationalCheckpointPage,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    if !page.valid() || id_page_members.contains_key(&page.member) {
        return Err(Denial::OriginalAuthorityUnavailable);
    }
    let key = (page.range(), page.first);
    if ordered_by_id.contains_key(&key) {
        return Err(Denial::OriginalAuthorityUnavailable);
    }
    let mut merged = occupied_by_range
        .get(&page.range())
        .copied()
        .unwrap_or([0; 4]);
    for (existing, incoming) in merged.iter_mut().zip(page.membership) {
        if *existing & incoming != 0 {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
        *existing |= incoming;
    }
    id_page_members
        .insert_cloned_native_for_request(&page.member, &(), *request)
        .map_err(map_stop)?;
    occupied_by_range
        .insert_cloned_native_for_request(&page.range(), &merged, *request)
        .map_err(map_stop)?;
    ordered_by_id
        .insert_cloned_native_for_request(&key, &page, *request)
        .map_err(map_stop)?;
    Ok(())
}

fn range_contains(occupied: &ExecutionOrderedMap<u64, [u64; 4]>, symbol: Symbol) -> bool {
    let id = u64::from(symbol.0);
    if id == 0 || id >= u64::MAX - 1 {
        return false;
    }
    let range = RelationalCheckpointPage::range_of(id);
    occupied.get(&range).is_some_and(|words| {
        let offset = (id - 1) % 256;
        words[(offset / 64) as usize] & (1 << (offset % 64)) != 0
    })
}
