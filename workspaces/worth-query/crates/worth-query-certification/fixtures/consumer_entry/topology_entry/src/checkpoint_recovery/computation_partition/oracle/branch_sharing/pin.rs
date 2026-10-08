//! A live fork pins the captured state without observer-held custody.
use super::*;
#[test]
fn a_fork_pins_its_captured_state_after_the_observer_is_drained() {
    let _guard = checkpoint_recovery_test_guard();
    let app = install(|graph| {
        facts::seed_set(graph, "even", -0.0);
        for number in 0..32 {
            seed_entry(
                graph,
                &["even"],
                number,
                RegionEntry {
                    id: number as u64,
                    region: number as u32,
                    value: 1.0,
                    work: 1,
                    fault: None,
                },
            );
        }
    });
    let parent = app.current_world();
    // run drains the observer; drop its returned handles before the fork
    // or any successor can make the old record appear independently shared.
    drop(run(&app, parent));
    let child = fork(&app, parent);
    let value = |entry, bits| {
        at_demand_scope(EntryEdit::new(
            "even",
            entry,
            super::super::super::entry_edit::EntryFact::Value,
            bits,
        ))
    };
    apply(
        &app,
        parent,
        vec![Change::Entry(value(26, 3.25_f64.to_bits()))],
        &mut 0x6118,
    );
    drop(run(&app, parent));
    apply(
        &app,
        child,
        vec![Change::Entry(value(7, 2.5_f64.to_bits()))],
        &mut 0x6119,
    );
    let child_run = run(&app, child).pop().unwrap();
    assert_eq!(
        child_run.runs,
        [Run::Incremental],
        "the fork alone keeps the captured prior retained"
    );
    assert_eq!(
        child_run.calls,
        OwnerCalls {
            plans: 0,
            keys: 0,
            gathers: 1,
            kernels: 1
        }
    );
    assert_eq!(child_run.outcome.as_ref().unwrap().0, 33.5_f64.to_bits());
}
