//! Existing source courts are reported; only new constructor evidence is refused.
use super::{
    failure,
    source_construction::{self, Construction},
};
use crate::{diagnostics::Diagnostic, source_rules::GovernedCrate};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub(super) const BASELINE: &str = "tools/boundary-check/config/src-court-baseline.json";
pub(super) type Baseline = BTreeMap<String, BTreeMap<String, BTreeSet<String>>>;

pub(super) fn grouped(rows: BTreeSet<Construction>) -> Baseline {
    let mut baseline = Baseline::new();
    for row in rows {
        baseline
            .entry(row.source)
            .or_default()
            .entry(row.test)
            .or_default()
            .insert(row.constructor);
    }
    baseline
}

pub(super) fn observed(
    root: &Path,
    owner: &GovernedCrate,
) -> Result<BTreeSet<Construction>, String> {
    source_construction::inventory(root, &owner.crate_root.join("src"))
}

pub(super) fn validate(root: &Path, owner: &GovernedCrate) -> Vec<Diagnostic> {
    let observations = match observed(root, owner) {
        Ok(value) => value,
        Err(error) => return vec![failure(&owner.relative_crate_root, error)],
    };
    let baseline: Baseline = match std::fs::read_to_string(root.join(BASELINE)) {
        Ok(text) => match serde_json::from_str(&text) {
            Ok(value) => value,
            Err(error) => return vec![failure(BASELINE, error.to_string())],
        },
        Err(error)
            if error.kind() == std::io::ErrorKind::NotFound && !root.join(".git").exists() =>
        {
            Baseline::new()
        }
        Err(error) => return vec![failure(BASELINE, error.to_string())],
    };
    let baseline: BTreeSet<Construction> = baseline
        .into_iter()
        .flat_map(|(source, tests)| {
            tests.into_iter().flat_map(move |(test, constructors)| {
                let source = source.clone();
                constructors
                    .into_iter()
                    .map(move |constructor| Construction {
                        source: source.clone(),
                        test: test.clone(),
                        constructor,
                    })
            })
        })
        .collect();
    let existing = observations.intersection(&baseline).count();
    if existing > 0 {
        println!("test-topology: {}: {existing} baselined source-test construction witnesses (report mode)", owner.package);
    }
    observations.difference(&baseline).map(|item| failure(&item.source, format!("new source court `{}` constructs `{}`; move to a court or audit private logic before adding a baseline exception", item.test, item.constructor))).collect()
}
