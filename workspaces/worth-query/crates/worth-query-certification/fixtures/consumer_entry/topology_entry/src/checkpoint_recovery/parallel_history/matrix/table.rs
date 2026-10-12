//! Fresh processes preserve binding inputs; every row is accounted for.
use super::cases::{self, Cell};
use std::collections::BTreeMap;
const CELL_TEST: &str =
    "checkpoint_recovery::parallel_history::matrix::runtime::required_matrix_cell";
fn cell(cell: Cell) -> Result<(String, (u64, u64)), String> {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CELL_TEST, "--nocapture"])
        .env("WORTH_REQUIRED_MATRIX_CELL", cell.encode())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        return Err(format!(
            "{stdout}{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let encoded = stdout
        .lines()
        .find_map(|line| line.strip_prefix("MATRIX_OBSERVATION:"))
        .expect("cell returns its observation");
    let remainder = stdout
        .lines()
        .find_map(|line| line.strip_prefix("MODEL_REMAINDER:work="))
        .unwrap();
    let (work, span) = remainder.split_once(" span=").unwrap();
    Ok((
        encoded.to_owned(),
        (work.parse().unwrap(), span.parse().unwrap()),
    ))
}
fn matrix(rows: Vec<Cell>) {
    let mut counts = [[0usize; 3]; 7];
    let mut anchors = BTreeMap::new();
    let mut red = Vec::new();
    let mut remainders = BTreeMap::new();
    for row in rows {
        if row.family == 0 && row.fault > 0 {
            counts[0][1] += 1;
            println!("NOT_RUN:{row:?}:empty family has no failure member");
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
        1400,
        "one-worker seed is equivalent and counted once"
    );
    for family in 0..7 {
        assert_eq!(
            rows.iter().filter(|row| row.family == family).count(),
            200,
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
                dispatch: 0,
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
                    dispatch: 0,
                    fault,
                    reuse: false,
                })
            })
            .collect(),
    );
}
