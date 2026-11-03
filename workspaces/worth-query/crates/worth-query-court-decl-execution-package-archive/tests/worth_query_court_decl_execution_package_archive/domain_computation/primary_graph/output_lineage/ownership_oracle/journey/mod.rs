//! Actual Query admission and fresh disclosure for one real MODE2 completion.

use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::planar_read::PlanarReadBinding;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::principal;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::principal::ConsumerPrincipalBinding;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAccessContext;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandAdvance;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandSettlement;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
use worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls;

use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::installation::Application;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::installation::OwnerSchema;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::installation::ANCHOR;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::*;
use std::sync::Arc;
use std::time::Duration;
use worth_query_decl::facade::application_query::ApplicationQueryParameterSet;
pub fn complete(runtime: &Application) -> Arc<WorthQueryOutputDemandSettlement> {
    complete_with_contacts(runtime, 1)
}
pub fn complete_with_contacts(
    runtime: &Application,
    contacts: usize,
) -> Arc<WorthQueryOutputDemandSettlement> {
    let scope = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    let external = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authentication::authenticate_external(
        runtime.installed_schema(),
        "alice",
        Duration::from_secs(60),
        &scope,
    );
    let binding = runtime
        .installed_schema()
        .installed_query_binding::<PlanarReadBinding<OwnerSchema>>()
        .unwrap();
    runtime
        .with_application_advancement(&scope, |phase| {
            let execution = phase
                .execution_request_for_for_test(&(*runtime.product_runtime_for_test()))
                .unwrap();
            let branch = runtime.current_world();
            let principal_binding = runtime
                .installed_schema()
                .principal_binding(ConsumerPrincipalBinding::reference())
                .unwrap();
            let selected = runtime.on_branch(branch).select(&phase).unwrap();
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &principal_binding,
                    &external,
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let entity = selected
                .resolve_entity(
                    &phase,
                    BodyKey::reference(),
                    ANCHOR.to_owned(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &entity);
            let schedule = runtime
                .plan_application_query_batch_reads(&binding, 2)
                .unwrap();
            let batch = runtime
                .application_query_batch_from_installed_reads(&[schedule])
                .unwrap();
            batch.freeze_planned_reads().unwrap();
            let read_source = || {
                let plan = runtime
                    .on_branch(branch)
                    .select(&phase)
                    .unwrap()
                    .admit_application_query(
                        &phase,
                        binding.query(),
                        &access,
                        ApplicationQueryParameterSet::new(),
                        WorthQueryProductQueryControls::new(
                            binding.limits().maximum_results(),
                            &scope,
                        ),
                    )
                    .unwrap();
                let read = runtime
                    .execute_application_query_one_shot_in_batch(&phase, plan, &batch)
                    .unwrap()
                    .into_result();
                read.into_admitted_disclosed().into_output_demand_source()
            };
            let mut demand = runtime
                .admit_required_output_demand_for_test::<RegionOutputFamily>(
                    &execution,
                    read_source(),
                    runtime.output_demand_resource_profile().limits(),
                )
                .unwrap();
            let disclosure = read_source();
            let advanced = runtime
                .advance_program_output_demand_for_test(
                    &execution,
                    &phase,
                    &mut demand,
                    &external,
                    &scope,
                    branch,
                    disclosure,
                )
                .unwrap();
            let WorthQueryOutputDemandAdvance::Settled(settlement) = advanced else {
                panic!("the synchronous real MODE2 handler must settle")
            };
            assert_eq!(settlement.producer_contacts_in_this_demand(), contacts);
            assert!(!settlement.is_stably_reused());
            settlement
        })
        .unwrap()
}


use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::*;
#[test]
fn real_mode2_mixed_query_publication_attests_neither_whole_nor_created_subset() {
    let runtime = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::installation::install();
    let settled = complete(&runtime);
    crate::domain_computation::primary_graph::output_lineage::ownership_oracle::admission::refuse_mixed_and_created_subset(&runtime, &settled);
    crate::domain_computation::primary_graph::output_lineage::ownership_oracle::birth::admit_genuine_all_create(&runtime);
}
