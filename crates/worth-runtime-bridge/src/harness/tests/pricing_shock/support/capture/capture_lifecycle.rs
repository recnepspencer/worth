use super::*;

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_discard_bundle(
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> PricingDiscardBundle {
    let scenario = generated_pricing_scenario();
    let mut truth = RelationalPricing::reference(execution, &scenario, None);
    let runtime = truth.runtime(RecordingSignalBridgeSink::default());
    let discard_session_identity =
        BridgePreviewSessionIdentity::admit_bridge_owned("pricing:preview-discard-churn");
    let session = truth.begin_preview(&runtime, discard_session_identity);
    let observer = session.liveness_observer();

    let updated_steel = truth.publish_main_cost(
        execution,
        PricingMaterial::Steel,
        scenario.live_main_steel_cost,
    );

    let live_main_route = runtime
        .route(updated_steel, execution)
        .expect("main branch should keep routing during speculative churn");
    let (_, speculative_eval) = session
        .compare_to_main(
            TruthBranchIdentity::from_relational_branch_id("main"),
            truth.rubber_read_packet(),
            execution,
        )
        .expect("preview should still see its pinned shock pricing");
    session.discard();

    let post_discard_main_eval = runtime
        .evaluate(
            BridgeTruthViewEvaluationRequest::for_branch_head(
                TruthBranchIdentity::from_relational_branch_id("main"),
            )
            .with_read_packet(truth.cost_packet(PricingMaterial::Steel)),
            execution,
        )
        .expect("main branch should still evaluate after discard");
    assert_eq!(
        post_discard_main_eval.snapshot_identity(),
        truth.main_snapshot()
    );
    assert_eq!(
        live_main_route.result().receipt().snapshot_identity(),
        truth.main_snapshot()
    );
    assert_eq!(speculative_eval.snapshot_identity(), truth.shock_snapshot());

    PricingDiscardBundle {
        live_main_snapshot: live_main_route
            .result()
            .receipt()
            .snapshot_identity()
            .canonical_label(),
        speculative_rubber_cost_cents: read_cost(&speculative_eval, execution)
            .parse()
            .expect("native pricing cost"),
        post_discard_main_snapshot: post_discard_main_eval.snapshot_identity().canonical_label(),
        post_discard_main_steel_cost_cents: read_cost(&post_discard_main_eval, execution)
            .parse()
            .expect("native pricing cost"),
        session_closed: observer.admit_active_session().is_none(),
    }
}

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_promotion_bundle(
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> PricingPromotionBundle {
    let scenario = generated_pricing_scenario();
    let mut truth = RelationalPricing::reference(execution, &scenario, None);
    let runtime = truth.runtime(RecordingSignalBridgeSink::default());
    let promotion_session_identity =
        BridgePreviewSessionIdentity::admit_bridge_owned("pricing:preview-promote-churn");
    let session = truth.begin_preview(&runtime, promotion_session_identity);
    let observer = session.liveness_observer();

    truth.publish_main_cost(
        execution,
        PricingMaterial::Rubber,
        scenario.interleaved_main_rubber_cost,
    );

    let (main_eval, speculative_eval) = session
        .compare_to_main(
            TruthBranchIdentity::from_relational_branch_id("main"),
            truth.rubber_read_packet(),
            execution,
        )
        .expect("the preview and the advanced main should both evaluate");
    let finished = session.finish();

    assert_eq!(main_eval.snapshot_identity(), truth.main_snapshot());
    assert_eq!(speculative_eval.snapshot_identity(), truth.shock_snapshot());

    PricingPromotionBundle {
        main_snapshot: main_eval.snapshot_identity().canonical_label(),
        speculative_snapshot: speculative_eval.snapshot_identity().canonical_label(),
        main_rubber_cost_cents: read_cost(&main_eval, execution)
            .parse()
            .expect("native pricing cost"),
        speculative_rubber_cost_cents: read_cost(&speculative_eval, execution)
            .parse()
            .expect("native pricing cost"),
        promotion_session_identity: finished.session_identity().clone(),
        session_closed: observer.admit_active_session().is_none(),
    }
}

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_fanout_bundle(
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> PricingFanoutBundle {
    let scenario = generated_pricing_scenario();
    let mut truth = RelationalPricing::reference(execution, &scenario, None);
    let sink = RecordingSignalBridgeSink::default();
    let runtime = truth.fanout_runtime(sink.clone());

    let first_route = runtime
        .route(truth.steel_commit.clone(), execution)
        .expect("first steel fanout route should succeed");

    let latest_commit = truth.publish_main_cost(
        execution,
        PricingMaterial::Steel,
        scenario.fanout_second_steel_cost,
    );

    let second_route = runtime
        .route(latest_commit.clone(), execution)
        .expect("second steel fanout route should succeed");
    let second_eval = runtime
        .evaluate_current(second_route.target(), execution)
        .expect("second steel fanout route should prepare evaluation");
    let branch_eval = runtime
        .evaluate(
            BridgeTruthViewEvaluationRequest::for_branch_head(
                TruthBranchIdentity::from_relational_branch_id("main"),
            )
            .with_read_packet(truth.cost_packet(PricingMaterial::Steel)),
            execution,
        )
        .expect("main branch should evaluate after repeated steel churn");

    let route_records = runtime.diagnostics().route_records();
    let last_record = route_records
        .last()
        .expect("repeated steel churn should retain the last route record");
    let mut last_targets = last_record
        .invalidation_targets()
        .iter()
        .map(|target| target.signal_scope().to_owned())
        .collect::<Vec<_>>();
    last_targets.sort();

    assert_eq!(last_record.source_commit(), &latest_commit);
    assert_eq!(
        second_eval.snapshot().snapshot_identity(),
        truth.main_snapshot()
    );
    assert_eq!(branch_eval.snapshot_identity(), truth.main_snapshot());

    PricingFanoutBundle {
        total_deliveries: sink.deliveries().len(),
        first_delivery_target_count: first_route.result().receipt().delivered_target_count(),
        second_delivery_target_count: second_route.result().receipt().delivered_target_count(),
        second_source_commit: last_record.source_commit().clone(),
        second_snapshot: second_eval.snapshot().snapshot_identity().canonical_label(),
        branch_snapshot: branch_eval.snapshot_identity().canonical_label(),
        branch_steel_cost_cents: read_cost(&branch_eval, execution)
            .parse()
            .expect("native pricing cost"),
        retained_target_count: last_targets.len(),
        first_target: last_targets.first().cloned().unwrap_or_default(),
        last_target: last_targets.last().cloned().unwrap_or_default(),
    }
}

use super::super::relational_provenance::provenance_patch_fields;
use crate::harness::fixtures::RelationalFixture;
use crate::relational_source::relational_test_support::{
    aspect_key, field_key, string_aspect_field_patch, test_owner_begin_transaction_for_branch,
};
use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
use worth_relational::facade::{
    history::BranchId,
    identity::EntityId,
    transactions::{
        CommitResult, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
        WorkerIntentBatch,
    },
};

impl RelationalPricing {
    pub fn publish_main_cost(
        &mut self,
        execution: ExecutionRequest<'_, '_>,
        material: PricingMaterial,
        cost: i64,
    ) -> TruthCommitIdentity {
        let main = BranchId("main".to_owned());
        let published = update_component(
            execution,
            &self.native,
            &main,
            self.component(material),
            (cost, None),
        );
        let main_identity = self.native.runtime.branch_identity(&main).unwrap();
        let (_, basis) = self
            .native
            .source
            .observe_branch_basis(&main_identity)
            .unwrap();
        self.main_head = self
            .native
            .source
            .bind_branch_head_basis_for_bridge(&basis)
            .unwrap();
        TruthCommitIdentity::from_relational_commit_id(published.commit.commit_id.0)
    }
}

pub(super) fn update_component(
    execution: ExecutionRequest<'_, '_>,
    native: &RelationalFixture,
    branch: &BranchId,
    entity: EntityId,
    (cost, provenance): (i64, Option<[String; 7]>),
) -> CommitResult {
    let runtime = &native.runtime;
    let cost = cost.to_string();
    let fields = std::iter::once((aspect_key("cost"), field_key("usd"), cost.as_str()))
        .chain(provenance.iter().flat_map(provenance_patch_fields));
    let mut tx = test_owner_begin_transaction_for_branch(runtime, branch.clone());
    tx.push_batch(
        WorkerIntentBatch::new("update-cost").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: entity,
                fields: string_aspect_field_patch(fields),
            }),
        )),
        ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();
    tx.commit(&execution, runtime).unwrap()
}
