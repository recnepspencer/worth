use super::duplicate_identity::duplicate_groups;
use super::families::{OfflineSelectorFacts, SelectorRole};
use super::root_protocol_walk::{RootEntry, SelectorEntry};
use super::{
    BoundedMediaWalk, OfflineIntegrityOutcome, OfflinePhysicalBlastRadius,
    OfflinePhysicalDamageCause, OfflinePhysicalDamageLocalization, OfflinePhysicalFormatField,
    OfflineUnknownPhysicalReason,
};

pub(crate) fn mark_selector_duplicates(
    selectors: &mut [SelectorEntry],
    walk: &mut BoundedMediaWalk,
) {
    let groups = duplicate_groups(selectors.iter().enumerate().filter_map(|(index, entry)| {
        (entry.outcome == OfflineIntegrityOutcome::Intact)
            .then_some(entry.facts.as_ref())
            .flatten()
            .map(|facts| (index, facts.selector_identity))
    }));
    for group in groups {
        let distinct_physical_files = group
            .iter()
            .filter(|index| selectors[**index].physical_alias_of.is_none())
            .count();
        walk.counters_mut().duplicate_identities +=
            distinct_physical_files.saturating_sub(1) as u64;
        for index in group {
            selectors[index].semantic_duplicate = true;
        }
    }
}

pub(crate) fn mark_root_duplicates(roots: &mut [RootEntry], walk: &mut BoundedMediaWalk) {
    let groups = duplicate_groups(roots.iter().enumerate().filter_map(|(index, entry)| {
        (entry.exact_scope_established && entry.outcome == OfflineIntegrityOutcome::Intact)
            .then_some(entry.facts.as_ref())
            .flatten()
            .map(|facts| (index, facts.generation))
    }));
    for group in groups {
        let distinct_physical_files = group
            .iter()
            .filter(|index| roots[**index].physical_alias_of.is_none())
            .count();
        walk.counters_mut().duplicate_identities +=
            distinct_physical_files.saturating_sub(1) as u64;
        for index in group {
            roots[index].semantic_duplicate = true;
        }
    }
}

pub(crate) fn apply_selector_store_scope(
    selectors: &mut [SelectorEntry],
    expected: Option<[u8; 16]>,
) {
    for entry in selectors {
        let Some(facts) = entry.facts.as_ref() else {
            continue;
        };
        match expected {
            Some(identity) if facts.store_identity != identity => {
                entry.outcome = damage(
                    OfflinePhysicalDamageCause::ScopeMismatch,
                    Some((48, 16)),
                    Some(OfflinePhysicalFormatField::StoreIdentity),
                    OfflinePhysicalBlastRadius::Field,
                );
                entry.facts = None;
            }
            None => {
                entry.outcome = OfflineIntegrityOutcome::Unknown(
                    OfflineUnknownPhysicalReason::StoreIdentityUnavailable,
                );
                entry.facts = None;
            }
            _ => {}
        }
    }
}

pub(crate) fn apply_selector_linkage(selectors: &mut [SelectorEntry]) {
    let current = selectors
        .iter()
        .position(|entry| entry.canonical && entry.role == SelectorRole::Current);
    let previous = selectors
        .iter()
        .position(|entry| entry.canonical && entry.role == SelectorRole::Previous);
    let (Some(current), Some(previous)) = (current, previous) else {
        return;
    };
    let current_facts = selectors[current].facts.clone();
    let previous_facts = selectors[previous].facts.clone();
    let (Some(current_facts), Some(previous_facts)) = (current_facts, previous_facts) else {
        return;
    };
    if current_facts.format != previous_facts.format {
        let format_damage = damage(
            OfflinePhysicalDamageCause::ScopeMismatch,
            Some((10, 10)),
            Some(OfflinePhysicalFormatField::EmbeddedFormat),
            OfflinePhysicalBlastRadius::Field,
        );
        selectors[current].outcome = format_damage.clone();
        selectors[current].facts = None;
        selectors[previous].outcome = format_damage;
        selectors[previous].facts = None;
        return;
    }
    if !link_matches(&current_facts, &previous_facts) {
        selectors[current].outcome = pointer_link_damage();
        selectors[current].facts = None;
    }
    if !link_matches(&previous_facts, &current_facts) {
        selectors[previous].outcome = pointer_link_damage();
        selectors[previous].facts = None;
    }
}

pub(crate) fn apply_selector_candidate_scope(selectors: &mut [SelectorEntry]) {
    for candidate_index in 0..selectors.len() {
        if selectors[candidate_index].canonical || selectors[candidate_index].facts.is_none() {
            continue;
        }
        let role = selectors[candidate_index].role;
        let same_role = selectors.iter().find(|entry| {
            entry.canonical
                && entry.role == role
                && entry.outcome == OfflineIntegrityOutcome::Intact
        });
        let opposite = selectors.iter().find(|entry| {
            entry.canonical
                && entry.role != role
                && entry.outcome == OfflineIntegrityOutcome::Intact
        });
        let candidate = selectors[candidate_index]
            .facts
            .clone()
            .expect("checked facts");
        let Some(authority) = same_role.and_then(|entry| entry.facts.as_ref()) else {
            selectors[candidate_index].outcome =
                OfflineIntegrityOutcome::Unknown(OfflineUnknownPhysicalReason::SelectorUnavailable);
            selectors[candidate_index].facts = None;
            continue;
        };
        if candidate.format != authority.format {
            selectors[candidate_index].outcome = damage(
                OfflinePhysicalDamageCause::ScopeMismatch,
                Some((10, 10)),
                Some(OfflinePhysicalFormatField::EmbeddedFormat),
                OfflinePhysicalBlastRadius::Field,
            );
            selectors[candidate_index].facts = None;
            continue;
        }
        if candidate.linked_selector_identity.is_some()
            && opposite
                .and_then(|entry| entry.facts.as_ref())
                .is_none_or(|target| !link_matches(&candidate, target))
        {
            selectors[candidate_index].outcome = pointer_link_damage();
            selectors[candidate_index].facts = None;
        }
    }
}

fn link_matches(source: &OfflineSelectorFacts, target: &OfflineSelectorFacts) -> bool {
    source.linked_selector_identity.is_none()
        || (source.linked_selector_identity == Some(target.selector_identity)
            && source.linked_root_generation == Some(target.root_generation))
}

fn pointer_link_damage() -> OfflineIntegrityOutcome {
    damage(
        OfflinePhysicalDamageCause::Pointer,
        Some((73, 16)),
        Some(OfflinePhysicalFormatField::LinkedSelector),
        OfflinePhysicalBlastRadius::Field,
    )
}

fn damage(
    cause: OfflinePhysicalDamageCause,
    range: Option<(u64, u64)>,
    field: Option<OfflinePhysicalFormatField>,
    blast: OfflinePhysicalBlastRadius,
) -> OfflineIntegrityOutcome {
    OfflineIntegrityOutcome::Damaged(OfflinePhysicalDamageLocalization::new(
        cause, range, field, blast,
    ))
}
