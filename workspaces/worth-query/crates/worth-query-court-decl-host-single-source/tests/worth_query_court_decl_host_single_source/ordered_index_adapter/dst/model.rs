//! Brute-force sorted-scan oracle and bounded-budget refusal probe for ordered-index DST.
use super::super::queries::FindTitlePrefix;
use super::super::schema::TitlePrefixIndex;
use std::collections::{BTreeMap, BTreeSet};
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::primary_graph::*;
use worth_query_host::facade::runtime::MapKernelStop;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

pub const MAX_ROWS: u64 = 8;
pub const MAX_COORD: i64 = 16;

pub const TITLES: &[&str] = &[
    "Alpha",
    "alpine",
    "ALDER",
    "Beta",
    "beta-plan",
    "Gamma",
    "delta",
    "Delta-Review",
    "Focus",
    "focus-deep",
    "élan",
    "Éclair",
];

pub const PREFIXES: &[&str] = &[
    "", "a", "al", "AL", "alp", "b", "be", "d", "del", "f", "foc", "g", "z", "é",
];

#[derive(Clone, Debug)]
pub struct OracleTask {
    pub title: String,
    pub lo: i64,
    pub hi: i64,
}

#[derive(Default)]
pub struct SortedScanOracle {
    pub rows: BTreeMap<u64, OracleTask>,
}

impl SortedScanOracle {
    pub fn live_ids(&self) -> BTreeSet<u64> {
        self.rows.keys().copied().collect()
    }

    pub fn prefix_matches(&self, prefix: &str) -> Vec<u64> {
        let needle = prefix.to_ascii_lowercase();
        self.rows
            .iter()
            .filter(|(_, t)| t.title.to_ascii_lowercase().starts_with(&needle))
            .map(|(&k, _)| k)
            .collect()
    }

    pub fn focus_matches(&self, lo: i64, hi: i64) -> Vec<u64> {
        self.rows
            .iter()
            .filter(|(_, t)| t.lo <= hi && lo <= t.hi)
            .map(|(&k, _)| k)
            .collect()
    }
}

pub fn verify_tight_budget_refusal(oracle: &SortedScanOracle) -> Result<(), String> {
    let tasks: Vec<(u64, &str, i64, i64)> = oracle
        .rows
        .iter()
        .take(2)
        .map(|(&k, t)| (k, t.title.as_str(), t.lo, t.hi + 1))
        .collect();
    let expected: Vec<u64> = tasks.iter().map(|&(k, ..)| k).collect();
    let tight = super::super::install(&tasks, None, Some(500_000));
    let scope = request_scope();
    let principal = authenticate_operator(tight.installed_schema(), &scope);
    let req = tight.runtime().request(&principal, &scope);
    let idx = req
        .query(FindTitlePrefix {
            workspace: "main".into(),
            prefix: String::new(),
        })
        .open_index::<TitlePrefixIndex>()
        .map_err(|e| format!("tight open: {e:?}"))?;
    let mut refused = req
        .query(FindTitlePrefix {
            workspace: "main".into(),
            prefix: String::new(),
        })
        .rebuild_index(&idx)
        .is_err();
    if !refused {
        let shared = "m".repeat(49_999);
        let mut ws = Vec::new();
        for ch in ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'] {
            let mut prefix = shared.clone();
            prefix.push(ch);
            let q = FindTitlePrefix {
                workspace: "main".into(),
                prefix,
            };
            match req.query(q).watch_index::<TitlePrefixIndex>() {
                Ok((_, w)) => ws.push(w),
                Err(WorthQueryApplicationRequestQueryDenial::ManagedIndex(cause))
                    if matches!(
                        cause.cause(),
                        WorthQueryManagedIndexDenialCause::Work(
                            WorthQueryManagedIndexWorkDenial::Controlled(
                                WorthQueryManagedIndexStop::ValueDecode(
                                    WorthQueryValueDecodeDenial::Checkpoint(
                                        MapKernelStop::WorkCeiling
                                    )
                                ) | WorthQueryManagedIndexStop::Request(_)
                            )
                        ) | WorthQueryManagedIndexDenialCause::QueryRead(_)
                    ) =>
                {
                    refused = true;
                    break;
                }
                Err(other) => return Err(format!("unexpected tight denial: {other:?}")),
            }
        }
        for w in ws {
            let _ = w.close();
        }
        let after = super::super::keys(
            req.query(FindTitlePrefix {
                workspace: "main".into(),
                prefix: String::new(),
            })
            .execute()
            .map_err(|e| format!("post-refusal query err: {e:?}"))?
            .rows(),
        );
        if after != expected {
            return Err(format!(
                "post-refusal rows {after:?} != expected {expected:?}"
            ));
        }
    }
    idx.close();
    if !refused {
        return Err("tight work ceiling failed to refuse".to_owned());
    }
    Ok(())
}
