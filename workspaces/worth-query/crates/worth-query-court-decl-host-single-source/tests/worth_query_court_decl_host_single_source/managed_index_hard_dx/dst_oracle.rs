//! Brute-force 4-family oracle and tight-budget refusal probe for Hard DX DST.
use super::operations::AssetSpec;
use super::schema::*;
use super::{create_asset, install, keys};
use std::collections::BTreeMap;
use worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial;
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::primary_graph::*;
use worth_query_host::facade::runtime::MapKernelStop;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

pub const TITLES: &[&str] = &[
    "CNC-Alpha",
    "cnc-beta",
    "Lathe-01",
    "lathe-02",
    "Press-A",
    "Route-Main",
    "route-aux",
    "élan-bay",
    "Éclair-bay",
];
pub const PREFIXES: &[&str] = &["", "c", "CNC", "l", "lathe", "p", "r", "route", "z", "é"];

pub fn row_matches_prefix(s: &AssetSpec, prefix: &str) -> bool {
    s.title
        .to_ascii_lowercase()
        .starts_with(&prefix.to_ascii_lowercase())
}

pub fn row_matches_interval(s: &AssetSpec, start: i64, end: i64) -> bool {
    s.start_min < s.end_min && start < end && s.start_min < end && start < s.end_min
}

pub fn row_matches_box2d(s: &AssetSpec, p: QueryBox2D) -> bool {
    s.min[0] < s.max[0]
        && s.min[1] < s.max[1]
        && p.min_x < p.max_x
        && p.min_y < p.max_y
        && s.min[0] < p.max_x
        && p.min_x < s.max[0]
        && s.min[1] < p.max_y
        && p.min_y < s.max[1]
}

pub fn row_matches_box3d(s: &AssetSpec, p: QueryBox3D) -> bool {
    (0..3).all(|i| s.min[i] <= s.max[i])
        && p.min_x <= p.max_x
        && p.min_y <= p.max_y
        && p.min_z <= p.max_z
        && s.min[0] <= p.max_x
        && p.min_x <= s.max[0]
        && s.min[1] <= p.max_y
        && p.min_y <= s.max[1]
        && s.min[2] <= p.max_z
        && p.min_z <= s.max[2]
}

#[derive(Default)]
pub struct FourFamilyOracle {
    pub rows: BTreeMap<u64, AssetSpec>,
}

impl FourFamilyOracle {
    pub fn prefix_matches(&self, prefix: &str) -> Vec<u64> {
        self.rows
            .iter()
            .filter(|(_, s)| row_matches_prefix(s, prefix))
            .map(|(&k, _)| k)
            .collect()
    }

    pub fn interval_matches(&self, start: i64, end: i64) -> Vec<u64> {
        self.rows
            .iter()
            .filter(|(_, s)| row_matches_interval(s, start, end))
            .map(|(&k, _)| k)
            .collect()
    }

    pub fn box2d_matches(&self, p: QueryBox2D) -> Vec<u64> {
        self.rows
            .iter()
            .filter(|(_, s)| row_matches_box2d(s, p))
            .map(|(&k, _)| k)
            .collect()
    }

    pub fn box3d_matches(&self, p: QueryBox3D) -> Vec<u64> {
        self.rows
            .iter()
            .filter(|(_, s)| row_matches_box3d(s, p))
            .map(|(&k, _)| k)
            .collect()
    }
}

pub const UNIT_B2: QueryBox2D = QueryBox2D {
    min_x: 0,
    min_y: 0,
    max_x: 1,
    max_y: 1,
};
pub const UNIT_B3: QueryBox3D = QueryBox3D {
    min_x: 0,
    min_y: 0,
    min_z: 0,
    max_x: 1,
    max_y: 1,
    max_z: 1,
};

pub fn qp(p: &str) -> FindByTitlePrefix {
    FindByTitlePrefix {
        floor: "floor-1".into(),
        prefix: p.to_owned(),
    }
}
pub fn qi(w: FocusWindow) -> FindByFocusWindow {
    FindByFocusWindow {
        floor: "floor-1".into(),
        window: w,
    }
}
pub fn q2(b: QueryBox2D) -> FindEquipmentInBox2D {
    FindEquipmentInBox2D {
        floor: "floor-1".into(),
        region: b,
    }
}
pub fn q3(b: QueryBox3D) -> FindRoutesInBox3D {
    FindRoutesInBox3D {
        floor: "floor-1".into(),
        region: b,
    }
}

fn outcome_label<Q>(outcome: &WorthQueryManagedIndexWatchOutcome<Q, AssetRow>) -> &'static str {
    match outcome {
        WorthQueryManagedIndexWatchOutcome::Delivered(_) => "Delivered",
        WorthQueryManagedIndexWatchOutcome::Pending => "Pending",
        WorthQueryManagedIndexWatchOutcome::Closed => "Closed",
        WorthQueryManagedIndexWatchOutcome::Overflow { .. } => "Overflow",
        WorthQueryManagedIndexWatchOutcome::Unavailable => "Unavailable",
    }
}

pub fn check_watch<Q>(
    name: &str,
    outcome: Result<
        WorthQueryManagedIndexWatchOutcome<Q, AssetRow>,
        WorthQueryApplicationRequestQueryDenial,
    >,
    expected: &[u64],
    expect_delivery: bool,
) -> Result<bool, String> {
    match (
        expect_delivery,
        outcome.map_err(|e| format!("{name}: {e:?}"))?,
    ) {
        (true, WorthQueryManagedIndexWatchOutcome::Delivered(u)) => {
            let got = keys(u.read().rows());
            if got != expected {
                return Err(format!(
                    "{name} watch mismatch: got {got:?}, expected {expected:?}"
                ));
            }
            Ok(true)
        }
        (true, other) => Err(format!(
            "{name} expected Delivered (matching rows {expected:?}), got {}",
            outcome_label(&other)
        )),
        (false, WorthQueryManagedIndexWatchOutcome::Pending) => Ok(false),
        (false, other) => Err(format!(
            "{name} expected Pending on unrelated mutation, got {}",
            outcome_label(&other)
        )),
    }
}

pub fn verify_budget_refusal(oracle: &FourFamilyOracle) -> Result<(), String> {
    let seed: Vec<AssetSpec> = oracle.rows.values().take(1).cloned().collect();
    let expected: Vec<u64> = seed.iter().map(|s| s.key).collect();
    let tight = install(&seed, None, Some(850_000));
    let scope = request_scope();
    let principal = authenticate_operator(tight.installed_schema(), &scope);
    let req = tight.runtime().request(&principal, &scope);
    let idx = req
        .query(qp(""))
        .ensure_index_ready::<TitlePrefixIndex>()
        .map_err(|e| format!("tight ensure_index_ready failed: {e:?}"))?;

    // Open a baseline watch before triggering the budget-refused watch open so we
    // can prove zero leaked watch registrations and intact routing on the same host.
    let (_init_w, mut existing_watch) = req
        .query(qp("WatchHit-"))
        .watch_index::<TitlePrefixIndex>()
        .map_err(|e| format!("baseline watch open failed: {e:?}"))?;

    let shared = "m".repeat(49_990);
    let mut refused = false;
    let mut ws = Vec::new();
    for i in 0..64u8 {
        let prefix = format!("{shared}{i:02}");
        match req.query(qp(&prefix)).watch_index::<TitlePrefixIndex>() {
            Ok((_, w)) => ws.push(w),
            Err(WorthQueryApplicationRequestQueryDenial::ManagedIndex(cause))
                if matches!(
                    cause.cause(),
                    WorthQueryManagedIndexDenialCause::Work(
                        WorthQueryManagedIndexWorkDenial::Controlled(
                            WorthQueryManagedIndexStop::ValueDecode(
                                WorthQueryValueDecodeDenial::Checkpoint(MapKernelStop::WorkCeiling)
                            )
                        )
                    )
                ) || matches!(
                    cause.cause(),
                    WorthQueryManagedIndexDenialCause::QueryRead(read)
                        if matches!(
                            read.kind(),
                            WorthQueryApplicationOneShotDenialKind::ExecutionRequest(
                                WorthQueryProviderSessionDenialKind::ExecutionResource {
                                    denial: WorthQueryManagedComputationResourceDenial::WorkExhausted,
                                    ..
                                }
                            )
                        )
                ) =>
            {
                refused = true;
                break;
            }
            Err(other) => {
                return Err(format!(
                    "expected exact WorkCeiling refusal, got Err({other:?})"
                ));
            }
        }
    }
    for w in ws {
        let _ = w.close();
    }
    if !refused {
        return Err("tight work ceiling failed to refuse".to_owned());
    }

    // Same-host post-refusal parity: one-shot rows AND existing watch routing both stay intact.
    let after = keys(
        req.query(qp(""))
            .execute()
            .map_err(|e| format!("post-refusal query: {e:?}"))?
            .rows(),
    );
    if after != expected {
        return Err(format!(
            "same-host post-refusal state {after:?} != {expected:?}"
        ));
    }
    // Unrelated mutation must NOT wake `existing_watch`.
    create_asset(
        &tight,
        AssetSpec::i64_only(901, "Unrelated-After-Refusal", (0, 5), [0, 0, 0], [1, 1, 1]),
        901,
    );
    check_watch(
        "post-refusal-unrelated",
        req.query(qp("WatchHit-")).next_watch(&mut existing_watch),
        &[],
        false,
    )?;
    // Matching mutation MUST wake `existing_watch` with exact rows.
    create_asset(
        &tight,
        AssetSpec::i64_only(902, "WatchHit-After-Refusal", (0, 5), [0, 0, 0], [1, 1, 1]),
        902,
    );
    check_watch(
        "post-refusal-matching",
        req.query(qp("WatchHit-")).next_watch(&mut existing_watch),
        &[902],
        true,
    )?;
    let _ = existing_watch.close();
    idx.close();
    Ok(())
}
