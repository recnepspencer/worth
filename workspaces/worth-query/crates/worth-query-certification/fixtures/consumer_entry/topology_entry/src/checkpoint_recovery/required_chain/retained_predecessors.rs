//! Settled native evidence needs its real demand interest until descendants seal.
use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandDenial as DemandDenial,
    WorthQueryApplicationReadObservationDenial as ReadDenial,
    WorthQueryRequiredOutputPreparationDenial as Denial,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryProductBranchAdmissionDenial as AdmissionDenial,
    WorthQueryProductBranchAdmissionRefusalKind as AdmissionRefusal,
};
mod program;

#[test]
fn fixed_tree_retains_exact_predecessors_through_cancel_reclaim_and_fresh_resume() {
    let _guard = checkpoint_recovery_test_guard();
    // B interrupts before claiming A; C interrupts before claiming B. Both
    // are real installed current_output reads, not injected stale receipts.
    for cancelled_at in ["anchor-b", "anchor-c"] {
        let mut application =
            support::install_program::<program::Program>(None, Default::default());
        {
            take_all_decisions();
            let (scope, principal) = authenticate(&application);
            let cancellation = authentication::WorthQueryCancellationSource::new();
            let stopped_scope =
                authentication::WorthQueryRequestScope::new(None, cancellation.token());
            let stopped = application.request(&principal, &stopped_scope);
            let mut tree = stopped
                .advanced()
                .start_program_outputs::<program::Program, program::Root>(
                    &application,
                    PlanarOutputDemand::new("anchor-a"),
                    WorthQueryOutputDemandControls::host_policy(),
                )
                .unwrap();
            binding::cancel_during_next_decision(cancelled_at, cancellation);
            let stop = tree
                .settle(&application, &stopped)
                .err()
                .expect("the actual child cancels");
            assert!(
                matches!(&stop, Denial::Demand(DemandDenial::Demand(cause))
                if cause.kind() == WorthQueryOutputDemandDenialKind::Cancelled),
                "{stop:?}"
            );
            let root_commit = tree
                .settled_root_observation()
                .expect("A genuinely settled")
                .selected_commit()
                .clone();
            assert_eq!(
                decisions_snapshot()
                    .iter()
                    .filter(|(key, _)| key == cancelled_at)
                    .count(),
                1
            );

            let request = application.request(&principal, &scope);
            // Real caller observations force production reclamation to inspect
            // closed cached rows. Native evidence alone must not lose its posting.
            let mut loans = Vec::new();
            loop {
                match request.retain_current_read() {
                    Ok(observation) => loans.push(observation),
                    Err(ReadDenial::ProductSelection(AdmissionDenial::Refused(
                        AdmissionRefusal::ObservationCapacityExhausted,
                    ))) => break,
                    Err(other) => panic!("unexpected observation refusal: {other:?}"),
                }
            }
            assert!(!loans.is_empty());
            drop(loans);
            // No new demand, refreshed source, producer replay or receipt recovery.
            let fresh_scope = authentication::WorthQueryRequestScope::new(
                None,
                authentication::WorthQueryCancellationSource::new().token(),
            );
            let fresh = application.request(&principal, &fresh_scope);
            let settled = match tree.settle(&application, &fresh).unwrap() {
                WorthQueryApplicationProgramOutputProgress::Settled(value) => value,
                WorthQueryApplicationProgramOutputProgress::Pending => {
                    panic!("released room permits settlement")
                }
            };
            assert_eq!(settled.root_observation().selected_commit(), &root_commit);
            assert_eq!(settled.root_producer_contacts_in_this_demand(), 1);
            // The fixture's registered final and alternate producers remain real
            // sibling branches; the disputed dependent path is still A -> B -> C.
            assert_eq!(settled.output_count(), 4);
            let decisions = take_all_decisions();
            for key in ["anchor-b", "anchor-c"] {
                let inputs: Vec<_> = decisions
                    .iter()
                    .filter(|(node, _)| node == key)
                    .map(|(_, values)| values.clone())
                    .collect();
                assert_eq!(
                    inputs,
                    vec![vec![2]; if key == cancelled_at { 2 } else { 1 }]
                );
            }
            let current = fresh.retain_current_read().unwrap();
            fresh
                .at(&current)
                .require_current_program_output(&settled, NonZeroUsize::new(1_000_000).unwrap())
                .unwrap();
            for (key, expected) in [("anchor-a", 2), ("anchor-b", 2), ("anchor-c", 11)] {
                let actual = fresh
                    .query(PlanarOutputRead {
                        body_key: key.into(),
                    })
                    .execute()
                    .unwrap();
                assert_eq!(actual.rows()[0].value, length(expected));
            }
            // The completed original tree refuses further progression honestly.
            assert!(matches!(
                tree.advance(&application, &fresh),
                Err(Denial::Closed)
            ));
        }
        application
            .close_conditional_runtime()
            .expect("all genuine graph custody releases");
    }
}
