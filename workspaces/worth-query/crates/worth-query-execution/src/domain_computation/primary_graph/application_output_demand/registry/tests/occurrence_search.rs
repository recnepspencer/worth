//! Charging cannot change occurrence selection, ordering, or stale ownership.
use super::super::record_map::DemandRecords;
use super::super::refreshed_rejoin::infallible;
use super::super::refreshed_rejoin::*;
use super::*;
use crate::domain_computation::primary_graph::application_contribution::{
    WorthQueryProducerApplicability, WorthQueryProducerLifecyclePosture,
};
use std::cmp::Ordering;

#[test]
fn seeded_charged_and_uncharged_occurrence_searches_agree() {
    let mut seed = 0x8a31_774b_29d6_c025_u64;
    let mut visits = ArmVisits::default();
    for run in 0..16 {
        let records = occurrence_run(&mut seed);
        for requested in records.iter().map(|(key, _)| key) {
            let plain: Vec<_> = infallible(occurrence_rows(
                &records,
                requested,
                &mut OracleWalk::after_effect(),
            ))
            .collect();
            let mut admission = record_admission();
            let charged: Vec<_> = occurrence_rows(&records, requested, &mut admission)
                .unwrap()
                .collect();
            assert_eq!(
                plain.iter().map(|(key, _)| key).collect::<Vec<_>>(),
                charged.iter().map(|(key, _)| key).collect::<Vec<_>>(),
                "occurrence rows, run {run}, {requested:?}"
            );
            assert!(plain.iter().all(|(key, _)| key.same_occurrence(requested)));
            assert!(!plain.is_empty());

            let mut candidates: Vec<_> = plain.iter().map(|(key, _)| *key).collect();
            for index in (1..candidates.len()).rev() {
                let other = (next(&mut seed) % (index as u64 + 1)) as usize;
                candidates.swap(index, other);
            }
            let eligible: Vec<_> = candidates
                .iter()
                .copied()
                .filter(|candidate| refresh_order(candidate, requested) == Some(Ordering::Greater))
                .collect();
            if !eligible.is_empty() && next(&mut seed) & 1 != 0 {
                let repeated = eligible[(next(&mut seed) % eligible.len() as u64) as usize];
                candidates.splice(0..0, [repeated, repeated]);
            }
            assert_eq!(
                infallible(newest_lawful_successor(
                    candidates.iter().copied(),
                    requested,
                    &mut OracleWalk::after_effect()
                )),
                tally_fold(&candidates, requested, &mut visits),
                "generated key walk, run {run}, {requested:?}"
            );

            let mut admission = record_admission();
            assert_eq!(
                infallible(newest_lawful_successor(
                    plain.iter().map(|(key, _)| *key),
                    requested,
                    &mut OracleWalk::after_effect()
                ))
                .cloned(),
                newest_matching_row(&records, requested, &mut admission, |key, _| {
                    refresh_order(key, requested) == Some(Ordering::Greater)
                })
                .unwrap(),
                "newest successor, run {run}, {requested:?}"
            );
            let mut admission = record_admission();
            assert_eq!(
                infallible(awaited_by_stale_owner(
                    &records,
                    requested,
                    &mut OracleWalk::after_effect()
                )),
                awaited_by_stale_owner(&records, requested, &mut admission).unwrap(),
                "stale owner, run {run}, {requested:?}"
            );
            assert_eq!(
                infallible(awaited_by_stale_owner(
                    &records,
                    requested,
                    &mut OracleWalk::after_effect()
                )),
                tally_stale_owner(&plain, requested, &mut visits)
            );
        }
    }
    visits.assert_reached();
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
        occurrence_rows(&records, &keys[0], &mut admission)
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
    assert!(!awaited_by_stale_owner(&records, &keys[2], &mut admission).unwrap());
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

#[test]
fn charged_stale_owner_exit_pays_through_the_published_newer_row_only() {
    let mut records = DemandRecords::new();
    for generation in 1..=4 {
        let state = match generation {
            1 => DemandState::Failed(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::Superseded,
                "older owner",
            )),
            3 => closed_retirement::ready(),
            _ => DemandState::Admitted,
        };
        records.insert(
            key("exit", generation, 2),
            record(occurrence(), state, usize::from(generation == 1)),
        );
    }
    records.insert(
        key("neighbor", 1, 1),
        record(occurrence(), DemandState::Admitted, 0),
    );
    let mut admission = InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: 5,
        maximum_preparation_bytes: 8 * 1024 * 1024,
    });
    assert!(!awaited_by_stale_owner(&records, &key("exit", 2, 2), &mut admission).unwrap());
    // Two seeks + older row + self + published newer row = 5. Generation 4 is not inspected.
    assert_eq!(admission.charged_work(), 5);
    // Five records have bit length 3: two paths of (3 * 11 + 1) = 68.
    assert_eq!(admission.charged_navigation(), 68);
}

fn occurrence_run(seed: &mut u64) -> DemandRecords {
    let mut rows = Vec::new();
    let generation = 1 + next(seed) % 100;
    for offset in 0..8 {
        if offset != 0 && offset != 7 && next(seed) & 1 == 0 {
            continue;
        }
        let mut key = key_with_identity("initial", generation + offset, 2, 90 + offset);
        key.applicability = applicability(WorthQueryProducerLifecyclePosture::Initial);
        let state = match next(seed) % 4 {
            0 => DemandState::Failed(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::Superseded,
                "seeded superseded row",
            )),
            1 => closed_retirement::ready(),
            2 => DemandState::Running,
            _ => DemandState::Admitted,
        };
        let mut row = record(occurrence(), state, (next(seed) & 1) as usize);
        row.framework_required_count = (next(seed) & 1) as usize;
        rows.push((key, row));
    }
    if next(seed) & 1 != 0 {
        let offset = next(seed) % 8;
        for producer in ["left-preserve", "right-preserve"] {
            let key = key_with_identity(producer, generation + offset, 2, 90 + offset);
            rows.push((
                key,
                record(
                    occurrence(),
                    DemandState::Admitted,
                    (next(seed) & 1) as usize,
                ),
            ));
        }
    }
    for root in [1, 3] {
        for offset in 0..2 {
            rows.push((
                key("neighbor", generation + offset, root),
                record(occurrence(), DemandState::Admitted, 0),
            ));
        }
    }
    let mut records = DemandRecords::new();
    records.extend(rows);
    records
}

#[derive(Debug, Default)]
struct ArmVisits {
    first: usize,
    greater: usize,
    less: usize,
    equal: usize,
    incomparable: usize,
    rejected: usize,
    self_skip: usize,
    published_exit: usize,
    finished_true: usize,
    finished_false: usize,
    interest_only: usize,
    framework_only: usize,
    unheld_superseded: usize,
    other_stale_row: usize,
}

impl ArmVisits {
    fn assert_reached(&self) {
        for (name, count) in [
            ("first", self.first),
            ("greater", self.greater),
            ("less", self.less),
            ("equal", self.equal),
            ("incomparable", self.incomparable),
            ("rejected", self.rejected),
            ("self skip", self.self_skip),
            ("published exit", self.published_exit),
            ("finished true", self.finished_true),
            ("finished false", self.finished_false),
            ("interest alone", self.interest_only),
            ("framework alone", self.framework_only),
            ("unheld superseded", self.unheld_superseded),
            ("other stale row", self.other_stale_row),
        ] {
            assert!(count > 0, "unreached {name}: {self:?}");
        }
    }
}

fn tally_fold<'a>(
    candidates: &[&'a WorthQueryOutputDemandKey],
    predecessor: &WorthQueryOutputDemandKey,
    visits: &mut ArmVisits,
) -> Option<&'a WorthQueryOutputDemandKey> {
    let mut newest = None;
    for &candidate in candidates {
        if refresh_order(candidate, predecessor) != Some(Ordering::Greater) {
            visits.rejected += 1;
            continue;
        }
        match newest {
            None => {
                visits.first += 1;
                newest = Some(candidate);
            }
            Some(current) => match refresh_order(candidate, current) {
                Some(Ordering::Greater) => {
                    visits.greater += 1;
                    newest = Some(candidate);
                }
                Some(Ordering::Less) => visits.less += 1,
                Some(Ordering::Equal) if candidate == current => visits.equal += 1,
                _ => {
                    visits.incomparable += 1;
                    return None;
                }
            },
        }
    }
    newest
}

fn tally_stale_owner(
    rows: &[(&WorthQueryOutputDemandKey, &DemandRecord)],
    key: &WorthQueryOutputDemandKey,
    visits: &mut ArmVisits,
) -> bool {
    let mut awaited = false;
    for &(other, record) in rows {
        if other == key {
            visits.self_skip += 1;
            continue;
        }
        match refresh_order(other, key) {
            Some(Ordering::Greater) if !record.unpublished_new_key() => {
                visits.published_exit += 1;
                return false;
            }
            Some(Ordering::Less) => {
                if superseded(record) {
                    match (record.interests != 0, record.framework_required_count != 0) {
                        (true, false) => visits.interest_only += 1,
                        (false, true) => visits.framework_only += 1,
                        (false, false) => visits.unheld_superseded += 1,
                        _ => {}
                    }
                    awaited |= record.interests != 0 || record.framework_required_count != 0;
                }
            }
            _ => visits.other_stale_row += 1,
        }
    }
    if awaited {
        visits.finished_true += 1;
    } else {
        visits.finished_false += 1;
    }
    awaited
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
