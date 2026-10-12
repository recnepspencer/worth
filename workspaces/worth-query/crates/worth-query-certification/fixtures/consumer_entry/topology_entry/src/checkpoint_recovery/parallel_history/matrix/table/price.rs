//! Same graph, facts and encoded widths; only declared checkpoints double.
use super::super::cases::Cell;
#[derive(Debug)]
struct PricedObservation {
    encoded: String,
    work: u64,
    span: u64,
    checkpoint_work: u64,
    checkpoint_span: u64,
    semantic: String,
}
fn amount(encoded: &str, name: &str) -> u64 {
    encoded
        .split_once(&format!("{name}: "))
        .unwrap()
        .1
        .split(',')
        .next()
        .unwrap()
        .parse()
        .unwrap()
}
const CELL_TEST: &str =
    "checkpoint_recovery::parallel_history::matrix::runtime::required_matrix_cell";

fn run(row: Cell, scale: u64) -> Result<PricedObservation, String> {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CELL_TEST, "--nocapture"])
        .env("WORTH_REQUIRED_MATRIX_CELL", row.encode())
        .env("WORTH_REQUIRED_PRICE_SCALE", scale.to_string())
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
        .expect("cell reports its typed observation");
    Ok(PricedObservation {
        encoded: encoded.to_owned(),
        work: amount(encoded, "work"),
        span: amount(encoded, "span"),
        checkpoint_work: amount(encoded, "checkpoint_work"),
        checkpoint_span: amount(encoded, "checkpoint_span"),
        semantic: encoded.split_once("committed: ").unwrap().1.to_owned(),
    })
}

pub(super) fn cell(row: Cell) -> Result<(String, (u64, u64)), String> {
    let base = run(row, 1)?;
    let twin = run(row, 2)?;
    if base.semantic != twin.semantic {
        return Err(format!(
            "price twins preserve facts and canonical prefix: {base:?} versus {twin:?}"
        ));
    }
    let declared = (
        twin.checkpoint_work - base.checkpoint_work,
        twin.checkpoint_span - base.checkpoint_span,
    );
    let expected = (base.work + declared.0, base.span + declared.1);
    if (twin.work, twin.span) != expected {
        return Err(format!("advancement work and span equal the independent checkpoint price by value: base={:?}, twin={:?}, expected={expected:?}, declared={declared:?}", (base.work, base.span), (twin.work, twin.span)));
    }
    absolute_anchor(row, &base)?;
    let remainder = (
        base.work - base.checkpoint_work,
        base.span - base.checkpoint_span,
    );
    Ok((base.encoded, remainder))
}

fn absolute_anchor(row: Cell, observed: &PricedObservation) -> Result<(), String> {
    if row.fault != 0 {
        return Ok(());
    }
    // Owner/framing totals are the named aggregate remainder; checkpoint prices
    // below come from the input model. This pins one absolute bill per shape,
    // independently of the differential (zero bills cannot certify themselves).
    let expected = [
        (0, 0),                 // Already-current empty caller dispatches and charges nothing.
        (557_789, 557_774),     // owner 557787/557772 + one checkpoint 2/2.
        (1_410_281, 1_410_221), // owner 1410273/1410213 + four serial stages 8/8.
        (1_410_281, 1_410_212), // owner 1410273/1410206 + diamond sum/max 8/6.
        (1_410_311, 1_410_209), // owner 1410303/1410207 + four independent 8/2.
        (1_410_349, 1_410_229), // same owner + declared [1,8,15,22] => 46/22.
        (1_410_419, 1_410_236), // owner 1410387/1410228 + four zipped members 32/8.
        (1_410_435, 1_410_230), // owner 1410403/1410228 + four nested maps 32/2.
    ][row.family];
    if (observed.work, observed.span) != expected {
        return Err(format!("absolute shape anchor prices its advancement by value: {row:?}: actual={:?}, expected={expected:?}", (observed.work, observed.span)));
    }
    Ok(())
}

#[test]
fn required_every_shape_prices_its_checkpoints() {
    for family in 0..super::super::FAMILIES.len() {
        for fault in [0, 4] {
            if family == 0 && fault != 0 {
                continue;
            }
            let row = Cell {
                family,
                workers: 1,
                declaration: 0,
                dispatch: super::super::cases::UNSEEDED,
                fault,
                reuse: false,
            };
            if let Err(reason) = cell(row) {
                panic!("{row:?}: {reason}");
            }
        }
    }
}

#[test]
fn required_nested_shape_prices_its_critical_path() {
    for fault in [0, 4] {
        let row = Cell {
            family: 7,
            workers: 1,
            declaration: 0,
            dispatch: super::super::cases::UNSEEDED,
            fault,
            reuse: false,
        };
        if let Err(reason) = cell(row) {
            panic!("{row:?}: {reason}");
        }
    }
}
