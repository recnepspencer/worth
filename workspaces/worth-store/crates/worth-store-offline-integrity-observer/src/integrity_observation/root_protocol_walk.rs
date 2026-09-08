use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::families::{
    read_current_selector, read_previous_selector, read_root_manifest, OfflineRootManifestFacts,
    OfflineSelectorFacts, SelectorRole,
};
use super::root_protocol_identity::{
    apply_selector_candidate_scope, apply_selector_linkage, apply_selector_store_scope,
    mark_root_duplicates, mark_selector_duplicates,
};
use super::root_protocol_paths::{
    add_missing_selector, root_manifest_generation, selector_order, selector_path_role,
};
use super::root_protocol_projection::{root_observations, selector_observations};
use super::unknown_artifact::{relative_path, unknown_artifact};
use super::{
    BoundedMediaWalk, OfflineArtifactObservation, OfflineIndeterminatePhysicalReason,
    OfflineIntegrityObservationDenial, OfflineIntegrityOutcome, OfflinePhysicalBlastRadius,
    OfflinePhysicalDamageCause, OfflinePhysicalDamageLocalization, OfflinePhysicalFormatField,
    OfflineUnknownPhysicalReason,
};

const RECORDS_RELATIVE: &str = "families/records";
const ROOTS_RELATIVE: &str = "families/records/roots";

pub(crate) struct SelectorEntry {
    pub(crate) path: PathBuf,
    pub(crate) relative: String,
    pub(crate) role: SelectorRole,
    pub(crate) canonical: bool,
    pub(crate) byte_length: usize,
    pub(crate) facts: Option<OfflineSelectorFacts>,
    pub(crate) observed_identity: Option<u64>,
    pub(crate) outcome: OfflineIntegrityOutcome,
    pub(crate) physical_alias_of: Option<String>,
    pub(crate) semantic_duplicate: bool,
}

pub(crate) struct RootEntry {
    pub(crate) relative: String,
    pub(crate) expected_generation: u64,
    pub(crate) facts: Option<OfflineRootManifestFacts>,
    pub(crate) byte_length: usize,
    pub(crate) outcome: OfflineIntegrityOutcome,
    pub(crate) exact_scope_established: bool,
    pub(crate) physical_alias_of: Option<String>,
    pub(crate) semantic_duplicate: bool,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct AddressedRootExpectation {
    pub(crate) generation: u64,
    pub(crate) format: [u8; 10],
}

pub(crate) struct ObservedRootProtocol {
    pub(crate) artifacts: Vec<OfflineArtifactObservation>,
    pub(crate) roots: Vec<OfflineRootManifestFacts>,
}

pub(crate) fn observe_root_protocol(
    store_root: &Path,
    expected_store_identity: Option<[u8; 16]>,
    walk: &mut BoundedMediaWalk,
) -> Result<ObservedRootProtocol, OfflineIntegrityObservationDenial> {
    let (mut selectors, mut unknowns) = read_selector_entries(store_root, walk)?;
    apply_selector_store_scope(&mut selectors, expected_store_identity);
    apply_selector_linkage(&mut selectors);
    apply_selector_candidate_scope(&mut selectors);
    mark_selector_duplicates(&mut selectors, walk);
    let addressed = addressed_root_expectations(&selectors);
    let (mut roots, root_incomplete, root_unknowns) =
        read_root_entries(store_root, &addressed, walk)?;
    unknowns.extend(root_unknowns);
    mark_root_duplicates(&mut roots, walk);
    mark_missing_selector_pointers(&mut selectors, &roots, root_incomplete, walk);
    let admitted_roots = roots
        .iter()
        .filter(|entry| {
            entry.exact_scope_established && entry.outcome == OfflineIntegrityOutcome::Intact
        })
        .filter_map(|entry| entry.facts)
        .collect();
    let mut observations = selector_observations(selectors);
    observations.extend(root_observations(&addressed, roots, root_incomplete));
    observations.extend(unknowns);
    observations.sort_by(|left, right| left.relative_path().cmp(right.relative_path()));
    Ok(ObservedRootProtocol {
        artifacts: observations,
        roots: admitted_roots,
    })
}

fn read_selector_entries(
    store_root: &Path,
    walk: &mut BoundedMediaWalk,
) -> Result<(Vec<SelectorEntry>, Vec<OfflineArtifactObservation>), OfflineIntegrityObservationDenial>
{
    let records = store_root.join(RECORDS_RELATIVE);
    let scan = walk
        .scan_directory(&records, 2)
        .map_err(|_| OfflineIntegrityObservationDenial::RecordDirectoryUnavailable)?;
    let mut selector_paths = Vec::new();
    let mut unknown_paths = Vec::new();
    for path in &scan.entries {
        let Some((role, canonical)) = selector_path_role(path) else {
            if path.file_name().is_some_and(|name| {
                ![
                    "roots",
                    "bootstrap.catalog",
                    "segments",
                    "segment-manifests",
                    "extents",
                    "extent-manifests",
                    "free-space",
                ]
                .iter()
                .any(|known| name == *known)
            }) {
                unknown_paths.push(path.clone());
            }
            continue;
        };
        selector_paths.push((path.clone(), role, canonical));
    }
    selector_paths.sort_by(|left, right| {
        (!left.2, left.1 as u8, &left.0).cmp(&(!right.2, right.1 as u8, &right.0))
    });
    let mut selectors: Vec<_> = selector_paths
        .into_iter()
        .map(|(path, role, canonical)| {
            read_selector_entry(store_root, &path, role, canonical, walk)
        })
        .collect();
    let unknowns = unknown_paths
        .into_iter()
        .map(|path| unknown_artifact(store_root, &path, 3, walk))
        .collect();
    add_missing_selector(
        &mut selectors,
        SelectorRole::Current,
        store_root,
        scan.incomplete_reason,
        walk,
    );
    add_missing_selector(
        &mut selectors,
        SelectorRole::Previous,
        store_root,
        scan.incomplete_reason,
        walk,
    );
    selectors.sort_by(|left, right| selector_order(left).cmp(&selector_order(right)));
    Ok((selectors, unknowns))
}

fn read_selector_entry(
    store_root: &Path,
    path: &Path,
    role: SelectorRole,
    canonical: bool,
    walk: &mut BoundedMediaWalk,
) -> SelectorEntry {
    let relative = relative_path(store_root, path);
    let acquired = walk.acquire(path, 3);
    let byte_length = acquired.as_ref().map_or(0, |value| value.byte_length);
    let physical_alias_of = acquired
        .as_ref()
        .ok()
        .and_then(|value| value.physical_alias_of.as_ref())
        .map(|path| relative_path(store_root, path));
    let parsed = acquired.and_then(|acquired| {
        if acquired.physical_alias_of.is_some() {
            return Err(OfflineIntegrityOutcome::Unknown(
                OfflineUnknownPhysicalReason::PhysicalAliasNotReinspected,
            ));
        }
        match role {
            SelectorRole::Current => read_current_selector(&acquired.bytes, walk.counters_mut()),
            SelectorRole::Previous => read_previous_selector(&acquired.bytes, walk.counters_mut()),
        }
    });
    let (facts, observed_identity, outcome) = match parsed {
        Ok(facts) => {
            let identity = facts.selector_identity;
            (Some(facts), Some(identity), OfflineIntegrityOutcome::Intact)
        }
        Err(outcome) => (None, None, outcome),
    };
    walk.record_outcome(&outcome);
    SelectorEntry {
        path: path.to_path_buf(),
        relative,
        role,
        canonical,
        byte_length,
        facts,
        observed_identity,
        outcome,
        physical_alias_of,
        semantic_duplicate: false,
    }
}

fn read_root_entries(
    store_root: &Path,
    addressed: &BTreeMap<u64, AddressedRootExpectation>,
    walk: &mut BoundedMediaWalk,
) -> Result<
    (
        Vec<RootEntry>,
        Option<OfflineIndeterminatePhysicalReason>,
        Vec<OfflineArtifactObservation>,
    ),
    OfflineIntegrityObservationDenial,
> {
    let roots_path = store_root.join(ROOTS_RELATIVE);
    let scan = walk
        .scan_directory(&roots_path, 3)
        .map_err(|_| OfflineIntegrityObservationDenial::RootDirectoryUnavailable)?;
    let mut candidates = Vec::new();
    let mut unknown_paths = Vec::new();
    for path in scan.entries {
        match root_manifest_generation(&path) {
            Some(generation) => candidates.push((path, generation)),
            None if path
                .file_name()
                .is_some_and(|name| name.to_string_lossy().contains("-block-")) => {}
            None => unknown_paths.push(path),
        }
    }
    candidates.sort_by_key(|(_, generation)| !addressed.contains_key(generation));
    let roots = candidates
        .into_iter()
        .map(|(path, generation)| {
            read_root_entry(
                store_root,
                &path,
                generation,
                addressed.get(&generation).copied(),
                walk,
            )
        })
        .collect();
    let unknowns = unknown_paths
        .into_iter()
        .map(|path| unknown_artifact(store_root, &path, 4, walk))
        .collect();
    Ok((roots, scan.incomplete_reason, unknowns))
}

fn read_root_entry(
    store_root: &Path,
    path: &Path,
    expected_generation: u64,
    addressed: Option<AddressedRootExpectation>,
    walk: &mut BoundedMediaWalk,
) -> RootEntry {
    let relative = relative_path(store_root, path);
    let acquired = walk.acquire(path, 4);
    let byte_length = acquired.as_ref().map_or(0, |value| value.byte_length);
    let physical_alias_of = acquired
        .as_ref()
        .ok()
        .and_then(|value| value.physical_alias_of.as_ref())
        .map(|path| relative_path(store_root, path));
    let parsed = acquired.and_then(|acquired| {
        if acquired.physical_alias_of.is_some() {
            return Err(OfflineIntegrityOutcome::Unknown(
                OfflineUnknownPhysicalReason::PhysicalAliasNotReinspected,
            ));
        }
        read_root_manifest(&acquired.bytes, walk.counters_mut())
    });
    let (facts, outcome, exact_scope_established) = match (parsed, addressed) {
        (Ok(facts), None) => (
            Some(facts),
            OfflineIntegrityOutcome::Unknown(OfflineUnknownPhysicalReason::RootNotAddressed),
            false,
        ),
        (Ok(facts), Some(expectation)) if facts.generation != expectation.generation => (
            Some(facts),
            damage(
                OfflinePhysicalDamageCause::ScopeMismatch,
                Some((48, 8)),
                Some(OfflinePhysicalFormatField::ManifestGeneration),
                OfflinePhysicalBlastRadius::Field,
            ),
            false,
        ),
        (Ok(facts), Some(expectation)) if facts.format != expectation.format => (
            Some(facts),
            damage(
                OfflinePhysicalDamageCause::ScopeMismatch,
                Some((10, 10)),
                Some(OfflinePhysicalFormatField::EmbeddedFormat),
                OfflinePhysicalBlastRadius::Field,
            ),
            false,
        ),
        (Ok(facts), Some(_)) => (Some(facts), OfflineIntegrityOutcome::Intact, true),
        (Err(outcome), _) => (None, outcome, false),
    };
    walk.record_outcome(&outcome);
    RootEntry {
        relative,
        expected_generation,
        facts,
        byte_length,
        outcome,
        exact_scope_established,
        physical_alias_of,
        semantic_duplicate: false,
    }
}

fn addressed_root_expectations(
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

fn mark_missing_selector_pointers(
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
                entry.outcome = damage(
                    OfflinePhysicalDamageCause::Pointer,
                    Some((65, 8)),
                    Some(OfflinePhysicalFormatField::RootGeneration),
                    OfflinePhysicalBlastRadius::ReachableRootSubtree,
                );
            }
        }
    }
    if incomplete.is_none() {
        walk.counters_mut().missing_artifacts += addressed.difference(&present).count() as u64;
    }
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
