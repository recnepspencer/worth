//! Cold delta-chain conformance trace helpers & Lean runner invocation for
//! `CowMapDeltaSeal` (`proofs/lean/persistent_naming`).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, OnceLock};

use worth_relational_contracts::internal_port::history::data::coordinates::BranchId;
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use worth_relational_storage::internal_port::storage::residency::chunked::cold_member_port::CollectedMembers;

use super::{fmt_opt, writes};
use crate::lineage::naming_pages::builder::{seal_pages_with_ops, PageChange};
use crate::lineage::naming_pages::store::ColdNamingStore;
use crate::lineage::naming_pages::tables::INDEX_NAME;
use crate::lineage::naming_pages::{BranchNamingRoot, CheckpointNamingState, NameIndexHeader};

fn active_index_hdr() -> NameIndexHeader {
    NameIndexHeader {
        earliest_commit: Some(1),
        head: Some(1),
        retention_floor: None,
        unavailable: false,
    }
}

pub(super) fn trace_cold_chain_case(
    layers_spec: &[(&str, bool, bool, Vec<(u64, Option<u64>)>)],
) -> String {
    let row_key = |k: u64| [vec![INDEX_NAME], k.to_be_bytes().to_vec()].concat();
    let mut sink = CollectedMembers::default();
    let mut roots: Vec<BranchNamingRoot> = layers_spec
        .iter()
        .enumerate()
        .map(|(idx, (name, active, is_delta, rows))| {
            let ops = rows
                .iter()
                .map(|&(k, v)| {
                    (
                        row_key(k),
                        v.map_or(PageChange::Tombstone, |x| {
                            PageChange::Put(x.to_be_bytes().to_vec())
                        }),
                    )
                })
                .collect();
            let pages = seal_pages_with_ops(None, ops, &mut sink).unwrap();
            let delta_base = (*is_delta && idx + 1 < layers_spec.len())
                .then(|| BranchId(layers_spec[idx + 1].0.into()));
            BranchNamingRoot {
                branch: BranchId((*name).into()),
                naming: None,
                index: active.then(active_index_hdr),
                groups: None,
                pages,
                delta_base,
            }
        })
        .collect();
    roots.sort_by(|a, b| a.branch.cmp(&b.branch));
    let state = Arc::new(CheckpointNamingState {
        branches: roots,
        group_log: Vec::new(),
        group_log_through: None,
        retired_forks: Vec::new(),
    });
    let store = ColdNamingStore::open(
        Arc::clone(&state),
        crate::lineage::naming_pages::tests::MemSource::from_sink(sink),
    );
    let leaf = state.root(&BranchId(layers_spec[0].0.into())).unwrap();
    let probes = 12u64;
    let gets: Vec<String> = (0..probes)
        .map(|k| {
            fmt_opt(
                store
                    .get_root(leaf, &row_key(k))
                    .unwrap()
                    .map(|b| u64::from_be_bytes(b.try_into().unwrap())),
            )
        })
        .collect();
    let mut scanned = Vec::new();
    store
        .scan_root(leaf, &[INDEX_NAME], |k, v| {
            let key = u64::from_be_bytes(k[1..].try_into().unwrap());
            let val = u64::from_be_bytes(v.try_into().unwrap());
            scanned.push(format!("[{key},{val}]"));
            Ok(true)
        })
        .unwrap();
    let layers_json: Vec<String> = layers_spec
        .iter()
        .map(|(_, active, is_delta, rows)| {
            format!(
                r#"{{"active_domain":{active},"is_delta":{is_delta},"rows":{}}}"#,
                writes(rows)
            )
        })
        .collect();
    format!(
        r#"{{"kind":"cold_chain","layers":[{}],"probes":{probes},"gets":[{}],"scan":[{}]}}"#,
        layers_json.join(","),
        gets.join(","),
        scanned.join(","),
    )
}

pub(super) fn assert_nondelta_tombstone_refuses() {
    let k = [vec![INDEX_NAME], 1u64.to_be_bytes().to_vec()].concat();
    let mut sink = CollectedMembers::default();
    let pages =
        seal_pages_with_ops(None, vec![(k.clone(), PageChange::Tombstone)], &mut sink).unwrap();
    let state = Arc::new(CheckpointNamingState {
        branches: vec![BranchNamingRoot {
            branch: BranchId("bad-base".into()),
            naming: None,
            index: Some(active_index_hdr()),
            groups: None,
            pages,
            delta_base: None,
        }],
        group_log: Vec::new(),
        group_log_through: None,
        retired_forks: Vec::new(),
    });
    let store = ColdNamingStore::open(
        Arc::clone(&state),
        crate::lineage::naming_pages::tests::MemSource::from_sink(sink),
    );
    let r = state.root(&BranchId("bad-base".into())).unwrap();
    assert_eq!(store.get_root(r, &k).unwrap_err(), Cause::Damaged);
}

fn find_lake() -> Option<PathBuf> {
    let exe = if cfg!(windows) { "lake.exe" } else { "lake" };
    if let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) {
        let cand = PathBuf::from(home).join(".elan").join("bin").join(exe);
        if cand.is_file() {
            return Some(cand);
        }
    }
    std::env::split_paths(&std::env::var_os("PATH")?)
        .map(|d| d.join(exe))
        .find(|p| p.is_file())
}

pub(super) fn conformance_bin() -> Option<&'static PathBuf> {
    static BIN: OnceLock<Option<PathBuf>> = OnceLock::new();
    BIN.get_or_init(|| {
        let required = std::env::var("WORTH_REQUIRE_LEAN_CONFORMANCE")
            .ok()
            .is_some_and(|v| !v.is_empty() && v != "0");
        let Some(lake) = find_lake() else {
            assert!(!required, "WORTH_REQUIRE_LEAN_CONFORMANCE set without lake");
            return None;
        };
        let lean_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .join("proofs/lean/persistent_naming");
        let exe = if cfg!(windows) {
            "conformance.exe"
        } else {
            "conformance"
        };
        let out = Command::new(&lake)
            .current_dir(&lean_dir)
            .args(["build", "conformance"])
            .output()
            .expect("invoke `lake build conformance`");
        assert!(
            out.status.success(),
            "lake build conformance failed:\n{}\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        Some(lean_dir.join(".lake/build/bin").join(exe))
    })
    .as_ref()
}
