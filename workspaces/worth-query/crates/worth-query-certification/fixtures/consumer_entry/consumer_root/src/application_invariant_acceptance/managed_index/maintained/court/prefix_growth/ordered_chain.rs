//! Cross-scope birth and lifecycle, through the same ordinary SDK composition.
use super::super::super::{events::chain, ordered_refusing};

mod cost {
    use super::*;
    #[test]
    fn an_inherited_row_change_lends_every_scope_with_work_linear_in_the_chain() {
        let work = |count| {
            let with = chain::fan_out::rename_inherited(count, true);
            let without = chain::fan_out::rename_inherited(count, false);
            eprintln!("inherited rename at {count}: {without} without the loan, {with} with it");
            with.saturating_sub(without)
        };
        let (small, large) = (work(16), work(64));
        // The loan's own share: four times the chain is near four times the
        // invocations; a per-scope rediscovery of the shared row would be
        // near sixteen.
        assert!(
            large <= small.max(1) * 8,
            "the loan's SDK work: {small} -> {large}"
        );
    }
    #[test]
    fn ordered_chain_birth_work_follows_delta_and_height_instead_of_history() {
        let small = chain::growth(16);
        let large = chain::growth(256);
        assert!(
            small > 0 && large <= small * 3,
            "birth SDK work: {small} -> {large}"
        );
    }
}

#[test]
fn ordered_chain_fork_and_reuse_preserve_both_scope_keys_and_isolate_edits() {
    chain::forks();
}
#[test]
fn ordered_chain_missing_parent_builds_only_the_successor_and_never_reads_stale_material() {
    chain::eviction();
}
#[test]
fn ordered_chain_birth_cancellation_preserves_both_roots_and_releases_staging() {
    chain::cancellation();
}
#[test]
fn ordered_chain_refuses_an_unretained_predecessor_under_refuse_policy() {
    ordered_refusing::missing_parent();
}
#[test]
fn ordered_chain_foreign_scope_slots_cannot_cross_the_predecessor_door() {
    chain::foreign();
}

#[test]
fn ordered_chain_stronger_graph_laws_still_refuse_an_excess_member() {
    ordered_refusing::stronger_law();
}

#[test]
fn managed_index_facade_matches_sdk_invoice_and_counts_shared_retained_pages() {
    chain::evidence::run();
}

#[test]
fn unrelated_retained_index_queries_leave_commit_work_equal() {
    chain::retained_queries::run();
}
