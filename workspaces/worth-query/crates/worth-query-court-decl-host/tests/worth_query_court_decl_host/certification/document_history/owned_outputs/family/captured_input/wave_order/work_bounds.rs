//! PR B wave cost courts (#254 grid, #232 wide fan-out, zero healthy contacts).
use super::super::super::super::synchronous_edit::{
    envelope_fixture as fixture, graph::many_routes,
};
use super::super::*;
use super::{install, producer_contacts, wave_units};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

/// `W` chains of length `L`: the batch count stays additive in `W + L`, and
/// charged wave work per placement stays within directed log-linear bounds.
#[test]
fn wide_chains_cost_placements_not_turns_times_placements() {
    let measure = |width: usize, length: usize| {
        type Host = WorthQueryProgramApplicationRuntime<Schema, chains::Program>;
        let host: Host = install(world::Configuration {
            captured_chains: Some((width, length)),
            ..Default::default()
        });
        let (outcome, events, units) =
            edit_stage!(&host, chains::Tree, "chains-born", "carriage:d", 4);
        assert_eq!(outcome, "committed");
        let batches = events
            .iter()
            .filter(|event| event.starts_with("batch "))
            .count() as f64;
        let placements = (width * length) as f64;
        (
            batches,
            units as f64 / (placements * (2.0 * placements).log2()),
            units,
        )
    };
    let grid = [(2, 2), (2, 8), (8, 2), (8, 8)].map(|(w, l)| ((w, l), measure(w, l)));
    eprintln!("#254 wave grid (W, L) -> (batches, normalized, units): {grid:?}");
    let [b22, b28, b82, b88] = grid.map(|(_, (batches, _, _))| batches);
    assert!(
        b88 - b82 - b28 + b22 <= 0.10 * b88,
        "batches serialize W x L: {grid:?}"
    );
    // Plan r3 Blocker 3 ruling: directed superlinear-growth upper bounds.
    let [n22, n28, n82, n88] = grid.map(|(_, (_, normalized, _))| normalized);
    for (wide, label) in [(n88, "(8,8)"), (n82, "(8,2)"), (n28, "(2,8)")] {
        assert!(
            wide <= 2.0 * n22,
            "normalized work at {label} grows superlinearly over (2,2): {grid:?}"
        );
    }
    let [c22, c28, c82, c88] = grid.map(|(_, (_, _, units))| units as f64);
    for (wide, narrow) in [(c82, c22), (c88, c28), (c28, c22), (c88, c82)] {
        assert!(
            wide <= 4.5 * narrow,
            "one axis grows superlinearly: {grid:?}"
        );
    }
}

/// PR C §4 touch-filtered reclosure: a quiescent outgoing reclosure runs
/// again only the completed rows whose outgoing support changed since it was
/// retained, or that hold none. Each chain link's outgoing selector reads
/// what no later append changes, so the born edit re-closes no row; full
/// reclosure (PR B) re-ran every one of the `W * L` links. Red with the
/// filter disabled.
#[test]
fn quiescent_reclosure_reruns_only_rows_whose_outgoing_support_changed() {
    for (width, length) in [(2, 2), (8, 8)] {
        type Host = WorthQueryProgramApplicationRuntime<Schema, chains::Program>;
        let host: Host = install(world::Configuration {
            captured_chains: Some((width, length)),
            ..Default::default()
        });
        let observation = WorthQueryExecutionObservation::install_on_this_thread();
        let (outcome, events, _) =
            edit_stage!(&host, chains::Tree, "chains-reclosure", "carriage:d", 4);
        assert_eq!(outcome, "committed");
        assert!(events.iter().any(|event| event == "quiescence"), "{events:#?}");
        let revisited = observation.outgoing_rows_revisited_for_test();
        eprintln!("PR C reclosure (W={width}, L={length}): {revisited} rows re-closed");
        assert_eq!(revisited, 0, "an untouched link was re-closed (W={width}, L={length})");
    }
}

/// #232-like one-stage fan-out at 16 and 32 routes: wave charged work scales
/// near-linearly with the route count, not with its square.
#[test]
fn wide_one_stage_fan_out_wave_work_follows_routes_at_16_and_32() {
    let measure = |routes: u64| {
        let host = many_routes::install(false);
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        many_routes::seed(&host, &request, routes);
        let obs = WorthQueryExecutionObservation::install_on_this_thread();
        let outcome = many_routes::fan_out(&host, &request, routes, routes as usize + 1);
        assert!(
            matches!(
                outcome,
                WorthQueryApplicationPerformedMutationOutcome::Performed(_)
            ),
            "wide fan-out must perform"
        );
        wave_units(&obs)
    };
    let (small, large) = (measure(16), measure(32));
    eprintln!("#232 one-stage fan-out wave units: 16 -> {small}, 32 -> {large}");
    assert!(small > 0, "wave charged work must be observed");
    // 17 then 33 routes: linear-log grows by ~2x; quadratic grows by ~4x.
    assert!(
        large * 2 <= small * 5,
        "fan-out wave units grew superlinearly from {small} to {large}"
    );
}

/// Base-green half of the wide captured-reader cohort: a healthy unchanged edit
/// contacts zero producers and runs zero captures across 8 siblings.
#[test]
fn healthy_wide_edit_adds_zero_producer_contacts() {
    type Host = WorthQueryProgramApplicationRuntime<Schema, supplier::Program>;
    let host: Host = install(world::Configuration {
        captured_composed: Some((8, false)),
        ..Default::default()
    });
    let (born, _, _) = edit_stage!(&host, supplier::SupplierTree, "wide-born", "carriage:d", 4);
    assert_eq!(born, "committed");
    let _obs = WorthQueryExecutionObservation::install_on_this_thread();
    let before_contacts = producer_contacts(&host);
    let before_captures = handler::captures_on_this_thread();
    let (healthy, _, units) = edit_stage!(
        &host,
        supplier::SupplierTree,
        "wide-healthy",
        "carriage:d",
        4
    );
    assert_eq!(healthy, "committed");
    assert_eq!(
        producer_contacts(&host) - before_contacts,
        0,
        "healthy wide edit must contact zero producers"
    );
    assert_eq!(
        handler::captures_on_this_thread() - before_captures,
        0,
        "healthy wide edit must open zero captures"
    );
    eprintln!("#254 healthy wide (W=8) wave units: {units}");
}
