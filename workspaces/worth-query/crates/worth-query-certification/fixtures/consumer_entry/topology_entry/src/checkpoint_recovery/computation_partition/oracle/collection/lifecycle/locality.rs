//! Fixed correction and collection work excludes unrelated source and canonical history.
use super::super::super::source_delivery::correction as commands;
use super::*;
use std::num::NonZeroUsize;
use worth_query_host::facade::{application_entry::*, primary_graph::*};

#[derive(Debug, Eq, PartialEq)]
struct Work {
    owner: OwnerCalls,
    contacts: u64,
    source_reads: [usize; 5],
    source_admission: u64,
    source_comparisons: u64,
    value_comparisons: u64,
    ancestry_links: usize,
    lineage_examined: usize,
    consumption_examined: usize,
}
#[test]
fn correction_and_family_work_excludes_ten_and_one_thousand_unrelated_histories() {
    let _guard = checkpoint_recovery_test_guard();
    assert_eq!(run(10), run(1_000));
}
fn run(count: usize) -> Work {
    let mut app = correction::install(count);
    let (scope, principal) = authenticate(&app);
    let request = app.request(&principal, &scope);
    population::history(&app, &request, count);
    let before = correction::settled(&app, &request, true);
    let untouched = (0..count)
        .map(|index| {
            let query = request
                .query(PlanarRead {
                    body_key: population::scope(index),
                })
                .execute()
                .unwrap();
            let root = query.observed_sources()[0].root_entity_for_test();
            (
                root,
                app.producer_contacts_at_root_on_this_thread_for_test(root),
            )
        })
        .collect::<Vec<_>>();
    let source = correction::source(&app, &request);
    let mut prefix = vec![request.retain_current_read().unwrap()];
    let changed = correction::settled(&app, &request, false);
    prefix.push(request.retain_current_read().unwrap());
    assert!(changed.members.is_empty());
    let selected = request
        .corrections()
        .select(&source, commands::limits())
        .unwrap();
    let ancestry_links = selected.ancestry_parent_links_followed();
    drop(selected);
    fact_verification_on_this_thread_for_test();
    room().clear();
    let contacts_before = app.producer_contacts_on_this_thread_for_test();
    let reversal = correction::reverse(&app, &request, &source);
    prefix.push(request.retain_current_read().unwrap());
    let restored = correction::settled(&app, &request, false);
    prefix.push(request.retain_current_read().unwrap());
    for adjacent in prefix.windows(2) {
        assert_ne!(adjacent[0].selected_commit(), adjacent[1].selected_commit());
    }
    let verification = fact_verification_on_this_thread_for_test();
    correction::assert_restored(&before, &restored);
    let contacts = app.producer_contacts_on_this_thread_for_test() - contacts_before;
    let runs = std::mem::take(&mut *room());
    assert_eq!(runs.len(), 1);
    let owner = runs[0].calls;
    assert!(owner.kernels > 0, "the real changed partition recomputes");
    for (root, before) in untouched {
        assert_eq!(
            app.producer_contacts_at_root_on_this_thread_for_test(root),
            before,
            "unrelated authored scopes receive no producer contact"
        );
    }
    let rows = request
        .query(source_query::SourceRead {
            body_key: SCOPE.to_owned(),
        })
        .execute()
        .unwrap();
    let reads = rows.receipt().inspect().read_work();
    assert_eq!(reads.truncation_count(), 0);
    assert_eq!(rows.rows()[0].total("even"), 13);
    let source_reads = [
        reads.examined_candidate_count(),
        reads.projected_record_count(),
        reads.projected_field_count(),
        reads.adjacency_list_read_count(),
        reads.edge_scan_count(),
    ];
    drop(rows);
    let selected = request
        .corrections()
        .select(&source, commands::limits())
        .unwrap();
    let page = request
        .corrections()
        .alternatives(&selected, commands::limits(), NonZeroUsize::MIN)
        .unwrap();
    assert_eq!(page.entries().len(), 1);
    assert!(page.continuation().is_none());
    let lineage_examined = page.lineage_entries_examined();
    let consumption_examined = page.consumption_entries_examined();
    assert_eq!(
        lineage_examined, 1,
        "only this lineage's actual reversal posting is examined"
    );
    drop((page, selected));
    relevant_ancestry(&app, &request, &source);
    drop((prefix, reversal, source, principal, scope));
    app.close_conditional_runtime().unwrap();
    Work {
        owner,
        contacts,
        source_reads,
        source_admission: verification.source_admission_visits,
        source_comparisons: verification.source_currentness_visits,
        value_comparisons: verification.value_comparison_visits,
        ancestry_links,
        lineage_examined,
        consumption_examined,
    }
}
fn relevant_ancestry(
    app: &correction::App,
    request: &Request<'_, '_, '_>,
    source: &WorthQueryApplicationCommitReceipt,
) {
    let mut observations = vec![request.retain_current_read().unwrap()];
    let before = request
        .corrections()
        .select(source, commands::limits())
        .unwrap()
        .ancestry_parent_links_followed();
    for step in 0..3 {
        edit(
            request,
            app,
            EntryEdit::new(
                "odd",
                99,
                crate::checkpoint_recovery::computation_partition::entry_edit::EntryFact::Value,
                (50.0 + step as f64).to_bits(),
            ),
            0x9184_1000 + step,
        );
        observations.push(request.retain_current_read().unwrap());
        let latest = observations.len() - 1;
        assert_ne!(
            observations[latest - 1].selected_commit(),
            observations[latest].selected_commit()
        );
    }
    let after = request
        .corrections()
        .select(source, commands::limits())
        .unwrap()
        .ancestry_parent_links_followed();
    assert_eq!(
        after,
        before + 3,
        "relevant ancestry work grows separately from unrelated catalog size"
    );
    let bounded =
        WorthQueryCorrectionHistoryLimits::new(Some(NonZeroUsize::MIN), NonZeroUsize::MIN);
    assert!(matches!(
        request.corrections().select(source, bounded),
        Err(WorthQueryApplicationCorrectionDenial::Source(
            WorthQueryCorrectionSelectionDenial::Ancestry(RuntimeWorldServiceDenial::Denied(
                ProductBranchAncestorSelectionDenial::History(
                    CompositeAncestorSelectionDenial::SelectionLimitReached
                )
            ))
        ))
    ));
    drop(observations);
}
