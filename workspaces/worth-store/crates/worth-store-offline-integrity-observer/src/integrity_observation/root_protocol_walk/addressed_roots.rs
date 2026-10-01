//! Selector-derived root addressing and missing pointer localization.

use std::collections::{BTreeMap, BTreeSet};

use super::{AddressedRootExpectation, RootEntry, SelectorEntry};
use crate::integrity_observation::{
    BoundedMediaWalk, OfflineIndeterminatePhysicalReason, OfflineIntegrityOutcome,
    OfflinePhysicalBlastRadius, OfflinePhysicalDamageCause, OfflinePhysicalDamageLocalization,
    OfflinePhysicalFormatField,
};

pub(super) fn addressed_root_expectations(
    selectors: &[SelectorEntry],
) -> BTreeMap<u64, AddressedRootExpectation> {
    selectors
        .iter()
        .filter(|entry| {
            entry.canonical
                && !entry.semantic_duplicate
                && entry.outcome == OfflineIntegrityOutcome::Intact
        })
        .filter_map(|entry| {
            entry.facts.as_ref().map(|facts| {
                (
                    facts.root_generation,
                    AddressedRootExpectation {
                        generation: facts.root_generation,
                        format: facts.format,
                    },
                )
            })
        })
        .collect()
}

pub(super) fn mark_missing_selector_pointers(
    selectors: &mut [SelectorEntry],
    roots: &[RootEntry],
    incomplete: Option<OfflineIndeterminatePhysicalReason>,
    walk: &mut BoundedMediaWalk,
) {
    let present: BTreeSet<_> = roots
        .iter()
        .map(|entry| entry.expected_generation)
        .collect();
    let addressed: BTreeSet<_> = addressed_root_expectations(selectors).into_keys().collect();
    for entry in selectors
        .iter_mut()
        .filter(|entry| entry.canonical && !entry.semantic_duplicate)
    {
        if let Some(facts) = &entry.facts {
            if incomplete.is_none() && !present.contains(&facts.root_generation) {
                entry.outcome =
                    OfflineIntegrityOutcome::Damaged(OfflinePhysicalDamageLocalization::new(
                        OfflinePhysicalDamageCause::Pointer,
                        Some((65, 8)),
                        Some(OfflinePhysicalFormatField::RootGeneration),
                        OfflinePhysicalBlastRadius::ReachableRootSubtree,
                    ));
            }
        }
    }
    if incomplete.is_none() {
        walk.counters_mut().missing_artifacts += addressed.difference(&present).count() as u64;
    }
}
