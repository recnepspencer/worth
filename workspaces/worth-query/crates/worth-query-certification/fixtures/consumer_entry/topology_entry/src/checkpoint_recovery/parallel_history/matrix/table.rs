//! Fresh processes preserve binding inputs; every row is accounted for.
use super::cases::{self, Cell};
use std::collections::BTreeMap;
const CELL_TEST: &str =
    "checkpoint_recovery::parallel_history::matrix::runtime::required_matrix_cell";
fn cell(cell: Cell) -> Result<String, String> {
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
    Ok(encoded.to_owned())
}
fn matrix(rows: Vec<Cell>) {
    let mut counts = [[0usize; 3]; 7];
    let mut anchors = BTreeMap::new();
    let mut red = Vec::new();
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
                Ok(observed) => {
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
