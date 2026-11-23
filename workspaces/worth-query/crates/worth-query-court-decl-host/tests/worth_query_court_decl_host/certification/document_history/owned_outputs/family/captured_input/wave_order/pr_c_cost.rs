//! PR C cost courts (plan §4): qualified use charges its marking, exact
//! comparisons and reopens, and stays within the bound the plan states.
use super::super::super::super::synchronous_edit::envelope_fixture as fixture;
use super::super::*;
use super::install;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

/// Shape (1) (#254 r2): a supplier-only change reopens each of `W` wide
/// captured readers, which base refused at once. Every reader is reopened
/// exactly once, and charged wave work grows log-linearly with the width:
/// `W = 32` costs at most 6 times `W = 8` (4 times the readers, a log factor
/// of 5/3, and slack), so a quadratic reopen (16 times) fails.
#[test]
fn a_wide_supplier_change_reopens_each_reader_once_at_log_linear_cost() {
    let measure = |width: usize| {
        type Host = WorthQueryProgramApplicationRuntime<Schema, supplier::Program>;
        let host: Host = install(world::Configuration {
            captured_composed: Some((width, false)),
            ..Default::default()
        });
        let (born, _, _) =
            edit_stage!(&host, supplier::SupplierTree, "wide-cost-born", "carriage:d", 4);
        assert_eq!(born, "committed");
        let (changed, events, units) =
            edit_stage!(&host, supplier::SupplierTree, "wide-cost-changed", "carriage:d", 5);
        assert_eq!(changed, "committed", "{events:#?}");
        let reopens = events.iter().filter(|event| event.starts_with("reopen ")).count();
        assert_eq!(reopens, width, "each reader reopens once: {events:#?}");
        units
    };
    let (narrow, wide) = (measure(8), measure(32));
    eprintln!("PR C shape (1) wide supplier change: W=8 {narrow} units, W=32 {wide} units");
    assert!(
        wide <= 6 * narrow,
        "reopening wide readers grew superlinearly: {narrow} -> {wide}"
    );
}

/// The dense conservative case (§1.4): query support carries no touch key, so
/// every append compares every live selection entry retained before it. Each
/// placement retains at most two support entries (its selection and its
/// outgoing support), and the edit makes at most `P + 1` appends (one per
/// placement and the authored stage): the bound is per append, `(P + 1) * 2P`,
/// and every visit is charged as wave work.
#[test]
fn dense_conservative_comparisons_stay_within_appends_times_live_entries_and_are_charged() {
    for (width, length) in [(2, 2), (8, 8)] {
        type Host = WorthQueryProgramApplicationRuntime<Schema, chains::Program>;
        let host: Host = install(world::Configuration {
            captured_chains: Some((width, length)),
            ..Default::default()
        });
        let observation = WorthQueryExecutionObservation::install_on_this_thread();
        let (outcome, _, units) =
            edit_stage!(&host, chains::Tree, "chains-dense", "carriage:d", 4);
        assert_eq!(outcome, "committed");
        let visited = observation.retained_reads_visited_for_test();
        let placements = (width * length + 1) as u64;
        eprintln!(
            "PR C dense (W={width}, L={length}): {visited} comparisons, {units} units, P={placements}"
        );
        assert!(
            visited <= (placements + 1) * 2 * placements,
            "comparisons exceed appends times live entries: {visited} at P={placements}"
        );
        assert!(units >= visited, "a comparison was not charged");
    }
}
