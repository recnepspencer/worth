use super::*;

fn capture_pricing_reference_bundle(
    runtime: &RuntimeBridge,
    truth: &RelationalPricing,
    preview_session_identity: BridgePreviewSessionIdentity,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> PricingReferenceBundle {
    let route = runtime
        .route(truth.steel_commit.clone(), execution)
        .expect("pricing reference route should succeed");
    let route_record = runtime
        .diagnostics()
        .route_record_for_source_commit(route.result().result_summary().source_commit())
        .expect("pricing reference route should retain its route record");
    let preview = truth.begin_preview(runtime, preview_session_identity);
    let (main_eval, speculative_eval) = preview
        .compare_to_main(
            TruthBranchIdentity::from_relational_branch_id("main"),
            truth.rubber_read_packet(),
            execution,
        )
        .expect("pricing main and preview should evaluate");

    PricingReferenceBundle {
        source_branch: route_record.source_branch().clone(),
        source_commit: route_record.source_commit().clone(),
        route_snapshot: route.result().receipt().snapshot_identity().clone(),
        delivered_target_count: route.result().receipt().delivered_target_count(),
        route_entry_count: route_record.entries().len(),
        evaluation_record_identity: main_eval.record().record_identity().clone(),
        evaluation_selector_identity: main_eval
            .record()
            .decision_log()
            .selector_identity()
            .clone(),
        main_snapshot: main_eval.snapshot_identity().clone(),
        main_rubber_cost_cents: read_cost(&main_eval, execution)
            .parse()
            .expect("native pricing cost"),
        speculative_truth_branch: preview.branches().relational().clone(),
        speculative_signal_branch: preview.branches().signal().clone(),
        speculative_commit: truth.shock_commit.clone(),
        speculative_snapshot: speculative_eval.snapshot_identity().clone(),
        speculative_rubber_cost_cents: read_cost(&speculative_eval, execution)
            .parse()
            .expect("native pricing cost"),
    }
}

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_aspect_bundle(
    policy: BridgeRuntimePolicy,
    truth: &RelationalPricing,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> PricingAspectBundle {
    let runtime = truth.aspect_runtime(policy);
    let route = runtime
        .route(truth.steel_commit.clone(), execution)
        .expect("aspect-aware pricing route should succeed");

    let route_record = runtime
        .diagnostics()
        .route_record_for_source_commit(route.result().result_summary().source_commit())
        .expect("aspect-aware pricing route should retain a route record");
    let explanation = runtime
        .diagnostics()
        .explain_route(route_record.route_identity())
        .expect("aspect-aware pricing route should be explainable");
    let entry = &explanation.route_entries()[0];

    PricingAspectBundle {
        route_identity: explanation.route_identity().clone(),
        snapshot: explanation.snapshot_identity().clone(),
        source_branch: route_record.source_branch().clone(),
        source_commit: route_record.source_commit().clone(),
        truth_surface_kind: entry.truth_surface_kind(),
        fine_grained_match_status: entry.fine_grained_match_status(),
        aspect_registration_id: entry
            .aspect_registration_id()
            .expect("aspect-aware route entry should retain the aspect registration id")
            .clone(),
        subscription_slice_kind: entry
            .subscription_slice_kind()
            .expect("aspect-aware route entry should retain the subscription slice kind")
            .clone(),
        target_canonical_basis: entry.target_canonical_basis().to_owned(),
        invalidation_target: explanation.invalidation_targets()[0]
            .signal_scope()
            .to_owned(),
    }
}

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_missing_snapshot_failure_bundle(
    truth: &RelationalPricing,
    policy: BridgeRuntimePolicy,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> PricingFailureBundle {
    let runtime = truth.fault_runtime(
        PricingFault::UnavailableSnapshot(truth.steel_snapshot().clone()),
        policy,
    );
    let error = runtime
        .route(truth.steel_commit.clone(), execution)
        .expect_err("pricing route should fail when the source snapshot is absent");
    let error_kind = match error {
        BridgeStandardRouteError::Delivery(error) => error.kind(),
        BridgeStandardRouteError::Route(error) => {
            panic!("missing snapshot should fail at delivery, not route planning: {error}")
        }
    };

    let retained_failure = runtime
        .diagnostics()
        .last_failure_record()
        .expect("pricing failure should be retained in diagnostics");

    PricingFailureBundle {
        error_kind,
        failure_class: retained_failure.failure_class().clone(),
        source_commit: retained_failure.source_commit().clone(),
        source_snapshot: retained_failure.source_snapshot().clone(),
    }
}

fn capture_pricing_replay_bundle(
    runtime: &RuntimeBridge,
    commit: TruthCommitIdentity,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> PricingReplayBundle {
    runtime
        .route(commit, execution)
        .expect("pricing replay control route should succeed");
    let canonical_record = runtime
        .diagnostics()
        .last_canonical_route_record()
        .expect("pricing route should retain a canonical replay record");
    let replay = runtime
        .replay_canonical_record(&canonical_record, execution)
        .expect("pricing route replay should preserve canonical main-branch truth");

    PricingReplayBundle {
        source_commit: replay.source_commit().clone(),
        source_snapshot: replay.source_snapshot().clone(),
        route_identity: replay.route_identity().clone(),
        invalidation_identity: replay.invalidation_identity().clone(),
    }
}

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_certification_matrix(
    policy: BridgeRuntimePolicy,
    truth: &RelationalPricing,
    preview_session_identity: BridgePreviewSessionIdentity,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> PricingCertificationMatrix {
    let runtime = truth.runtime_with_policy(RecordingSignalBridgeSink::default(), policy);

    PricingCertificationMatrix {
        reference: capture_pricing_reference_bundle(
            &runtime,
            truth,
            preview_session_identity,
            resource_request,
        ),
        replay: capture_pricing_replay_bundle(
            &runtime,
            truth.steel_commit.clone(),
            resource_request,
        ),
    }
}

use super::super::relational_provenance::{
    corrupt_provenance, provenance_aspects, provenance_values,
};
use super::super::route_and_runtime::pricing_aspect;
use super::capture_lifecycle::update_component;
use crate::harness::fixtures::RelationalFixture;
use crate::relational_source::relational_test_support::{
    aspect_key, changed_entities, field_key, fork_branch, single_string_aspect_field_patch,
    test_owner_begin_transaction_for_branch, AspectSchemaFixture, ENTITY_KIND,
};
use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
use worth_relational::facade::{
    history::BranchId,
    identity::{EntityId, PartitionId},
    symbols::ClientKey,
    transactions::{CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch},
};

impl RelationalPricing {
    pub fn new(execution: ExecutionRequest<'_, '_>, steel: i64, rubber: i64, shock: i64) -> Self {
        Self::publish(execution, [(steel, None), (rubber, None), (shock, None)])
    }

    pub fn reference(
        execution: ExecutionRequest<'_, '_>,
        scenario: &GeneratedPricingScenario,
        corrupt_shock: Option<(&str, String)>,
    ) -> Self {
        let mut shock = provenance_values(&scenario.commit_attributions["commit:rubber-shock"]);
        if let Some((field, value)) = corrupt_shock {
            corrupt_provenance(&mut shock, field, value);
        }
        Self::publish(
            execution,
            [
                (scenario.main_steel_cost, None),
                (
                    scenario.main_rubber_cost,
                    Some(provenance_values(
                        &scenario.commit_attributions["commit:rubber-main"],
                    )),
                ),
                (scenario.speculative_rubber_cost, Some(shock)),
            ],
        )
    }

    fn publish(
        execution: ExecutionRequest<'_, '_>,
        [steel, rubber, shock]: [(i64, Option<[String; 7]>); 3],
    ) -> Self {
        let main = BranchId("main".to_owned());
        let branch = BranchId("pricing-shock".to_owned());
        let native = RelationalFixture::attach(
            AspectSchemaFixture {
                entity_aspects: std::iter::once(pricing_aspect())
                    .chain(provenance_aspects())
                    .collect(),
                ..AspectSchemaFixture::default()
            }
            .build_runtime(),
            "pricing",
        );
        let runtime = &native.runtime;
        let source = &native.source;
        let steel_id = create_component(execution, &native, &main, "steel");
        let rubber_id = create_component(execution, &native, &main, "rubber");

        let steel_commit = update_component(execution, &native, &main, steel_id, steel);
        let main_identity = runtime.branch_identity(&main).unwrap();
        let (_, basis) = source.observe_branch_basis(&main_identity).unwrap();
        let steel_observation = source.retain_branch_basis_for_bridge(&basis).unwrap();

        let rubber_commit = update_component(execution, &native, &main, rubber_id, rubber);
        let (_, main_basis) = source.observe_branch_basis(&main_identity).unwrap();
        let main_head = source
            .bind_branch_head_basis_for_bridge(&main_basis)
            .unwrap();
        fork_branch(&execution, runtime, branch.clone(), &main);

        let shock_commit = update_component(execution, &native, &branch, rubber_id, shock);
        let shock_identity = runtime.branch_identity(&branch).unwrap();
        let (_, shock_basis) = source.observe_branch_basis(&shock_identity).unwrap();
        let shock_head = source
            .bind_branch_head_basis_for_bridge(&shock_basis)
            .unwrap();

        Self {
            native,
            steel: steel_id,
            rubber: rubber_id,
            steel_commit: TruthCommitIdentity::from_relational_commit_id(
                steel_commit.commit.commit_id.0,
            ),
            rubber_commit: TruthCommitIdentity::from_relational_commit_id(
                rubber_commit.commit.commit_id.0,
            ),
            shock_commit: TruthCommitIdentity::from_relational_commit_id(
                shock_commit.commit.commit_id.0,
            ),
            steel_observation,
            main_head,
            shock_head,
        }
    }
}

fn create_component(
    execution: ExecutionRequest<'_, '_>,
    native: &RelationalFixture,
    branch: &BranchId,
    name: &str,
) -> EntityId {
    let runtime = &native.runtime;
    let mut tx = test_owner_begin_transaction_for_branch(runtime, branch.clone());
    tx.push_batch(
        WorkerIntentBatch::new(name).push(MutationIntent::Create(CreateIntent::Entity(
            EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: ENTITY_KIND,
                client_key: ClientKey::raw(name),
                fields: single_string_aspect_field_patch(aspect_key("cost"), field_key("usd"), "0"),
            },
        ))),
        ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();
    changed_entities(&tx.commit(&execution, runtime).unwrap())[0]
}
