//! Frozen inspection proves own-row truth without preparing fresh work.
use super::*;

fn case(transitive: bool, changed_own_source: bool) {
    let application = support::install_program_with_seed::<application::Program>(
        None,
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard(),
        4096,
        128 * 1024 * 1024,
        1_000_000,
        |graph| {
            source_world::seed(graph);
            ring_world::seed::<1>(graph);
        },
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut owner = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .start_in_program::<application::Program, program::ChainRoot>(&application)
        .unwrap();
    let mut b = request
        .demand(ChainDemand("anchor-b".into()))
        .start_dependent_in_program::<application::Program, program::ChainConnection>(&application)
        .unwrap();
    let mut c = request
        .demand(ChainDemand("anchor-c".into()))
        .start_dependent_in_program::<application::Program, program::ChainConnection>(&application)
        .unwrap();
    let mut own = request
        .demand(PlanarOutputDemand::new("ring-0-source-b"))
        .start_in_program::<application::Program, program::ChainRoot>(&application)
        .unwrap();
    macro_rules! settle {
        ($demand:expr) => {
            (0..64).any(|_| {
                matches!(
                    $demand.advance(&request).unwrap(),
                    WorthQueryApplicationOutputDemandProgress::Settled(_)
                )
            })
        };
    }
    assert!(settle!(owner));
    assert!(settle!(b));
    assert!(settle!(c));
    assert!(settle!(own));
    let mut roots = Vec::new();
    for (rank, key) in ["anchor-a", "ring-0-source-b"].into_iter().enumerate() {
        let selected = request
            .query(PlanarRead {
                body_key: key.into(),
            })
            .execute()
            .unwrap();
        roots.push(selected.observed_sources()[0].root_entity_for_test());
        if rank == 0 || changed_own_source {
            request
                .mutate(PlanarSourceAdjustment {
                    scope_key: key.into(),
                    replacement_y: length(7),
                })
                .expect_source(selected.observed_sources()[0].clone())
                .idempotency(&(0x9176_7880_u64 + rank as u64))
                .execute_performed::<application::Program, program::ChainRoot>(
                    &application,
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
        }
    }
    application.delay_next_output_readiness_delivery_for_test();
    assert!(matches!(
        owner.advance(&request).unwrap(),
        WorthQueryApplicationOutputDemandProgress::Pending
    ));
    assert_eq!(
        application.undelivered_publication_counts_for_test(),
        (1, 1)
    );
    let reads = primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test();
    let own_contacts = application.producer_contacts_at_root_on_this_thread_for_test(roots[1]);
    let pending = if changed_own_source {
        matches!(
            own.advance(&request).unwrap(),
            WorthQueryApplicationOutputDemandProgress::Pending
        )
    } else if transitive {
        matches!(
            c.advance(&request).unwrap(),
            WorthQueryApplicationOutputDemandProgress::Pending
        )
    } else {
        matches!(
            b.advance(&request).unwrap(),
            WorthQueryApplicationOutputDemandProgress::Pending
        )
    };
    assert_eq!(
        application.entered_required_selection_is_frozen_for_test(),
        Some(true)
    );
    assert!(pending,
        "inspection cannot prove Current while its source changed or its prerequisite is Published and undelivered: direct={}, changed={}", !transitive, changed_own_source);
    assert_eq!(
        primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test(),
        reads,
        "frozen inspection performs no source execution"
    );
    assert_eq!(
        application.producer_contacts_at_root_on_this_thread_for_test(roots[1]),
        own_contacts
    );
    assert_eq!(
        application.undelivered_publication_counts_for_test(),
        (1, 1),
        "a foreign advance leaves the frozen publication unchanged"
    );
    if changed_own_source {
        let issued = application.queued_required_rows_for_root_for_test(roots[1]);
        assert!(
            !issued.is_empty(),
            "the changed caller's own issued row stays queued"
        );
        assert!(matches!(
            owner.advance(&request).unwrap(),
            WorthQueryApplicationOutputDemandProgress::Settled(_)
        ));
        assert_eq!(
            application.undelivered_publication_counts_for_test(),
            (0, 0)
        );
        assert!(matches!(
            own.advance(&request).unwrap(),
            WorthQueryApplicationOutputDemandProgress::Settled(_)
        ));
        assert_eq!(
            application.entered_required_selection_is_frozen_for_test(),
            Some(false)
        );
        assert_eq!(
            application.producer_contacts_at_root_on_this_thread_for_test(roots[1]) - own_contacts,
            1,
            "the changed row executes exactly once after delivery thaws the head"
        );
        assert!(application
            .queued_required_rows_for_root_for_test(roots[1])
            .is_empty());
    }
}

#[test]
fn direct_and_transitive_dependents_are_pending_on_a_frozen_head() {
    let _guard = checkpoint_recovery_test_guard();
    case(false, false);
    case(true, false);
}
#[test]
fn an_unrelated_changed_source_waits_without_execution_until_the_head_thaws() {
    let _guard = checkpoint_recovery_test_guard();
    case(false, true);
}
