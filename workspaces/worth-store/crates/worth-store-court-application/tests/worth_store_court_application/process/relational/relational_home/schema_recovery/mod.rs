//! A home serves the schema its recovered main head records (D5): the initial
//! schema until a durable schema transition, its target after. Opening with
//! any other declared registry — the other side of the transition, or the
//! same kind contracts under another version label — refuses before service
//! and leaves the home intact. Each branch still recovers under the schema
//! boundary its own commits recorded. A registry with other kind contracts
//! refuses before service.
use worth_store_testkit_application_oracle::facade::home_opening::HomeOpeningRefused as OpenRefusal;

#[allow(unused_imports)]
use crate::schema_recovery::*;

use {
    crate::durable_run::open_declaring, crate::durable_run::recorded_schema,
    crate::durable_run::served_schema, crate::durable_run::steps_before,
};
use {
    crate::schema_transition::basis_of, crate::schema_transition::SOURCE,
    crate::schema_transition::TARGET,
};

mod between_operations;

#[test]
fn a_home_serves_only_the_schema_its_main_head_records() {
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    // Without checkpoints the transition is readmitted from the journal; with
    // one every four commits it is first in the tail, then inside the image.
    for checkpoint_every in [0, 4] {
        let script = crate::workload::Script::generate(SEED, COMMITS, checkpoint_every);
        for prefix in [THROUGH_TRANSITION - 1, THROUGH_TRANSITION, COMMITS] {
            let case =
                format!("seed {SEED:#x} checkpoint every {checkpoint_every}, {prefix} commits");
            let directory = tempfile::tempdir().unwrap();
            let location = directory.path().join("home");
            let held = steps_before(&script, prefix);
            // Kill only after the actual completed prefix, without taking a
            // close checkpoint. The original World owner remains in the child.
            let acknowledged = between_operations::run(
                &location,
                checkpoint_every,
                prefix,
                between_operations::ScriptPhase::Prefix,
                &[],
                directory.path(),
            );

            let served = served_schema(held);
            for declared in [SOURCE, TARGET, OTHER_LABEL] {
                if declared == served {
                    continue;
                }
                let refused = open_declaring(&location, declared, &execution);
                let refusal = refused.err().unwrap_or_else(|| {
                    panic!("{case}: declaring {declared:?} over {served:?} opened")
                });
                assert_eq!(
                    recorded_schema(&refusal),
                    Some(served),
                    "{case}: {refusal:?}"
                );
            }
            let home = crate::durable_run::world::open(&location, served, false, &execution)
                .unwrap_or_else(|refusal| panic!("{case}: {refusal:?}"));
            crate::durable_run::world::assert_holds(&execution, &home.home, &script, &acknowledged);
            assert_eq!(
                basis_of(home.home.relational().runtime(), &branch("main")),
                served,
                "{case}"
            );
            assert_eq!(
                basis_of(home.home.relational().runtime(), &branch("feature")),
                SOURCE,
                "{case}"
            );
            // Ordinary abandonment takes no Home checkpoint. The suffix runs
            // in the real child and is killed at its completed durable boundary.
            drop(home);
            let everything = between_operations::run(
                &location,
                checkpoint_every,
                prefix,
                between_operations::ScriptPhase::Suffix,
                &acknowledged,
                directory.path(),
            );
            assert_eq!(&everything[..prefix], &acknowledged[..], "{case}");

            let home = crate::durable_run::world::open(&location, TARGET, false, &execution)
                .unwrap_or_else(|refusal| panic!("{case}: second open: {refusal:?}"));
            crate::durable_run::world::assert_holds(&execution, &home.home, &script, &everything);
            crate::durable_run::world::close(home, &execution);
        }
    }
}

#[test]
fn a_registry_with_other_kind_contracts_refuses_before_service() {
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let script = crate::workload::Script::generate(SEED, COMMITS, 4);
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let home = crate::durable_run::world::open(&location, SOURCE, true, &execution)
        .expect("a new home opens");
    crate::durable_run::world::apply(
        &home.home,
        &execution,
        steps_before(&script, THROUGH_TRANSITION),
    );
    let acknowledged = crate::workload::artifacts_of(&execution, home.home.relational().runtime());
    crate::durable_run::world::close(home, &execution);

    let foreign = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .runtime_setup(|runtime| {
            runtime.runtime_name(crate::workload::RUNTIME_NAME);
        })
        .schema_setup(|setup| {
            setup.schema_registry(foreign_registry());
        });
    let refused = worth_store_testkit_application_oracle::facade::home_opening::open(
        worth_store_testkit_application_oracle::facade::home_opening::request(&location, foreign),
        &execution,
    );
    assert!(
        matches!(
            refused.as_ref().err().and_then(OpenRefusal::source_cause),
            Some(
                worth_store_application::ApplicationHomeOpenRefusal::Relational(
                    worth_store_relational::RelationalSectionReadmissionDenial::Native(_)
                )
            )
        ),
        "{:?}",
        refused.map(|_| ())
    );
    // Explicit ordinary abandonment after observing the actual refused owner;
    // this does not claim a successful close.
    drop(refused);
    let home = crate::durable_run::world::open(
        &location,
        crate::durable_run::served_schema(steps_before(&script, THROUGH_TRANSITION)),
        false,
        &execution,
    )
    .expect("the refusal left the home intact");
    crate::durable_run::world::assert_holds(&execution, &home.home, &script, &acknowledged);
    crate::durable_run::world::close(home, &execution);
}
