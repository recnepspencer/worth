//! Real lineage page custody and its existing execution-memory refusal.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{self, correction_source};
use std::{num::NonZeroUsize, sync::Arc};

#[test]
fn correction_page_retains_all_owned_receipts_until_its_last_shared_owner() {
    let world = correction_source::install();
    let source = correction_source::edit(&world, "page-source", "edited-label");
    let prepared = correction_source::prepare(&world, &source, "page-reversal");
    let reversal = correction_source::committed(correction_source::publish(&world, prepared));
    let request = fixture::live_scope();
    world
        .application
        .with_application_advancement(&request, |phase| {
            let principal = fixture::authenticate_external(
                world.application.installed_schema(),
                "alice",
                std::time::Duration::from_secs(120),
                &request,
            );
            let target = world.selected_product(&phase.test_host_request());
            let selected = target
                .select_correction_source(
                    &phase,
                    &principal,
                    &request,
                    &source,
                    None,
                    NonZeroUsize::MIN,
                )
                .unwrap();
            let execution = phase
                .checked_execution_request_for(world.application.product_runtime())
                .unwrap();
            let worth_execution::ExecutionAllocationPolicy::Execution(payer) =
                execution.allocation_policy()
            else {
                panic!("the page must borrow its actual native execution payer")
            };
            let baseline = payer.charged_memory_bytes();
            let page = Arc::new(
                target
                    .correction_alternatives(
                        &crate::publication_boundary::program_publication_access(),
                        &phase,
                        &principal,
                        &request,
                        &selected,
                        None,
                        NonZeroUsize::MIN,
                        NonZeroUsize::MIN,
                        None,
                    )
                    .unwrap(),
            );
            assert_eq!(page.entries().len(), 1);
            assert_eq!(
                page.entries()[0].root(),
                &source.correction_source().coordinates()
            );
            assert_eq!(
                page.entries()[0].performed_commit(),
                reversal.commit_reference()
            );
            assert!(!page.entries()[0].performed_commit().parents.is_empty());
            assert!(
                page.resolution().is_some(),
                "the complete page also owns the active reversal's receipt copies"
            );
            assert!(page.continuation().is_none());
            assert_eq!(page.lineage_entries_examined(), 1);
            assert_eq!(page.consumption_entries_examined(), 0);
            let charged = payer.charged_memory_bytes();
            assert!(charged > baseline);
            let sibling = Arc::clone(&page);
            assert_eq!(
                payer.charged_memory_bytes(),
                charged,
                "shared page ownership cannot charge the same heaps twice"
            );
            drop(page);
            assert_eq!(
                payer.charged_memory_bytes(),
                charged,
                "root, entries and active-reversal metadata remain in page custody"
            );
            assert_eq!(
                sibling.entries()[0].performed_commit(),
                reversal.commit_reference()
            );
            drop(sibling);
            assert_eq!(
                payer.charged_memory_bytes(),
                baseline,
                "the last page owner releases every newly owned receipt and vector"
            );
        })
        .unwrap();
}

#[test]
fn correction_page_allocation_preserves_native_memory_refusal_and_releases_its_retry() {
    use crate::domain_computation::{
        primary_graph::application_contribution::{
            with_bootstrap_advancement, WorthQueryManagedComputationResourceDenial as Resource,
            WorthQueryMemoryLimitLevel as Level,
        },
        WorthQueryProviderSessionDenialKind as Provider,
    };
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::MIN, Some(1 << 20), Some(1_000_000)),
    );
    with_bootstrap_advancement(policy, |phase| {
        let execution = phase.execution_request();
        let worth_execution::ExecutionAllocationPolicy::Execution(payer) =
            execution.allocation_policy()
        else {
            panic!("bootstrap carries its actual execution payer")
        };
        let baseline = payer.charged_memory_bytes();
        let mut allocation = WorthQueryCorrectionHistoryPageAllocation::new(&execution).unwrap();
        let held = payer
            .reserve_memory(payer.available_memory_bytes().unwrap())
            .unwrap();
        let denial = allocation
            .reserve_projection_slots::<WorthQueryCanonicalCorrectionFact>(1)
            .unwrap_err();
        assert!(
            matches!(denial, Denial::ExecutionDenied(Ok(Provider::ExecutionResource {
            denial: Resource::MemoryLimit { requested, admitted, level: Level::Policy }, ..
        })) if requested > admitted),
            "preserve the actual native memory stop: {denial:?}"
        );
        drop(held);
        let entries = allocation
            .reserve_projection_slots::<WorthQueryCanonicalCorrectionFact>(1)
            .unwrap();
        assert!(payer.charged_memory_bytes() > baseline);
        drop(entries);
        drop(allocation);
        assert_eq!(payer.charged_memory_bytes(), baseline);
    })
    .unwrap();
}
