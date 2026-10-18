//! Actual Query admission and fresh disclosure for one real MODE2 completion.
use super::installation::{Application, OwnerSchema, ANCHOR};
use super::*;
use std::{num::NonZeroUsize, sync::Arc, time::Duration};
use worth_query_decl::facade::application_query::ApplicationQueryParameterSet;

pub(super) fn complete(runtime: &Application) -> Arc<WorthQueryOutputDemandSettlement> {
    let scope = primary_graph::tests::fixture::live_scope();
    let external = primary_graph::tests::fixture::authenticate_external(
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
                .execution_request_for(&runtime.product_runtime)
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
                            NonZeroUsize::new(1).unwrap(),
                            NonZeroUsize::new(100_000).unwrap(),
                            &scope,
                        ),
                    )
                    .unwrap();
                runtime
                    .execute_application_query_one_shot(&phase, plan)
                    .unwrap()
                    .into_admitted_disclosed()
                    .into_output_demand_source()
            };
            let mut demand = runtime
                .admit_required_output_demand::<RegionOutputFamily>(
                    &execution,
                    read_source(),
                    runtime.output_demand_resource_profile().limits(),
                )
                .unwrap();
            let disclosure = read_source();
            let advanced = runtime
                .advance_program_output_demand(
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
            assert_eq!(settlement.producer_contacts_in_this_demand(), 1);
            assert!(!settlement.is_stably_reused());
            settlement
        })
        .unwrap()
}

#[test]
fn real_mode2_mixed_query_publication_attests_neither_whole_nor_created_subset() {
    let runtime = installation::install();
    let settled = complete(&runtime);
    admission::refuse_mixed_and_created_subset(&runtime, &settled);
    birth::admit_genuine_all_create(&runtime);
}
