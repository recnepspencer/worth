//! Fresh processes preserve binding inputs; every row is accounted for.
use super::cases::{self, Cell};
use std::collections::BTreeMap;
mod cell_process;
mod execution;
mod price;
use price::cell;
fn is_runnable(row: &Cell) -> bool {
    !(row.family == 0 && (1..10).contains(&row.fault))
}
fn matrix(rows: Vec<Cell>) -> [[usize; 3]; 8] {
    matrix_with(rows, |runnable| {
        runnable.iter().copied().map(cell).collect()
    })
}
fn matrix_with(
    rows: Vec<Cell>,
    execute: impl FnOnce(&[Cell]) -> Vec<execution::CellResult>,
) -> [[usize; 3]; 8] {
    let mut counts = [[0usize; 3]; super::FAMILIES.len()];
    let mut anchors = BTreeMap::new();
    let mut red = Vec::new();
    let mut remainders = BTreeMap::new();
    let declared = rows.len();
    let expected_not_run = rows.iter().filter(|row| !is_runnable(row)).count();
    let runnable: Vec<_> = rows.iter().copied().filter(is_runnable).collect();
    let mut observations = execute(&runnable).into_iter();
    for row in rows {
        if !is_runnable(&row) {
            counts[row.family][1] += 1;
            println!("NOT_RUN:{row:?}:empty family has no member at which a preparation, compute or completion failure can be placed");
        } else {
            counts[row.family][0] += 1;
            match observations
                .next()
                .expect("every runnable row has its own observation")
            {
                Err(error) => {
                    counts[row.family][2] += 1;
                    red.push(format!("{row:?}: {error}"));
                }
                Ok((observed, remainder)) => {
                    if [4, 5].contains(&row.family) {
                        if let Some(expected) = remainders.get(&row.fault) {
                            if *expected != remainder {
                                counts[row.family][2] += 1;
                                red.push(format!("exact shared work/span remainder {row:?}: {expected:?} versus {remainder:?}; observed={observed:?}"));
                            }
                        } else {
                            remainders.insert(row.fault, remainder);
                        }
                    }
                    let key = (row.family, row.fault);
                    if let Some(expected) = anchors.get(&key) {
                        if &observed != expected {
                            counts[row.family][2] += 1;
                            red.push(format!("posture/order/reuse equality {row:?}: {expected:?} versus {observed:?}"));
                        }
                    } else {
                        println!("ANCHOR:{key:?}:{observed}");
                        anchors.insert(key, observed);
                    }
                }
            }
        }
    }
    assert!(observations.next().is_none(), "no undeclared observations");
    for (family, count) in counts.iter().enumerate() {
        println!("MATRIX_FAMILY:{}:{count:?}", super::FAMILY_NAMES[family]);
    }
    println!("MATRIX_COUNTS:{counts:?}");
    assert_eq!(
        counts.iter().map(|c| c[0]).sum::<usize>(),
        declared - expected_not_run,
        "every runnable declared row was executed"
    );
    assert_eq!(
        counts.iter().map(|c| c[1]).sum::<usize>(),
        expected_not_run,
        "only the named empty-family failures are not run"
    );
    for finding in &red {
        println!("NAMED_RED:{finding}");
    }
    assert!(
        red.is_empty(),
        "matrix has named production or fixture findings; all other rows were run"
    );
    counts
}
#[test]
fn required_wave_iteration_21_cells() {
    matrix(cases::iteration());
}
#[test]
fn required_wave_full_declared_matrix() {
    let rows = cases::table();
    assert_eq!(
        rows.len(),
        1032,
        "fixed Certified product plus shipped slice and deadlines"
    );
    for family in 0..super::FAMILIES.len() {
        assert_eq!(
            rows.iter().filter(|row| row.family == family).count(),
            129,
            "declared product per family"
        );
    }
    assert_full_counts(matrix_with(rows, execution::parallel));
}

#[test]
fn required_price_failure_pair() {
    matrix(
        [4, 5]
            .map(|family| Cell {
                family,
                workers: 1,
                declaration: 0,
                dispatch: cases::UNSEEDED,
                fault: 2,
                reuse: false,
            })
            .to_vec(),
    );
}
#[test]
fn required_price_all_failure_remainders() {
    matrix(
        (0..10)
            .flat_map(|fault| {
                [4, 5].map(move |family| Cell {
                    family,
                    workers: 1,
                    declaration: 0,
                    dispatch: cases::UNSEEDED,
                    fault,
                    reuse: false,
                })
            })
            .collect(),
    );
}

#[test]
fn required_axis_witnesses() {
    matrix(vec![
        Cell {
            family: 4,
            workers: 1,
            declaration: 0,
            dispatch: cases::UNSEEDED,
            fault: 0,
            reuse: false,
        },
        Cell {
            family: 4,
            workers: 1,
            declaration: 1,
            dispatch: cases::UNSEEDED,
            fault: 0,
            reuse: false,
        },
        Cell {
            family: 1,
            workers: 2,
            declaration: 0,
            dispatch: cases::UNSEEDED,
            fault: 0,
            reuse: false,
        },
        Cell {
            family: 1,
            workers: 5,
            declaration: 0,
            dispatch: cases::UNSEEDED,
            fault: 0,
            reuse: false,
        },
    ]);
}
#[test]
fn required_unseeded_dispatch_slice() {
    matrix(cases::unseeded());
}

#[test]
fn required_nested_parallel_family() {
    matrix(
        cases::table()
            .into_iter()
            .filter(|cell| cell.family == 7 && [0, 4, 8].contains(&cell.fault))
            .collect(),
    );
}

#[test]
fn required_nested_parallel_shape() {
    matrix(vec![Cell {
        family: 7,
        workers: 1,
        declaration: 0,
        dispatch: cases::UNSEEDED,
        fault: 0,
        reuse: false,
    }]);
}

#[test]
fn required_present_deadlines_and_singleton_failures_are_rows() {
    let mut rows = cases::deadlines();
    rows.extend(
        cases::unseeded()
            .into_iter()
            .filter(|row| row.family == 1 && row.fault == 2),
    );
    matrix(rows);
}

#[test]
fn required_actual_worker_witnesses() {
    matrix(
        [4, 7]
            .into_iter()
            .flat_map(|family| {
                [2, 5].map(move |workers| Cell {
                    family,
                    workers,
                    declaration: 0,
                    dispatch: cases::UNSEEDED,
                    fault: 0,
                    reuse: false,
                })
            })
            .collect(),
    );
}

#[test]
fn required_certified_product_is_present() {
    let rows = cases::table();
    // Eight shapes * three widths * two declarations * ten faults * two reuse.
    assert_eq!(
        rows.iter().filter(|r| r.dispatch == 0).count(),
        960,
        "every shape keeps the fixed-seed Certified placement"
    );
    // Eight shapes * three widths * (success and rank-middle failure).
    assert_eq!(
        rows.iter()
            .filter(|r| r.dispatch == cases::UNSEEDED && r.fault != 10)
            .count(),
        48,
        "the shipped unseeded slice remains beside certification"
    );
}

#[test]
fn required_certified_paths_match_plain_success_and_singleton_stop() {
    let mut rows = Vec::new();
    for dispatch in [0, cases::UNSEEDED] {
        for workers in [1, 2, 5] {
            for (family, fault) in [(7, 0), (1, 2)] {
                rows.push(Cell {
                    family,
                    workers,
                    declaration: 0,
                    dispatch,
                    fault,
                    reuse: false,
                });
            }
        }
    }
    matrix(rows);
}

fn assert_full_counts(counts: [[usize; 3]; 8]) {
    // Per shape: 3 * 2 * 10 * 2 Certified + 3 * 2 plain + 3 deadline = 129.
    // Empty failures: 3 * 2 * 9 * 2 Certified + 3 plain = 111; 18 run.
    // Other seven shapes run all 129; 18 + 7 * 129 = 921, with no red rows.
    assert_eq!(
        counts,
        [
            [18, 111, 0],
            [129, 0, 0],
            [129, 0, 0],
            [129, 0, 0],
            [129, 0, 0],
            [129, 0, 0],
            [129, 0, 0],
            [129, 0, 0]
        ],
        "the full product executes 921 rows and names only 111 Empty failures"
    );
}

#[test]
fn required_full_matrix_counts_reject_a_dropped_row() {
    let complete = [
        [18, 111, 0],
        [129, 0, 0],
        [129, 0, 0],
        [129, 0, 0],
        [129, 0, 0],
        [129, 0, 0],
        [129, 0, 0],
        [129, 0, 0],
    ];
    assert_full_counts(complete);
    for (family, kind) in [(1, 0), (0, 1)] {
        let mut missing = complete;
        missing[family][kind] -= 1;
        assert!(
            std::panic::catch_unwind(|| assert_full_counts(missing)).is_err(),
            "a run or named-not-run row cannot disappear from the full product"
        );
    }
}
