//! Charging cannot change occurrence selection, ordering, or stale ownership.
use super::super::record_map::DemandRecords;
use super::super::refreshed_rejoin::*;
use super::*;
use crate::domain_computation::primary_graph::application_contribution::{
    WorthQueryProducerApplicability, WorthQueryProducerLifecyclePosture,
};
use std::cmp::Ordering;

#[test]
fn seeded_charged_and_uncharged_occurrence_searches_agree() {
    let mut seed = 0x8a31_774b_29d6_c025_u64;
    for run in 0..16 {
        let records = occurrence_run(&mut seed);
        for requested in records.iter().map(|(key, _)| key) {
            let plain: Vec<_> = occurrence_rows(&records, requested).collect();
            let mut admission = record_admission();
            let charged: Vec<_> = search_occurrence_rows(&records, requested, &mut admission)
                .unwrap()
                .collect();
            assert_eq!(
                plain.iter().map(|(key, _)| key).collect::<Vec<_>>(),
                charged.iter().map(|(key, _)| key).collect::<Vec<_>>(),
                "occurrence rows, run {run}, {requested:?}"
            );
            assert!(plain.iter().all(|(key, _)| key.same_occurrence(requested)));
            assert!(!plain.is_empty());

            let mut admission = record_admission();
            assert_eq!(
                newest_lawful_successor(plain.iter().map(|(key, _)| *key), requested).cloned(),
                newest_matching_row(&records, requested, &mut admission, |key, _| {
                    refresh_order(key, requested) == Some(Ordering::Greater)
                })
                .unwrap(),
                "newest successor, run {run}, {requested:?}"
            );
            let mut admission = record_admission();
            assert_eq!(
                awaited_by_stale_owner(&records, requested),
                search_stale_owner(&records, requested, &mut admission).unwrap(),
                "stale owner, run {run}, {requested:?}"
            );
        }
    }
}

#[test]
fn charged_occurrence_search_pins_path_and_row_work() {
    let keys = [
        key("charged", 1, 2),
        key("charged", 2, 2),
        key("charged", 3, 2),
    ];
    let mut records = DemandRecords::new();
    for key in &keys {
        records.insert(key.clone(), record(occurrence(), DemandState::Admitted, 0));
    }
    for root in [1, 3] {
        records.insert(
            key("neighbor", 1, root),
            record(occurrence(), DemandState::Admitted, 0),
        );
    }
    let mut admission = record_admission();
    assert_eq!(
        search_occurrence_rows(&records, &keys[0], &mut admission)
            .unwrap()
            .count(),
        3
    );
    // Five records: bit length 3, two paths of (3 * 11 + 1) = 68 navigation.
    assert_eq!(admission.charged_navigation(), 68);
    assert_eq!(admission.charged_work(), 2);

    let mut admission = record_admission();
    assert_eq!(
        newest_matching_row(&records, &keys[0], &mut admission, |key, _| {
            refresh_order(key, &keys[0]) == Some(Ordering::Greater)
        })
        .unwrap(),
        Some(keys[2].clone())
    );
    // Two seeks plus three inspected rows, including the rejected predecessor.
    assert_eq!(admission.charged_work(), 5);
    assert_eq!(admission.charged_navigation(), 68);

    let mut admission = record_admission();
    assert!(!search_stale_owner(&records, &keys[2], &mut admission).unwrap());
    assert_eq!(admission.charged_work(), 5);
    assert_eq!(admission.charged_navigation(), 68);

    for (budget, tested_rows) in [(1, 0), (4, 2)] {
        let mut admission = InvalidationEditAdmission::new(CompanionPreflightBudget {
            maximum_work_visits: budget,
            maximum_preparation_bytes: 8 * 1024 * 1024,
        });
        let tested = std::cell::Cell::new(0);
        let refusal = newest_matching_row(&records, &keys[0], &mut admission, |_, _| {
            tested.set(tested.get() + 1);
            false
        })
        .expect_err("the path and each inspected row must be paid before accepts");
        assert_eq!(
            refusal.kind(),
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
        );
        assert_eq!(tested.get(), tested_rows);
    }
}

fn occurrence_run(seed: &mut u64) -> DemandRecords {
    let mut rows = Vec::new();
    let generation = 1 + next(seed) % 100;
    for offset in 0..6 {
        let mut key = key_with_identity("initial", generation + offset, 2, 90 + offset);
        key.applicability = applicability(WorthQueryProducerLifecyclePosture::Initial);
        let state = match offset {
            0..=2 => DemandState::Failed(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::Superseded,
                "seeded superseded row",
            )),
            3 => closed_retirement::ready(),
            4 => DemandState::Running,
            _ => DemandState::Admitted,
        };
        let mut row = record(occurrence(), state, usize::from(offset == 0));
        row.framework_required_count = usize::from(offset == 2);
        rows.push((key, row));
    }
    for producer in ["left-preserve", "right-preserve"] {
        let key = key_with_identity(producer, generation + 6, 2, 96);
        rows.push((key, record(occurrence(), DemandState::Admitted, 0)));
    }
    assert_eq!(refresh_order(&rows[6].0, &rows[7].0), None);
    assert!(rows[4].1.unpublished_new_key());
    assert!(!rows[3].1.unpublished_new_key());
    for root in [1, 3] {
        for offset in 0..2 {
            rows.push((
                key("neighbor", generation + offset, root),
                record(occurrence(), DemandState::Admitted, 0),
            ));
        }
    }
    // Fixed seed varies insertion order without making failures unreproducible.
    for index in (1..rows.len()).rev() {
        let other = (next(seed) % (index as u64 + 1)) as usize;
        rows.swap(index, other);
    }
    let mut records = DemandRecords::new();
    records.extend(rows);
    records
}

fn applicability(posture: WorthQueryProducerLifecyclePosture) -> WorthQueryProducerApplicability {
    WorthQueryProducerApplicability::new("registry-fixture", posture)
}

fn next(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}
