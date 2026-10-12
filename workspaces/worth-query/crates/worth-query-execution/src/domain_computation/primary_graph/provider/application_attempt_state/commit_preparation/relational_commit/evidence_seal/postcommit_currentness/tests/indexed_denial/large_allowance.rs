use super::*;

#[test]
fn source_currentness_denial_keeps_its_large_u64_allowance() {
    let world = installed_authorization_world(true);
    let facts = committed_indexed_facts(&world);
    let fact = facts
        .iter()
        .find(|fact| {
            matches!(
                fact,
                WorthQueryApplicationObservedFact::IndexedEntitySelection { .. }
            )
        })
        .unwrap();
    world
        .application
        .with_host_advancement(|phase| {
            let execution = phase.borrowed_request_for_test();
            let selected = world.selected_product(&phase.test_host_request());
            world
                .application
                .runtime
                .primary_graph()
                .unwrap()
                .integration_handle()
                .with_runtime(&execution, |runtime| {
                    let maximum = 1_u64 << 40;
                    let mut admission = InvalidationEditAdmission::new(CompanionPreflightBudget {
                        maximum_work_visits: maximum,
                        maximum_preparation_bytes: 0,
                    });
                    admission.charge_external_work(maximum - 1).unwrap();
                    assert_eq!(
                        fact.source_currentness_in(
                            runtime,
                            &execution,
                            selected.application_basis().snapshot_handle(),
                            &mut admission
                        ),
                        Err(CompanionPreflightStop::WorkExhaustedWithoutRequirement { maximum }),
                        "a guarded source denial retains the exact u64 allowance"
                    );
                });
        })
        .expect("the fixture read borrows its installed phase");
}
