//! Shared dependency fanout through actual native readers.
use super::*;

#[test]
fn one_shared_fact_reaches_every_actual_reader_without_population_fact_probes() {
    let _guard = checkpoint_recovery_test_guard();
    let mut compared = Vec::new();
    for size in [1, 10, 100] {
        let application = install(|graph| {
            facts::seed_set(graph, "even", -0.0);
            for number in 0..size {
                seed_entry(
                    graph,
                    &["even"],
                    number,
                    RegionEntry {
                        id: number as u64,
                        region: (2 * number) as u32,
                        value: 1.0,
                        work: 1,
                        fault: None,
                    },
                );
            }
        });
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        let change = |fact, value: f64, command| {
            edit(
                &request,
                &application,
                EntryEdit::new("even", 0, fact, value.to_bits()),
                command,
            )
        };
        demand(&request, &application);
        change(EntryFact::Weight, 2.0_f64, 500_001);
        fact_verification();
        let (contacts, runs) = demand(&request, &application);
        let examined = fact_verification();
        assert_eq!(contacts, 1);
        let [run] = runs.as_slice() else {
            panic!("one real shared-fact publication");
        };
        assert_eq!(run.runs, [Run::Incremental]);
        assert_eq!(
            run.calls,
            OwnerCalls {
                plans: 0,
                keys: 0,
                gathers: size,
                kernels: size
            }
        );
        assert_eq!(
            run.outcome.as_ref().map(|(bits, _)| *bits),
            Ok((3.0 * size as f64).to_bits())
        );
        assert!(
            !run.published.is_empty(),
            "the computed total is actually retained at publication"
        );
        assert_eq!(examined.carried_seal_visits, 0);
        compared.push(examined.retained_native_visits);
        // A field ABA changes its native revision: output verification must
        // still execute. Original pre-effect values then permit actual carried
        // gathers, rather than confusing rebased revisions with their content.
        change(EntryFact::Weight, -0.0_f64, 500_002);
        demand(&request, &application);
        change(EntryFact::Value, 2.0_f64, 500_003);
        change(EntryFact::Value, 1.0_f64, 500_004);
        fact_verification();
        let (contacts, aba_runs) = demand(&request, &application);
        let aba = fact_verification();
        assert_eq!(contacts, 1, "native field revision preserves the ABA fence");
        assert_eq!(aba_runs.len(), 1);
        assert_eq!(aba_runs[0].runs, [Run::Incremental]);
        assert_eq!(
            aba_runs[0].calls,
            OwnerCalls {
                plans: 0,
                keys: 0,
                gathers: 0,
                kernels: 0
            }
        );
        assert_eq!(
            aba_runs[0].outcome.as_ref().map(|(bits, _)| *bits),
            Ok((size as f64).to_bits())
        );
        assert!(aba.source_currentness_visits > 0);
        assert!(
            aba.retained_native_visits > 0,
            "original values are actually compared"
        );
        assert_eq!(aba.carried_seal_visits, 0);
        change(EntryFact::Weight, 2.0_f64, 500_005);
        fact_verification();
        let (_, after_aba) = demand(&request, &application);
        let consumed = fact_verification();
        assert_eq!(after_aba.len(), 1);
        assert_eq!(after_aba[0].runs, [Run::Incremental]);
        assert_eq!(
            after_aba[0].outcome.as_ref().map(|(bits, _)| *bits),
            Ok((3.0 * size as f64).to_bits())
        );
        assert_eq!(
            consumed.retained_native_visits, examined.retained_native_visits,
            "performed publication starts its own exact computation frontier"
        );
        assert_eq!(consumed.carried_seal_visits, 0);
    }
    assert!(compared[0] > 0);
    assert!(
        compared.iter().all(|visits| *visits == compared[0]),
        "shared fact is observed once; its real call fanout is not hidden: {compared:?}"
    );
}
