use super::*;
#[test]
fn a_dependent_reads_a_held_stable_join_at_the_current_head_once() {
    let _guard = checkpoint_recovery_test_guard();
    let (application, _) = limited_application(
        4 * 5 * primary_graph::required_ready_custody_bytes_for_test(),
        128 * 1_024 * 1_024,
        8,
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (a, b, mut c, mut d) = chain_with_unrelated!(application, request);
    change_root_input!(request, application, 2, 0x612_8900_u64);
    settled_in_one_advance!(d, request, "C holds a Stable Ready");
    let current = request
        .query(PlanarOutputRead {
            body_key: "anchor-c".to_owned(),
        })
        .execute()
        .unwrap();
    let root = current.observed_sources()[0].root_entity_for_test();
    assert!(application
        .stable_ready_sources_for_test()
        .iter()
        .any(|(candidate, _, _)| *candidate == root));
    change_root_input!(request, application, 5, 0x612_8901_u64);
    application.joined_stable_roots_for_test();
    settled_in_one_advance!(d, request, "the wave joins C's held Stable row");
    assert_eq!(
        application
            .joined_stable_roots_for_test()
            .iter()
            .filter(|candidate| **candidate == root)
            .count(),
        1
    );
    application.exhaust_next_source_delivery_capacity_for_test();
    let inexact = inexact_deliveries();
    // A World overwrites C's output after the join. Its held alias no longer
    // certifies that output, so this dependent must rebuild its retained source.
    {
        use worth_query_consumer_values::{PlanarDerivedOutput, PlanarOperation};
        let selected = request
            .query(PlanarRead {
                body_key: "anchor-c".to_owned(),
            })
            .execute()
            .unwrap();
        let outcome = request
            .mutate(PlanarEdit(PlanarMutation {
                scope_key: "anchor-c".to_owned(),
                operation: PlanarOperation::PublishDerivedOutput(PlanarDerivedOutput {
                    body_key: "anchor-c".to_owned(),
                    value: length(40),
                }),
            }))
            .expect_source(selected.observed_sources()[0].clone())
            .idempotency(&0x612_8902_u64)
            .execute_in_program::<program::ChainProgram>(
                &application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        assert!(matches!(outcome,worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::Committed {..}));
    }
    assert_eq!(
        inexact_deliveries() - inexact,
        1,
        "real ledger pressure evicted the derived index"
    );
    application.retained_source_selections_for_test();
    let before = primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test();
    let head = request.retain_read().unwrap().selected_commit().clone();
    let advanced = c.advance(&request);
    let reads = application.retained_source_selections_for_test();
    let selected = reads
        .iter()
        .filter(|(candidate, _, _)| *candidate == root)
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), 1, "one actual retained source read of C");
    assert_ne!(selected[0].1, head, "the stored source address is stale");
    assert_eq!(
        selected.iter().filter(|read| read.2 == head).count(),
        1,
        "exactly one source read at the current head"
    );
    assert_eq!(
        selected.iter().filter(|read| read.2 == read.1).count(),
        0,
        "no read at the stale address"
    );
    assert!(
        matches!(
            advanced,
            Ok(WorthQueryApplicationOutputDemandProgress::Settled(_))
        ),
        "the retained read settles: {:?}",
        advanced.as_ref().err()
    );
    let after = primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test();
    assert_eq!(
        after.get(&root).copied().unwrap_or(0) - before.get(&root).copied().unwrap_or(0),
        1
    );
    drop((a, b, c, d));
}
