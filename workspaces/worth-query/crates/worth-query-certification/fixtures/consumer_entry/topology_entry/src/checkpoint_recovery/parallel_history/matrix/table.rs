//! Fresh processes preserve binding inputs; every row is accounted for.
use super::cases::{self, Cell};
use std::collections::BTreeMap;
mod price;
use price::cell;
fn matrix(rows: Vec<Cell>) {
    let mut counts = [[0usize; 3]; super::FAMILIES.len()];
    let mut anchors = BTreeMap::new();
    let mut red = Vec::new();
    let mut remainders = BTreeMap::new();
    for row in rows {
        if row.fault > 0
            && (row.family == 0 || (row.family == 1 && row.dispatch == cases::UNSEEDED))
        {
            counts[row.family][1] += 1;
            println!("NOT_RUN:{row:?}:unseeded rank >= 1 failure needs at least two members; empty families have none");
        } else {
            counts[row.family][0] += 1;
            match cell(row) {
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
    for (family, count) in counts.iter().enumerate() {
        println!("MATRIX_FAMILY:{}:{count:?}", super::FAMILY_NAMES[family]);
    }
    println!("MATRIX_COUNTS:{counts:?}");
    for finding in &red {
        println!("NAMED_RED:{finding}");
    }
    assert!(
        red.is_empty(),
        "matrix has named production or fixture findings; all other rows were run"
    );
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
        960,
        "plain leased placement has no unobserved seed axis"
    );
    for family in 0..super::FAMILIES.len() {
        assert_eq!(
            rows.iter().filter(|row| row.family == family).count(),
            120,
            "declared product per family"
        );
    }
    matrix(rows);
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
