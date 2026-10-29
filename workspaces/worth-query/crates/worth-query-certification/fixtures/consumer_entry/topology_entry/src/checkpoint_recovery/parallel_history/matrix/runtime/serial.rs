use super::*;
#[test]
fn required_matrix_cell() {
    let cell = std::env::var("WORTH_REQUIRED_MATRIX_CELL")
        .map(|text| cases::Cell::decode(&text))
        .unwrap_or(cases::Cell {
            family: 1,
            workers: 2,
            declaration: 0,
            dispatch: cases::UNSEEDED,
            fault: 0,
            reuse: true,
        });
    let _guard = crate::checkpoint_recovery::checkpoint_recovery_test_guard();
    let value = run(cell);
    assert!(
        value.work >= value.checkpoint_work && value.span >= value.checkpoint_span,
        "whole dispatched compute stays charged"
    );
    println!(
        "MODEL_REMAINDER:work={} span={}",
        value.work - value.checkpoint_work,
        value.span - value.checkpoint_span
    );
    println!("MATRIX_OBSERVATION:{value:?}");
}
