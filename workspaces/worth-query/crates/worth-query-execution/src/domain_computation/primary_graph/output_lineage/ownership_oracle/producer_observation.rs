//! Real settled producer seeds travel with exact reads, never same-looking heads.
use super::*;
use installation::{Application, OwnerSchema, ANCHOR};
use primary_graph::application_output_demand::OriginalSettledProducerSource;
use std::{sync::Arc, time::Duration};
use worth_query_decl::facade::application_query::ApplicationQueryParameterSet;

#[test]
fn real_settlement_seed_preserves_producer_a_and_later_query_basis_b_without_read_custody() {
    let runtime = installation::install();
    let original = journey::complete(&runtime);
    let original_read = original.retained_read();
    let original_weak = Arc::downgrade(&original_read);
    let native = original
        .readiness_delivery()
        .unwrap()
        .causal_observation()
        .unwrap()
        .clone();
    let exact = original
        .application_commit_receipt()
        .unwrap()
        .exact_output_settlement()
        .unwrap()
        .clone();
    birth::admit_genuine_all_create(&runtime);
    let later = journey::complete_with_contacts(&runtime, 0);
    let read = later.retained_read();
    assert_ne!(
        read.selected_commit(),
        original_read.selected_commit(),
        "a legal unrelated publication really advanced the read basis"
    );
    assert!(later
        .readiness_delivery()
        .unwrap()
        .causal_observation()
        .unwrap()
        .same_observation(&native));
    let weak = Arc::downgrade(&read);
    let registry = runtime.application_query_basis_observer();
    let baseline = registry.observe().active();
    let observations = [
        observe(&runtime, &read, &exact, &native, false),
        observe(&runtime, &read, &exact, &native, true),
    ];
    assert_eq!(registry.observe().active(), baseline);
    drop(original_read);
    drop(original);
    drop(read);
    drop(later);
    assert!(original_weak.upgrade().is_none());
    assert!(
        weak.upgrade().is_none(),
        "opaque query evidence retains no operational settlement read"
    );
    for observation in observations {
        observation.close();
    }
    assert_eq!(registry.observe().active(), baseline);
}

fn observe(
    runtime: &Application,
    read: &Arc<WorthQueryApplicationReadObservation>,
    expected: &Arc<super::super::RecordedSettlementIdentity>,
    native: &worth_runtime_bridge::facade::BridgeConditionalCausalObservation,
    batched: bool,
) -> primary_graph::causal_inspection::WorthQueryApplicationQueryObservation {
    let request = primary_graph::tests::fixture::live_scope();
    let external = primary_graph::tests::fixture::authenticate_external(
        runtime.installed_schema(),
        "alice",
        Duration::from_secs(60),
        &request,
    );
    let binding = runtime
        .installed_schema()
        .installed_query_binding::<PlanarReadBinding<OwnerSchema>>()
        .unwrap();
    runtime
        .with_application_advancement(&request, |phase| {
            let execution = phase
                .execution_request_for(&runtime.product_runtime)
                .unwrap();
            let current = runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap();
            assert!(
                current.original_settled_producer().is_none(),
                "fresh changed/head selection carries no settlement seed"
            );
            let principal_binding = runtime
                .installed_schema()
                .principal_binding(ConsumerPrincipalBinding::reference())
                .unwrap();
            let principal = current
                .resolve_authenticated_principal(
                    &phase,
                    &principal_binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let entity = current
                .resolve_entity(
                    &phase,
                    BodyKey::reference(),
                    ANCHOR.to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &entity);
            let retained = runtime
                .select_application_read_observation(&phase, read)
                .unwrap();
            let alias = retained
                .retain_selection(&execution)
                .unwrap()
                .retain_application_read();
            let mut retained = runtime
                .select_application_read_observation(&phase, &alias)
                .unwrap();
            let batch = batched.then(|| {
                let schedule = runtime
                    .plan_application_query_batch_reads(&binding, 1)
                    .unwrap();
                let batch = runtime
                    .application_query_batch_from_installed_reads(&[schedule])
                    .unwrap();
                batch.freeze_planned_reads().unwrap();
                batch
            });
            let plan = match &batch {
                Some(batch) => current
                    .admit_retained_application_query_from_selection(
                        &phase,
                        &mut retained,
                        binding.query(),
                        &access,
                        ApplicationQueryParameterSet::new(),
                        WorthQueryProductQueryControls::new(
                            binding.limits().maximum_results(),
                            &request,
                        ),
                        batch,
                    )
                    .unwrap(),
                None => current
                    .admit_retained_application_query(
                        &phase,
                        retained,
                        binding.query(),
                        &access,
                        ApplicationQueryParameterSet::new(),
                        WorthQueryProductQueryControls::new(
                            binding.limits().maximum_results(),
                            &request,
                        ),
                    )
                    .unwrap(),
            };
            let result = match &batch {
                Some(batch) => runtime
                    .execute_application_query_one_shot_in_batch(&phase, plan, batch)
                    .unwrap()
                    .into_result(),
                None => runtime
                    .execute_application_query_one_shot(&phase, plan)
                    .unwrap(),
            };
            let source = result.result_set_observation().source();
            let seed = source
                .original_settled_producer()
                .expect("the actual retained read carries its original supplier");
            assert_eq!(seed.read().selected_commit(), read.selected_commit());
            let OriginalSettledProducerSource::Committed {
                settlement,
                publication,
                conditional,
                ..
            } = seed.source()
            else {
                panic!("actual performed supplier")
            };
            assert!(Arc::ptr_eq(settlement.as_ref().unwrap(), expected));
            assert_ne!(
                publication.as_ref().unwrap().selected_commit(),
                seed.read().selected_commit(),
                "the original producer publication A must not replace read B"
            );
            assert!(conditional.as_ref().unwrap().same_observation(native));
            assert!(seed.correspondence().native_witness_roles().len() > 0);
            assert!(std::ptr::eq(
                seed,
                result.observed_sources()[0]
                    .clone()
                    .retyped::<()>()
                    .original_settled_producer()
                    .unwrap()
            ));
            let source_identity = source.idempotency_identity();
            let basis = result.receipt().basis_identity().clone();
            let observation = result
                .into_admitted_disclosed()
                .into_output_demand_source()
                .retain_inspection_observation()
                .unwrap();
            // An ordinary query at exactly B has the same native meaning, but no
            // original settlement supplier may be inferred from that equality.
            let plain = runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap()
                .admit_application_query(
                    &phase,
                    binding.query(),
                    &access,
                    ApplicationQueryParameterSet::new(),
                    WorthQueryProductQueryControls::new(
                        binding.limits().maximum_results(),
                        &request,
                    ),
                )
                .unwrap();
            let plain = runtime
                .execute_application_query_one_shot(&phase, plain)
                .unwrap();
            assert_eq!(
                plain.receipt().basis_identity().selection(),
                basis.selection()
            );
            assert_eq!(
                plain
                    .result_set_observation()
                    .source()
                    .idempotency_identity(),
                source_identity
            );
            assert!(plain
                .result_set_observation()
                .source()
                .original_settled_producer()
                .is_none());
            observation
        })
        .unwrap()
}
