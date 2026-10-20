//! Actual Query admission and fresh disclosure for one real MODE2 completion.
use super::installation::{Application, OwnerSchema, ANCHOR};
use super::*;
use std::{sync::Arc, time::Duration};
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
            let schedule = runtime
                .plan_application_query_batch_reads(&binding, 2)
                .unwrap();
            let batch = runtime
                .application_query_batch_from_installed_reads(&[schedule])
                .unwrap();
            assert_eq!(batch.maximum_result_bytes_per_item(), None);
            batch.freeze_planned_reads().unwrap();
            let row_custody = std::cell::RefCell::new(Vec::new());
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
                            binding.limits().maximum_work(),
                            &scope,
                        ),
                    )
                    .unwrap();
                let (read, rows) = runtime
                    .execute_application_query_one_shot_in_batch(&phase, plan, &batch)
                    .unwrap()
                    .into_parts();
                assert_eq!(read.receipt().result_buffer().unwrap().limit_bytes(), None);
                assert!(batch.observe().retained_bytes() > 0);
                row_custody.borrow_mut().push(rows);
                read.into_admitted_disclosed().into_output_demand_source()
            };
            let mut demand = runtime
                .admit_required_output_demand::<RegionOutputFamily>(
                    &execution,
                    read_source(),
                    runtime.output_demand_resource_profile().limits(),
                )
                .unwrap();
            let disclosure = read_source();
            assert_eq!(row_custody.borrow().len(), 2);
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

#[test]
fn ordinary_profiles_publish_mode2_with_large_live_lineage_and_real_source_authority() {
    let runtime = installation::install();
    let profile = runtime.output_demand_resource_profile();
    assert_eq!(profile.limits().settlement_attempts(), None);
    assert_eq!(profile.registry_obligation_retained_bytes(), None);
    assert_eq!(profile.registry_record_retained_bytes(), None);
    assert_eq!(profile.registry_required_retained_bytes(), None);
    assert_eq!(profile.lineage_retained_bytes(), None);
    let bytes = 65 * 1024 * 1024;
    let ticket = runtime
        .primary_provider
        .graph
        .output_lineage
        .lock()
        .unwrap()
        .retention
        .reserve(bytes)
        .unwrap();
    let mut backing = Vec::<u8>::new();
    backing.try_reserve_exact(bytes as usize).unwrap();
    backing.resize(bytes as usize, 3);
    let original = Arc::new((backing, ticket));
    let clone = Arc::clone(&original);
    let settled = complete(&runtime);
    admission::refuse_mixed_and_created_subset(&runtime, &settled);
    let before = runtime.output_lineage_retained_bytes_for_test();
    assert!(
        before > bytes,
        "actual produced lineage coexists with original backing"
    );
    drop(original);
    assert_eq!(runtime.output_lineage_retained_bytes_for_test(), before);
    assert_eq!(clone.0[0], 3);
    drop(clone);
    assert_eq!(
        runtime.output_lineage_retained_bytes_for_test(),
        before - bytes
    );
    drop(settled);
    drop(runtime);
}
