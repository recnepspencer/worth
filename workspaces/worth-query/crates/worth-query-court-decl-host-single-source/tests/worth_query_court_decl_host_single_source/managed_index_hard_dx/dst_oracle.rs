//! Brute-force 4-family oracle and watch outcome verifier for Hard DX DST.
use super::operations::AssetSpec;
use super::schema::*;
use std::collections::BTreeMap;
use worth_query_decl::facade::application_schema::CanonicalF64;
use worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial;
use worth_query_host::facade::primary_graph::WorthQueryManagedIndexWatchOutcome;

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

pub fn expected_row(s: &AssetSpec) -> AssetRow {
    AssetRow {
        key: s.key,
        title: s.title.clone(),
        start_min: s.start_min,
        end_min: s.end_min,
        min_x: s.min[0],
        min_y: s.min[1],
        min_z: s.min[2],
        max_x: s.max[0],
        max_y: s.max[1],
        max_z: s.max[2],
        fmin_x: CanonicalF64(s.fmin_bits[0]),
        fmin_y: CanonicalF64(s.fmin_bits[1]),
        fmax_x: CanonicalF64(s.fmax_bits[0]),
        fmax_y: CanonicalF64(s.fmax_bits[1]),
    }
}

#[derive(Default)]
pub struct FourFamilyOracle {
    pub rows: BTreeMap<u64, AssetSpec>,
}

impl FourFamilyOracle {
    pub fn prefix_rows(&self, prefix: &str) -> Vec<AssetRow> {
        self.rows
            .values()
            .filter(|s| row_matches_prefix(s, prefix))
            .map(expected_row)
            .collect()
    }

    pub fn interval_rows(&self, start: i64, end: i64) -> Vec<AssetRow> {
        self.rows
            .values()
            .filter(|s| row_matches_interval(s, start, end))
            .map(expected_row)
            .collect()
    }

    pub fn box2d_rows(&self, p: QueryBox2D) -> Vec<AssetRow> {
        self.rows
            .values()
            .filter(|s| row_matches_box2d(s, p))
            .map(expected_row)
            .collect()
    }

    pub fn box3d_rows(&self, p: QueryBox3D) -> Vec<AssetRow> {
        self.rows
            .values()
            .filter(|s| row_matches_box3d(s, p))
            .map(expected_row)
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
    expected: &[AssetRow],
    expect_delivery: bool,
) -> Result<bool, String> {
    match (
        expect_delivery,
        outcome.map_err(|e| format!("{name}: {e:?}"))?,
    ) {
        (true, WorthQueryManagedIndexWatchOutcome::Delivered(u)) => {
            let got = u.read().rows();
            if got != expected {
                return Err(format!(
                    "{name} watch projected row mismatch: got {got:?}, expected {expected:?}"
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
