//! Depth and sibling width are independent; scheduled maxima reuse this cost court.
mod host;
mod measurement;
mod scenario;
use measurement::Cost;
use scenario::Sample;

fn bounded(small: &Sample, large: &Sample, envelope: u64) {
    for (name, small, large) in [
        ("edit", &small.edit, &large.edit),
        (
            "distant_selection",
            &small.distant_selection,
            &large.distant_selection,
        ),
        (
            "return_selection",
            &small.return_selection,
            &large.return_selection,
        ),
        (
            "historical_read",
            &small.historical_read,
            &large.historical_read,
        ),
        ("diagnostics", &small.diagnostics, &large.diagnostics),
    ] {
        local(name, small, large, envelope);
    }
}

fn local(name: &str, small: &Cost, large: &Cost, envelope: u64) {
    assert!(
        small.journal.visits[0] > 0,
        "observe actual node lookup: {name}"
    );
    assert_eq!(
        large.journal.visits, small.journal.visits,
        "fixed local work must not enumerate retained nodes or siblings: {name}"
    );
    assert_eq!(
        large.journal.frontier, small.journal.frontier,
        "unrelated retained sources cannot become this command's frontier: {name}"
    );
    assert!(
        small.native_work > 0,
        "observe actual receiving-owner work: {name}"
    );
    assert!(
        large.native_work <= small.native_work * envelope,
        "receiving-owner validation must not replay retained history: {name}; {small:?}; {large:?}"
    );
    assert!(
        large.journal.requested_bytes <= small.journal.requested_bytes * envelope,
        "balanced index quotes cannot copy the full history: {name}"
    );
    assert!(
        small.peak_charged_bytes > 0,
        "observe charged working memory: {name}"
    );
    assert!(
        large.peak_charged_bytes <= small.peak_charged_bytes * envelope,
        "actual charged working memory cannot copy the full history: {name}"
    );
    assert!(large.journal.backed_bytes <= large.journal.requested_bytes);
}

#[test]
fn local_commands_and_bounded_pages_stay_local_as_depth_and_sibling_width_vary_independently() {
    let short_narrow = scenario::run(1, 2);
    let short_broad = scenario::run(1, 8);
    let deep_narrow = scenario::run(15, 2);
    let deep_broad = scenario::run(15, 8);
    // Each comparison changes exactly one independent historical axis.
    bounded(&short_narrow, &deep_narrow, 3);
    bounded(&short_broad, &deep_broad, 3);
    bounded(&short_narrow, &short_broad, 3);
    bounded(&deep_narrow, &deep_broad, 3);
}

#[test]
#[ignore = "scheduled query-scale qualification; retains1000 actual authored chain steps"]
fn a_thousand_step_chain_keeps_local_commands_and_historical_reads_bounded() {
    bounded(&scenario::run(1, 2), &scenario::run(1_000, 2), 8);
}

#[test]
#[ignore = "scheduled query-scale qualification; retains1000 actual sibling futures"]
fn a_thousand_sibling_futures_keep_local_commands_and_child_prefixes_bounded() {
    bounded(&scenario::run(1, 2), &scenario::run(1, 1_000), 8);
}
