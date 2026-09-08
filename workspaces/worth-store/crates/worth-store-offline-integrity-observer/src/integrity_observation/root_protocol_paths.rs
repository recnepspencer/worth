use std::path::Path;

use super::families::SelectorRole;
use super::root_protocol_walk::SelectorEntry;
use super::{
    BoundedMediaWalk, OfflineIndeterminatePhysicalReason, OfflineIntegrityOutcome,
    OfflinePhysicalBlastRadius, OfflinePhysicalDamageCause, OfflinePhysicalDamageLocalization,
};

const RECORDS_RELATIVE: &str = "families/records";
const CURRENT_SELECTOR: &str = "root-current.selector";
const PREVIOUS_SELECTOR: &str = "root-previous.selector";

pub(crate) fn add_missing_selector(
    selectors: &mut Vec<SelectorEntry>,
    role: SelectorRole,
    store_root: &Path,
    incomplete: Option<OfflineIndeterminatePhysicalReason>,
    walk: &mut BoundedMediaWalk,
) {
    if selectors
        .iter()
        .any(|entry| entry.canonical && entry.role == role)
    {
        return;
    }
    let name = match role {
        SelectorRole::Current => CURRENT_SELECTOR,
        SelectorRole::Previous => PREVIOUS_SELECTOR,
    };
    let outcome = incomplete.map_or_else(
        || {
            walk.counters_mut().missing_artifacts += 1;
            OfflineIntegrityOutcome::Damaged(OfflinePhysicalDamageLocalization::new(
                OfflinePhysicalDamageCause::MissingArtifact,
                None,
                None,
                OfflinePhysicalBlastRadius::Artifact,
            ))
        },
        OfflineIntegrityOutcome::Indeterminate,
    );
    selectors.push(SelectorEntry {
        path: store_root.join(RECORDS_RELATIVE).join(name),
        relative: format!("{RECORDS_RELATIVE}/{name}"),
        role,
        canonical: true,
        facts: None,
        observed_identity: None,
        outcome,
        physical_alias_of: None,
        semantic_duplicate: false,
    });
}

pub(crate) fn selector_path_role(path: &Path) -> Option<(SelectorRole, bool)> {
    let name = path.file_name()?.to_str()?;
    match name {
        CURRENT_SELECTOR => Some((SelectorRole::Current, true)),
        PREVIOUS_SELECTOR => Some((SelectorRole::Previous, true)),
        _ if candidate_name(name, "root-current-") => Some((SelectorRole::Current, false)),
        _ if candidate_name(name, "root-previous-") => Some((SelectorRole::Previous, false)),
        _ => None,
    }
}

fn candidate_name(name: &str, prefix: &str) -> bool {
    name.strip_prefix(prefix)
        .and_then(|rest| rest.strip_suffix(".candidate"))
        .is_some_and(|hex| hex.len() == 16 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

pub(crate) fn root_manifest_generation(path: &Path) -> Option<u64> {
    let name = path.file_name()?.to_str()?;
    let hex = name.strip_prefix("root-")?.strip_suffix(".manifest")?;
    (hex.len() == 16)
        .then(|| u64::from_str_radix(hex, 16).ok())
        .flatten()
}

pub(crate) fn selector_order(entry: &SelectorEntry) -> (u8, bool, &Path) {
    (entry.role as u8, !entry.canonical, &entry.path)
}
