use std::collections::BTreeSet;

use super::state::UiSelectionOwnerRecord;

pub(super) struct UiSelectionMutation {
    pub(super) added: Vec<super::UiSelectionStableKey>,
    pub(super) removed: Vec<super::UiSelectionStableKey>,
}

pub(super) fn validate_request(
    record: &UiSelectionOwnerRecord,
    request: super::UiSelectionRequest,
) -> Result<u32, super::UiSelectionRequestDenial> {
    match request {
        super::UiSelectionRequest::SelectSingle(key) => {
            require_key(record, key)?;
            Ok(u32::try_from(record.selected.len().saturating_add(1)).unwrap_or(u32::MAX))
        }
        #[cfg(any(test, feature = "certification-support"))]
        super::UiSelectionRequest::ToggleMultiple(key) => {
            require_multiple(record)?;
            require_key(record, key)?;
            Ok(1)
        }
        super::UiSelectionRequest::Add(key) | super::UiSelectionRequest::Remove(key) => {
            require_multiple(record)?;
            require_key(record, key)?;
            Ok(1)
        }
        super::UiSelectionRequest::SelectRange { target, .. } => {
            if record.policy != super::UiSelectionPolicy::MultipleWithRange {
                return Err(super::UiSelectionRequestDenial::RangeNotSupported);
            }
            let anchor = record
                .anchor
                .ok_or(super::UiSelectionRequestDenial::MissingRangeAnchor)?;
            let anchor_index = position(record, anchor)
                .ok_or(super::UiSelectionRequestDenial::MissingRangeAnchor)?;
            let target_index =
                position(record, target).ok_or(super::UiSelectionRequestDenial::UnknownKey)?;
            let visited = anchor_index
                .abs_diff(target_index)
                .saturating_add(1)
                .saturating_add(record.selected.len());
            Ok(u32::try_from(visited).unwrap_or(u32::MAX))
        }
    }
}

pub(super) fn apply_request(
    record: &mut UiSelectionOwnerRecord,
    request: super::UiSelectionRequest,
) -> Result<UiSelectionMutation, super::UiSelectionRequestDenial> {
    match request {
        super::UiSelectionRequest::SelectSingle(key) => select_single(record, key),
        #[cfg(any(test, feature = "certification-support"))]
        super::UiSelectionRequest::ToggleMultiple(key) => toggle_selection(record, key),
        super::UiSelectionRequest::Add(key) => add_selection(record, key),
        super::UiSelectionRequest::Remove(key) => remove_selection(record, key),
        super::UiSelectionRequest::SelectRange { target, extend } => {
            select_range(record, target, extend)
        }
    }
}

fn select_single(
    record: &mut UiSelectionOwnerRecord,
    key: super::UiSelectionStableKey,
) -> Result<UiSelectionMutation, super::UiSelectionRequestDenial> {
    let added = (!record.selected.contains(&key))
        .then_some(key)
        .into_iter()
        .collect();
    let removed = record
        .selected
        .iter()
        .filter(|item| **item != key)
        .copied()
        .collect();
    record.selected.clear();
    record.selected.insert(key);
    record.anchor = Some(key);
    record.cursor = Some(key);
    Ok(mutation(added, removed))
}

#[cfg(any(test, feature = "certification-support"))]
fn toggle_selection(
    record: &mut UiSelectionOwnerRecord,
    key: super::UiSelectionStableKey,
) -> Result<UiSelectionMutation, super::UiSelectionRequestDenial> {
    let (added, removed) = if record.selected.remove(&key) {
        (Vec::new(), vec![key])
    } else {
        record.selected.insert(key);
        (vec![key], Vec::new())
    };
    record.anchor.get_or_insert(key);
    record.cursor = Some(key);
    Ok(mutation(added, removed))
}

fn add_selection(
    record: &mut UiSelectionOwnerRecord,
    key: super::UiSelectionStableKey,
) -> Result<UiSelectionMutation, super::UiSelectionRequestDenial> {
    let added = record
        .selected
        .insert(key)
        .then_some(key)
        .into_iter()
        .collect();
    record.anchor.get_or_insert(key);
    record.cursor = Some(key);
    Ok(mutation(added, Vec::new()))
}

fn remove_selection(
    record: &mut UiSelectionOwnerRecord,
    key: super::UiSelectionStableKey,
) -> Result<UiSelectionMutation, super::UiSelectionRequestDenial> {
    let removed = record
        .selected
        .remove(&key)
        .then_some(key)
        .into_iter()
        .collect();
    if record.anchor == Some(key) {
        record.anchor = None;
    }
    if record.cursor == Some(key) {
        record.cursor = None;
    }
    Ok(mutation(Vec::new(), removed))
}

fn select_range(
    record: &mut UiSelectionOwnerRecord,
    target: super::UiSelectionStableKey,
    extend: bool,
) -> Result<UiSelectionMutation, super::UiSelectionRequestDenial> {
    let anchor = record
        .anchor
        .ok_or(super::UiSelectionRequestDenial::MissingRangeAnchor)?;
    let anchor_index =
        position(record, anchor).ok_or(super::UiSelectionRequestDenial::MissingRangeAnchor)?;
    let target_index =
        position(record, target).ok_or(super::UiSelectionRequestDenial::UnknownKey)?;
    let (start, end) = if anchor_index <= target_index {
        (anchor_index, target_index)
    } else {
        (target_index, anchor_index)
    };
    let range = record.catalog[start..=end].to_vec();
    let range_set = range.iter().copied().collect::<BTreeSet<_>>();
    let removed = if extend {
        Vec::new()
    } else {
        record
            .selected
            .iter()
            .filter(|key| !range_set.contains(key))
            .copied()
            .collect()
    };
    let added = range
        .iter()
        .filter(|key| !record.selected.contains(key))
        .copied()
        .collect();
    if !extend {
        record.selected.clear();
    }
    record.selected.extend(range);
    record.cursor = Some(target);
    Ok(mutation(added, removed))
}

fn require_multiple(
    record: &UiSelectionOwnerRecord,
) -> Result<(), super::UiSelectionRequestDenial> {
    if record.policy == super::UiSelectionPolicy::Single {
        Err(super::UiSelectionRequestDenial::MultipleNotSupported)
    } else {
        Ok(())
    }
}

fn mutation(
    added: Vec<super::UiSelectionStableKey>,
    removed: Vec<super::UiSelectionStableKey>,
) -> UiSelectionMutation {
    UiSelectionMutation { added, removed }
}

fn position(record: &UiSelectionOwnerRecord, key: super::UiSelectionStableKey) -> Option<usize> {
    record.catalog_positions.get(&key).copied()
}

fn require_key(
    record: &UiSelectionOwnerRecord,
    key: super::UiSelectionStableKey,
) -> Result<(), super::UiSelectionRequestDenial> {
    position(record, key)
        .map(|_| ())
        .ok_or(super::UiSelectionRequestDenial::UnknownKey)
}
