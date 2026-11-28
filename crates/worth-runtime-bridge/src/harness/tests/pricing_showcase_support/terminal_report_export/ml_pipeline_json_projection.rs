use super::super::super::pricing_support::PricingWorkloadCertificationBundle;
use serde_json::json;

impl PricingWorkloadCertificationBundle {
    pub(in crate::harness::tests) fn ml_pipeline_export_pretty_json(&self) -> String {
        serde_json::to_string_pretty(&self.ml_pipeline_export_json())
            .expect("ml pipeline export should serialize")
    }

    pub(in crate::harness::tests) fn ml_pipeline_export_json(&self) -> serde_json::Value {
        json!({
            "schema": "worth-runtime-bridge.pricing-showcase.ml-pipeline.v1",
            "bundle_digest": self.digest(),
            "showcase_artifact": self.showcase_artifact_json(),
            "counter_snapshot": self.counter_snapshot_json(),
            "suite_25": self.suite_25_artifact_json(),
            "suite_26": self.suite_26_artifact_json(),
            "suite_27": self.suite_27_artifact_json(),
            "lineage_provenance": {
                "reference_lineage": {
                    "source_branch": self.matrix.reference.source_branch.as_str(),
                    "source_commit": self.matrix.reference.source_commit.as_str(),
                    "route_snapshot": self.matrix.reference.route_snapshot.as_str(),
                    "main_snapshot": self.matrix.reference.main_snapshot.as_str(),
                    "speculative_truth_branch": self.matrix.reference.speculative_truth_branch.as_str(),
                    "speculative_signal_branch": self.matrix.reference.speculative_signal_branch.as_str(),
                    "speculative_snapshot": self.matrix.reference.speculative_snapshot.as_str(),
                    "evaluation_record_identity": self.matrix.reference.evaluation_record_identity.as_str(),
                    "evaluation_selector_identity": self.matrix.reference.evaluation_selector_identity.as_str(),
                },
                "route_and_aspect_lineage": {
                    "replay_source_commit": self.matrix.replay.source_commit.as_str(),
                    "replay_source_snapshot": self.matrix.replay.source_snapshot.as_str(),
                    "replay_route_identity": self.matrix.replay.route_identity.as_str(),
                    "replay_invalidation_identity": self.matrix.replay.invalidation_identity.as_str(),
                    "aspect_route_identity": self.aspect.route_identity.as_str(),
                    "aspect_source_branch": self.aspect.source_branch.as_str(),
                    "aspect_source_commit": self.aspect.source_commit.as_str(),
                    "aspect_registration_id": self.aspect.aspect_registration_id.as_str(),
                    "aspect_truth_surface_kind": format!("{:?}", self.aspect.truth_surface_kind),
                    "aspect_subscription_slice_kind": format!("{:?}", self.aspect.subscription_slice_kind),
                    "aspect_target_canonical_basis": self.aspect.target_canonical_basis,
                    "aspect_invalidation_target": self.aspect.invalidation_target,
                    "aspect_match_status": format!("{:?}", self.aspect.fine_grained_match_status),
                },
                "speculation_lifecycle_lineage": {
                    "discard_live_main_snapshot": self.discard.live_main_snapshot.as_str(),
                    "discard_post_discard_main_snapshot": self.discard.post_discard_main_snapshot.as_str(),
                    "promotion_main_snapshot": self.promotion.main_snapshot.as_str(),
                    "promotion_speculative_snapshot": self.promotion.speculative_snapshot.as_str(),
                    "promotion_session_identity": self.promotion.promotion_session_identity.as_str(),
                    "finished_preview_closed": self.promotion.session_closed,
                },
                "fanout_and_restart_lineage": {
                    "fanout_second_source_commit": self.fanout.second_source_commit.as_str(),
                    "fanout_second_snapshot": self.fanout.second_snapshot.as_str(),
                    "restart_source_commit": self.restart_replay.source_commit.as_str(),
                    "restart_source_snapshot": self.restart_replay.source_snapshot.as_str(),
                    "restart_route_identity": self.restart_replay.route_identity.as_str(),
                    "restart_invalidation_identity": self.restart_replay.invalidation_identity.as_str(),
                },
                "writeback_lineage": {
                    "writeback_family_kind": format!("{:?}", self.writeback.family_kind),
                    "writeback_strategy_class": format!("{:?}", self.writeback.strategy_class),
                    "writeback_commit_outcome_class": format!("{:?}", self.writeback.commit_outcome_class),
                    "writeback_committed_effect_intent_digest": self.writeback.committed_effect_intent_digest,
                },
                "historical_provenance": {
                    "main_commit": self.provenance.main_commit.as_str(),
                    "main_snapshot": self.provenance.main_snapshot.as_str(),
                    "main_regime": self.provenance.main_regime,
                    "shock_commit": self.provenance.shock_commit.as_str(),
                    "shock_snapshot": self.provenance.shock_snapshot.as_str(),
                    "shock_regime": self.provenance.shock_regime,
                    "shock_delta_microunits": self.provenance.shock_delta_microunits,
                    "shock_multiplier_per_mille": self.provenance.shock_multiplier_per_mille,
                    "representative_sku": self.provenance.representative_sku,
                },
                "hostile_and_trust_lineage": {
                    "hostile_error_kind": format!("{:?}", self.hostile_failure.error_kind),
                    "hostile_failure_class": format!("{:?}", self.hostile_failure.failure_class),
                    "hostile_source_commit": self.hostile_failure.source_commit.as_str(),
                    "hostile_source_snapshot": self.hostile_failure.source_snapshot.as_str(),
                    "replay_policy_error_kind": format!("{:?}", self.trust_attacks.replay_policy_error_kind),
                    "route_policy_error_kind": format!("{:?}", self.trust_attacks.route_policy_error_kind),
                },
                "causality": self.ml_pipeline_causality_digest_json(),
            },
            "lineage_provenance_edges": self.lineage_provenance_edges_json(),
            "simulation": {
                "branch_count": self.simulation.branch_count,
                "iterations_per_branch": self.simulation.iterations_per_branch,
                "ranked_materials_by_damage": self.simulation.ranked_materials_by_damage.material_names().collect::<Vec<_>>(),
                "material_summaries": self
                    .simulation
                    .material_summaries
                    .iter()
                    .map(|summary| json!({
                        "material": summary.material,
                        "branch_count": summary.branch_count,
                        "iterations_per_branch": summary.iterations_per_branch,
                        "mean_total_retail_delta_cents": summary.mean_total_retail_delta_cents,
                        "mean_shipping_delta_cents": summary.mean_shipping_delta_cents,
                        "mean_material_delta_cents": summary.mean_material_delta_cents,
                        "mean_margin_floor_breach_count": summary.mean_margin_floor_breach_count,
                        "mean_repricing_count": summary.mean_repricing_count,
                        "worst_branch_identity": summary.worst_branch_identity,
                        "worst_branch_mean_total_delta_cents": summary.worst_branch_mean_total_delta_cents,
                        "damage_score": summary.damage_score,
                    }))
                    .collect::<Vec<_>>(),
                "iteration_traces": self
                    .simulation
                    .iteration_traces
                    .iter()
                    .map(|trace| json!({
                        "material": trace.material,
                        "branch_identity": trace.branch_identity,
                        "iteration_index": trace.iteration_index,
                        "regime": trace.regime,
                        "event_kind": trace.event_kind,
                        "shock_multiplier_per_mille": trace.shock_multiplier_per_mille,
                        "baseline_total_retail_cents": trace.baseline_total_retail_cents,
                        "shocked_total_retail_cents": trace.shocked_total_retail_cents,
                        "total_retail_delta_cents": trace.total_retail_delta_cents,
                        "shipping_delta_cents": trace.shipping_delta_cents,
                        "material_delta_cents": trace.material_delta_cents,
                        "margin_floor_breach_count": trace.margin_floor_breach_count,
                        "repricing_count": trace.repricing_count,
                    }))
                    .collect::<Vec<_>>(),
            },
        })
    }

    fn ml_pipeline_causality_digest_json(&self) -> serde_json::Value {
        let suite_25 = self.suite_25_digest_evidence();
        json!({
            "suite_25_causality_digest": suite_25.causality_digest,
            "suite_25_routing_digest": suite_25.routing_digest,
            "suite_25_replay_digest": suite_25.replay_digest,
            "suite_25_discard_digest": suite_25.discard_digest,
            "suite_25_promotion_digest": suite_25.promotion_digest,
            "suite_25_fanout_digest": suite_25.fanout_digest,
            "suite_25_writeback_digest": suite_25.writeback_digest,
            "suite_25_historical_provenance_digest": suite_25.historical_provenance_digest,
            "bundle_digest": self.digest(),
        })
    }
}
