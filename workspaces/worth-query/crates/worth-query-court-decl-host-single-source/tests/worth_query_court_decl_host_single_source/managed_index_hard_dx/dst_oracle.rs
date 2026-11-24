//! Brute-force 4-family oracle and tight-budget refusal probe for Hard DX DST.
use super::operations::AssetSpec;
use super::schema::*;
use super::{install, keys};
use std::collections::BTreeMap;
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

#[derive(Default)]
pub struct FourFamilyOracle {
    pub rows: BTreeMap<u64, AssetSpec>,
}

impl FourFamilyOracle {
    pub fn prefix_matches(&self, prefix: &str) -> Vec<u64> {
        let needle = prefix.to_ascii_lowercase();
        self.rows
            .iter()
            .filter(|(_, s)| s.title.to_ascii_lowercase().starts_with(&needle))
            .map(|(&k, _)| k)
            .collect()
    }

    pub fn interval_matches(&self, start: i64, end: i64) -> Vec<u64> {
        self.rows
            .iter()
            .filter(|(_, s)| {
                s.start_min < s.end_min && start < end && s.start_min < end && start < s.end_min
            })
            .map(|(&k, _)| k)
            .collect()
    }

    pub fn box2d_matches(&self, p: QueryBox2D) -> Vec<u64> {
        self.rows
            .iter()
            .filter(|(_, s)| {
                s.min[0] < s.max[0]
                    && s.min[1] < s.max[1]
                    && p.min_x < p.max_x
                    && p.min_y < p.max_y
                    && s.min[0] < p.max_x
                    && p.min_x < s.max[0]
                    && s.min[1] < p.max_y
                    && p.min_y < s.max[1]
            })
            .map(|(&k, _)| k)
            .collect()
    }

    pub fn box3d_matches(&self, p: QueryBox3D) -> Vec<u64> {
        self.rows
            .iter()
            .filter(|(_, s)| {
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
            })
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

pub fn check_watch<Q>(
    name: &str,
    outcome: Result<
        WorthQueryManagedIndexWatchOutcome<Q, AssetRow>,
        WorthQueryApplicationRequestQueryDenial,
    >,
    expected: &[u64],
) -> Result<bool, String> {
    if let WorthQueryManagedIndexWatchOutcome::Delivered(u) =
        outcome.map_err(|e| format!("{name}: {e:?}"))?
    {
        if keys(u.read().rows()) != expected {
            return Err(format!("{name} watch mismatch"));
        }
        return Ok(true);
    }
    Ok(false)
}

pub fn verify_budget_refusal(oracle: &FourFamilyOracle) -> Result<(), String> {
    let seed: Vec<AssetSpec> = oracle.rows.values().take(1).cloned().collect();
    let expected: Vec<u64> = seed.iter().map(|s| s.key).collect();
    let tight = install(&seed, None, Some(500_000));
    let scope = request_scope();
    let principal = authenticate_operator(tight.installed_schema(), &scope);
    let req = tight.runtime().request(&principal, &scope);
    let qp = |p: &str| FindByTitlePrefix {
        floor: "floor-1".into(),
        prefix: p.to_owned(),
    };
    let idx = req
        .query(qp(""))
        .ensure_index_ready::<TitlePrefixIndex>()
        .map_err(|e| format!("tight ensure_index_ready failed: {e:?}"))?;
    let shared = "m".repeat(49_990);
    let mut refused = false;
    let mut ws = Vec::new();
    for i in 0..16u8 {
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
                            ) | WorthQueryManagedIndexStop::Request(_)
                        )
                    ) | WorthQueryManagedIndexDenialCause::QueryRead(_)
                ) =>
            {
                refused = true;
                break;
            }
            Err(other) => return Err(format!("unexpected budget refusal: {other:?}")),
        }
    }
    for w in ws {
        let _ = w.close();
    }
    let after = keys(
        req.query(qp(""))
            .execute()
            .map_err(|e| format!("post-refusal query: {e:?}"))?
            .rows(),
    );
    idx.close();
    if !refused {
        return Err("tight work ceiling failed to refuse".to_owned());
    }
    if after != expected {
        return Err(format!(
            "same-host post-refusal state {after:?} != {expected:?}"
        ));
    }
    Ok(())
}
