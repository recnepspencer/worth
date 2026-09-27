use std::time::Duration;

use super::{travel, whole_periods, PERIOD};

#[test]
fn the_drag_turns_at_its_reach_and_returns_to_the_corner() {
    let reach = [840, 460];
    assert_eq!(travel(0.0, reach), [0, 0]);
    assert_eq!(travel(PERIOD / 2.0, reach), reach);
    assert_eq!(travel(PERIOD, reach), [0, 0]);
    assert_eq!(travel(3.0 * PERIOD, reach), [0, 0]);
}

#[test]
fn a_drag_of_whole_periods_turns_at_every_half_period() {
    let seconds = whole_periods(Duration::from_secs(12)).as_secs_f64();
    let steps = (seconds * 100.0).round() as i32;
    let mut widths: Vec<i32> = (0..=steps)
        .map(|step| travel(f64::from(step) / 100.0, [840, 460])[0])
        .collect();
    // The corner rests for a sample or two at each turn; a rest is no turn.
    widths.dedup();
    let reversals = widths
        .windows(3)
        .filter(|run| (run[1] - run[0]) * (run[2] - run[1]) < 0)
        .count();
    assert_eq!(
        reversals, 5,
        "three periods turn inward three times, outward twice"
    );
}

#[test]
fn a_drag_lasts_whole_periods() {
    assert_eq!(
        whole_periods(Duration::from_secs(12)),
        Duration::from_secs(12)
    );
    assert_eq!(
        whole_periods(Duration::from_secs(10)),
        Duration::from_secs(12)
    );
    assert_eq!(
        whole_periods(Duration::from_secs(0)),
        Duration::from_secs(4)
    );
}
