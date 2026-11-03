//! Caller cancellation during real invariant preparation must not fail a shared row.

use worth_query_testkit_execution_decl::domain_computation::primary_graph::output_lineage::ownership_oracle::commit_interruption::Interruption;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialStage;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandAdvance;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;
use worth_query_testkit_execution_decl::domain_computation::primary_graph::output_lineage::ownership_oracle::commit_interruption::source;

use worth_query_testkit_execution_decl::domain_computation::primary_graph::output_lineage::ownership_oracle::installation::Application;
use worth_query_testkit_execution_decl::domain_computation::primary_graph::output_lineage::ownership_oracle::installation::OwnerSchema;
use worth_query_testkit_execution_decl::domain_computation::primary_graph::output_lineage::ownership_oracle::installation::ANCHOR;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::*;

use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_decl::facade::application_query::ApplicationQueryParameterSet;
#[test]
fn cancelling_one_caller_during_producer_commit_keeps_the_other_callers_row_live() {
    let interruption = Interruption::default();
    let app = worth_query_testkit_execution_decl::domain_computation::primary_graph::output_lineage::ownership_oracle::installation::install_with_interruption(interruption.clone());
    let runtime = &app;
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, cancellation.token());
    let external = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authentication::authenticate_external(
        runtime.installed_schema(),
        "alice",
        Duration::from_secs(600),
        &scope,
    );
    let (mut survivor, stopped) = runtime
        .with_application_advancement(&scope, |phase| {
            use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
            let execution = phase
                .execution_request_for(&runtime.product_runtime)
                .unwrap();
            let admit = || {
                runtime
                    .admit_required_output_demand::<RegionOutputFamily>(
                        &execution,
                        source(runtime, &phase, &external, &scope),
                        runtime.output_demand_resource_profile().limits(),
                    )
                    .unwrap()
            };
            let mut cancelled = admit();
            let survivor = admit();
            let disclosure = source(runtime, &phase, &external, &scope);
            *interruption.lock().unwrap() = Some(cancellation);
            let stop = runtime
                .advance_program_output_demand(
                    &execution,
                    &phase,
                    &mut cancelled,
                    &external,
                    &scope,
                    runtime.current_world(),
                    disclosure,
                )
                .err()
                .expect("the cancelled producer commit must stop");
            assert!(
                interruption.lock().unwrap().is_none(),
                "the actual invariant prepared during producer commit"
            );
            assert_eq!(
                stop.commit_execution_denial(),
                Some((
                    WorthQueryApplicationCommitDenialStage::InvariantExecution,
                    Err(Control::Cancelled),
                ))
            );
            // A correctly typed local denial alone is insufficient if the shared
            // registry has already retained a row failure: observe the other caller first.
            (survivor, stop)
        })
        .unwrap();
    let fresh = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    let external = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authentication::authenticate_external(
        runtime.installed_schema(),
        "alice",
        Duration::from_secs(600),
        &fresh,
    );
    runtime
        .with_application_advancement(&fresh, |phase| {
            let execution = phase
                .execution_request_for(&runtime.product_runtime)
                .unwrap();
            let advanced = runtime
                .advance_program_output_demand(
                    &execution,
                    &phase,
                    &mut survivor,
                    &external,
                    &fresh,
                    runtime.current_world(),
                    source(runtime, &phase, &external, &fresh),
                )
                .expect(
                    "another caller can still claim the shared row after a mid-commit cancellation",
                );
            let WorthQueryOutputDemandAdvance::Settled(settled) = advanced else {
                panic!("the live row must settle by its real producer");
            };
            assert_eq!(settled.producer_contacts_in_this_demand(), 1);
            assert!(
                settled.application_commit_receipt().is_some(),
                "the surviving caller performs a real publication"
            );
        })
        .unwrap();
    assert_eq!(stopped.kind(), WorthQueryOutputDemandDenialKind::Cancelled);
}
