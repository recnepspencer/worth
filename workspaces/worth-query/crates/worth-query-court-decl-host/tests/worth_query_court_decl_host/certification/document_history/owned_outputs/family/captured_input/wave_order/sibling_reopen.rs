//! PR C width court (#254): stored same-rank siblings that read the one writer
//! the supplier owes are reopened together after that writer changes their read.
use super::super::super::super::synchronous_edit::envelope_fixture as fixture;
use super::super::*;
use super::{install, scheduling_measures};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

type Host = WorthQueryProgramApplicationRuntime<Schema, supplier::Program>;

/// The changing edit's outcome, wave units, and the retained reads its
/// qualification visited, read before the observation is released.
fn changed_edit(host: &Host) -> (bool, u64, u64) {
    let observation = WorthQueryExecutionObservation::install_on_this_thread();
    let outcome = retention::edit_supplier(host, "wide-changed", 5);
    let (units, visited) = scheduling_measures(&observation);
    let committed = matches!(
        outcome,
        worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome::Committed(_)
    );
    (committed, units, visited)
}

/// `width` stored siblings read the one writer the supplier owes. Returns the
/// wave units of a healthy edit and of the changing edit, and the retained
/// reads the changing edit's qualification visited.
fn wide(width: usize) -> (u64, u64, u64) {
    let host: Host = install(world::Configuration {
        captured_composed: Some((width, false)),
        ..Default::default()
    });
    let born = edit_stage!(&host, supplier::SupplierTree, "wide-born", "carriage:d", 4);
    assert_eq!(born.0, "committed", "{born:?}");
    let siblings = (0..width).map(|k| format!("captured:capture:w{k}"));
    let before: Vec<_> = siblings
        .clone()
        .map(|key| retention::row(&host, &key))
        .collect();
    // Source 3 + late50 + 2 * d40.
    assert!(before.iter().all(|row| row.value == 3 + 50 + 2 * 40));
    let captures = handler::captures_on_this_thread();
    let healthy = edit_stage!(
        &host,
        supplier::SupplierTree,
        "wide-healthy",
        "carriage:d",
        4
    );
    assert_eq!(healthy.0, "committed", "{healthy:?}");
    assert_eq!(
        handler::captures_on_this_thread(),
        captures,
        "a healthy sibling verifies without capture"
    );
    let (committed, changed, visited) = changed_edit(&host);
    assert!(committed, "the changing edit commits");
    assert_eq!(
        handler::captures_on_this_thread() - captures,
        width as u64,
        "each sibling recaptures once"
    );
    for (key, row) in siblings.zip(before) {
        let after = retention::row(&host, &key);
        assert_eq!(
            (after.value, after.native_id),
            (3 + 50 + 2 * 50, row.native_id)
        );
    }
    (healthy.2, changed, visited)
}

/// The changing edit reopens every stale sibling once and derives them in one
/// batch, so each sibling adds a constant plus its ordered-map logs: a x4
/// width growth is linear with 0.5 of headroom for the log (as the PR B grid).
/// Reopening one sibling per round re-runs the frontier per sibling, a
/// `c * W^2` term whose x16 growth this bound catches. Qualification looks
/// each stale sibling up by its read: the retained reads it visits grow with
/// the width, never with the width times the retained reads (a scan).
#[test]
fn same_rank_siblings_recapture_together_after_one_writer() {
    let (narrow, wide) = (wide(2), wide(8));
    eprintln!(
        "#254 width: W=2 {narrow:?}, W=8 {wide:?} (healthy units, changed units, reads visited)"
    );
    let delta = |(healthy, changed, _): (u64, u64, u64)| changed.saturating_sub(healthy);
    assert!(
        delta(wide) as f64 <= 4.5 * delta(narrow) as f64,
        "{narrow:?} {wide:?}"
    );
    assert!(
        narrow.2 > 0,
        "qualification visited the stale siblings: {narrow:?}"
    );
    assert!(
        wide.2 as f64 <= 4.5 * narrow.2 as f64,
        "qualification visits grow with width times retained reads: {narrow:?} {wide:?}"
    );
}
