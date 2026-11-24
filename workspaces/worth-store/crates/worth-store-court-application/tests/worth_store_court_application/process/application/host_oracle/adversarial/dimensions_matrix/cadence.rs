//! Cadence journal coverage assertions for C4b.3/C4b.4 D14 discard and retention courts.
use super::SeededCoverage;
use std::collections::BTreeMap;

pub(in super::super) const EXPECTED_CADENCE_KEYS: [&str; 5] = [
    "relational.branch-transitions",
    "relational.canonical-commits",
    "world.branches",
    "world.custody",
    "world.occurrences",
];

fn assert_cadence_roster(journals: &BTreeMap<String, (u64, u64)>, label: &str) {
    let keys: Vec<&str> = journals.keys().map(String::as_str).collect();
    assert_eq!(keys, EXPECTED_CADENCE_KEYS, "{label}: cadence roster");
}

pub(in super::super) fn assert_cadence_discarded(
    journals: &BTreeMap<String, (u64, u64)>,
    label: &str,
) {
    assert_cadence_roster(journals, label);
    assert!(
        journals.values().all(|&(covered, _)| covered == 0),
        "{label}: D14 discard requires covered==0 across all 5 journals: {journals:?}"
    );
    for key in [
        "relational.canonical-commits",
        "world.branches",
        "world.occurrences",
    ] {
        assert!(
            journals[key].1 > 0,
            "{label}: authoritative head for {key} must stay > 0"
        );
    }
}

pub(in super::super) fn assert_cadence_retained(
    journals: &BTreeMap<String, (u64, u64)>,
    seeded: SeededCoverage,
    label: &str,
) {
    assert_cadence_roster(journals, label);
    let (native_cov, native_head) = journals["relational.canonical-commits"];
    let (world_cov, world_head) = journals["world.occurrences"];
    assert!(
        native_cov >= seeded.native_covered && native_cov > 0 && native_head >= native_cov,
        "{label}: retained Native checkpoint lost (seeded={}, got {native_cov}/{native_head})",
        seeded.native_covered
    );
    assert!(
        world_cov >= seeded.world_covered && world_cov > 0 && world_head >= world_cov,
        "{label}: retained World checkpoint lost (seeded={}, got {world_cov}/{world_head})",
        seeded.world_covered
    );
}

pub(in super::super) fn assert_cadence_checkpointed(
    journals: &BTreeMap<String, (u64, u64)>,
    label: &str,
) {
    assert_cadence_roster(journals, label);
    for key in ["relational.canonical-commits", "world.occurrences"] {
        let (covered, head) = journals[key];
        assert!(
            covered > 0 && covered == head,
            "{label}: post-close checkpoint must cover {key} ({covered}/{head})"
        );
    }
}
