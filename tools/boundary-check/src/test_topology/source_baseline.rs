//! Compact per-test baseline of registered runtime-root construction witnesses.
use super::{
    failure,
    runtime_roots::Registry,
    source_construction::{self, Construction},
};
use crate::{diagnostics::Diagnostic, source_rules::GovernedCrate};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub(super) const BASELINE: &str = "tools/boundary-check/config/src-court-baseline.json";
pub(super) type Baseline = BTreeMap<(String, String), BTreeSet<String>>;
#[derive(Deserialize, Serialize)]
pub(super) struct Entry {
    source: String,
    test: String,
    constructors: BTreeSet<String>,
}
pub(super) fn grouped(rows: BTreeSet<Construction>) -> Vec<Entry> {
    let mut grouped = Baseline::new();
    for row in rows {
        grouped
            .entry((row.source, row.test))
            .or_default()
            .insert(row.constructor);
    }
    grouped
        .into_iter()
        .map(|((source, test), constructors)| Entry {
            source,
            test,
            constructors,
        })
        .collect()
}
pub(super) fn observed(
    root: &Path,
    owner: &GovernedCrate,
    roots: &Registry,
) -> Result<BTreeSet<Construction>, String> {
    source_construction::inventory(root, &owner.crate_root.join("src"), roots)
}
pub(super) fn load(root: &Path) -> Result<Baseline, String> {
    match std::fs::read_to_string(root.join(BASELINE)) {
        Ok(text) => {
            let entries: Vec<Entry> =
                serde_json::from_str(&text).map_err(|e| format!("{BASELINE}: {e}"))?;
            let mut known = Baseline::new();
            for entry in entries {
                if known
                    .insert((entry.source, entry.test), entry.constructors)
                    .is_some()
                {
                    return Err(format!("{BASELINE}: duplicate test entry"));
                }
            }
            Ok(known)
        }
        Err(error)
            if error.kind() == std::io::ErrorKind::NotFound && !root.join(".git").exists() =>
        {
            Ok(Baseline::new())
        }
        Err(error) => Err(format!("{BASELINE}: {error}")),
    }
}
pub(super) fn validate(
    root: &Path,
    owner: &GovernedCrate,
    baseline: &Baseline,
    roots: &Registry,
) -> Vec<Diagnostic> {
    let observations = match observed(root, owner, roots) {
        Ok(value) => value,
        Err(error) => return vec![failure(&owner.relative_crate_root, error)],
    };
    let known = |item: &Construction| {
        baseline
            .get(&(item.source.clone(), item.test.clone()))
            .is_some_and(|constructors| constructors.contains(&item.constructor))
    };
    let existing = observations
        .iter()
        .filter(|item| known(item))
        .map(|item| (&item.source, &item.test))
        .collect::<BTreeSet<_>>()
        .len();
    if existing > 0 {
        println!(
            "test-topology: {}: {existing} baselined runtime-root source tests (report mode)",
            owner.package
        );
    }
    observations.iter().filter(|item|!known(item)).map(|item|failure(&item.source,format!("new source court `{}` constructs `{}`; move to a court or audit private logic before adding a baseline exception",item.test,item.constructor))).collect()
}
