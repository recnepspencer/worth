//! Every refused charge in the actual drain leaves retriable custody.
//! The receipt is real; this fixture declares funded terminal rows and edges.
use super::super::PerformedOutputObligation;
use super::*;

type Receipt = crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;
fn admission(work: u64) -> InvalidationEditAdmission {
    InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: Some(work),
        maximum_preparation_bytes: None,
    })
}
fn source_owner(
) -> crate::domain_computation::primary_graph::output_lineage::SourceInvalidationOwner {
    use crate::domain_computation::execution_runtime::source_invalidation::{
        WorthQueryInvalidationResourceInstallation, WorthQueryInvalidationResources,
    };
    crate::domain_computation::primary_graph::output_lineage::SourceInvalidationOwner::new(
        WorthQueryInvalidationResources::install(
            WorthQueryInvalidationResourceInstallation::bounded(1_000_000, 1 << 20, 1 << 20, 1),
        )
        .unwrap(),
        1,
    )
}
fn queued(
    receipt: &Receipt,
) -> (
    WorthQueryOutputDemandRegistry,
    Vec<WorthQueryOutputDemandKey>,
    (usize, usize, usize),
) {
    let registry = WorthQueryOutputDemandRegistry::default();
    let mut state = registry.state.lock().unwrap();
    let (baseline_key, baseline) =
        super::reservation_baseline::install(&mut state, receipt.clone());
    let mut keys = Vec::new();
    for slot in [2, 3] {
        let key = key("paid-cleanup-entry", 1, slot);
        let mut row = record(
            occurrence(),
            DemandState::Failed(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::Closed,
                "",
            )),
            0,
        );
        row.performed_obligations.push(PerformedOutputObligation {
            source_commit: receipt
                .committed_product_publication()
                .composite_commit()
                .clone(),
            source: key.source.clone(),
        });
        state.obligation_reserved_bytes += row.obligation_reserved_bytes();
        if slot == 2 {
            row.prerequisites.push(Arc::new(baseline_key.clone()));
            state.required_reserved_bytes += row.prerequisites.capacity()
                * std::mem::size_of::<Arc<WorthQueryOutputDemandKey>>();
            state
                .records
                .get_mut(&baseline_key)
                .unwrap()
                .framework_required_count += 1;
        }
        state.records.insert(key.clone(), row);
        let member = state.prepare_required_member(&key).unwrap();
        state.install_required_member(member);
        assert!(state.defer_terminal_cleanup(&Arc::new(key.clone()), 0));
        keys.push(key);
    }
    drop(state);
    (registry, keys, baseline)
}
fn finished(
    registry: &WorthQueryOutputDemandRegistry,
    keys: &[WorthQueryOutputDemandKey],
    baseline: (usize, usize, usize),
) {
    let state = registry.state.lock().unwrap();
    assert!(state.pending_cleanup_head.is_none());
    for key in keys {
        assert!(
            !state.records.contains_key(key),
            "each declared terminal entry is retired"
        );
    }
    assert_eq!(
        state.records.cleanup_entries_for_test(),
        2,
        "the production loop retires exactly two entries, once"
    );
    assert_eq!(super::reservation_baseline::balances(&state), baseline);
    assert_eq!(state.records.len(), 1);
}
fn every_ceiling(bucket: u64) {
    let receipt=crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let _occurrence = occurrence();
    let owner = source_owner();
    let cost = crate::domain_computation::primary_graph::with_test_advancement(|_| {
        let (whole, keys, baseline) = queued(&receipt);
        let mut paid = admission(u64::MAX);
        whole
            .drain_terminal_cleanup_admitted(&owner, &mut paid)
            .unwrap();
        finished(&whole, &keys, baseline);
        paid.charged_work()
    });
    crate::domain_computation::primary_graph::drain_test_advancement_observations();
    assert!(cost > 2);
    // Independent requests cover every integer ceiling exactly once.
    for k in (bucket..cost).step_by(8) {
        crate::domain_computation::primary_graph::with_test_advancement(|_| {
            let (retry, keys, baseline) = queued(&receipt);
            let mut short = admission(k);
            let stop = retry
                .drain_terminal_cleanup_admitted(&owner, &mut short)
                .expect_err("each ceiling below the whole cost refuses");
            assert_eq!(
                stop.kind(),
                WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                "ceiling {k}"
            );
            assert!(short.charged_work() <= k);
            {
                let state = retry.state.lock().unwrap();
                assert!(
                    state.pending_cleanup_head.is_some(),
                    "ceiling {k} retains the unfinished queue"
                );
                assert!(state.records.cleanup_entries_for_test() < 2);
            }
            retry
                .drain_terminal_cleanup_admitted(&owner, &mut admission(cost))
                .unwrap();
            finished(&retry, &keys, baseline);
        });
        crate::domain_computation::primary_graph::drain_test_advancement_observations();
    }
}
macro_rules! cells { ($( $name:ident => $bucket:expr ),*) => { $( #[test] fn $name(){every_ceiling($bucket);} )* }; }
cells!(production_cleanup_resumes_0=>0,production_cleanup_resumes_1=>1,
    production_cleanup_resumes_2=>2,production_cleanup_resumes_3=>3,
    production_cleanup_resumes_4=>4,production_cleanup_resumes_5=>5,
    production_cleanup_resumes_6=>6,production_cleanup_resumes_7=>7);
