use super::super::super::pricing_support::PricingWorkloadCertificationBundle;
use serde_json::json;

impl PricingWorkloadCertificationBundle {
    pub(in crate::harness::tests) fn showcase_artifact_json(&self) -> serde_json::Value {
        json!({
            "executive_summary": {
                "main_branch": self.matrix.reference.source_branch.as_str(),
                "speculative_truth_branch": self.matrix.reference.speculative_truth_branch.as_str(),
                "main_snapshot": self.matrix.reference.main_snapshot.as_str(),
                "speculative_snapshot": self.matrix.reference.speculative_snapshot.as_str(),
                "same_fork_basis_preserved": true,
                "shock_commit": self.provenance.shock_commit.as_str(),
                "shock_regime": self.provenance.shock_regime,
                "shock_multiplier_per_mille": self.provenance.shock_multiplier_per_mille,
                "main_vs_speculative_rubber_delta_cents":
                    self.matrix.reference.speculative_rubber_cost_cents
                    - self.matrix.reference.main_rubber_cost_cents,
                "fanout_target_count": self.fanout.second_delivery_target_count,
                "discard_zero_residue": self.discard.has_discard_record
                    && !self.discard.has_promotion_record
                    && self.discard.promotion_record_count == 0,
                "promotion_outcome": format!("{:?}", self.promotion.lifecycle_state),
                "writeback_commit_outcome": format!("{:?}", self.writeback.commit_outcome_class),
                "writeback_noop_outcome": format!("{:?}", self.writeback.noop_outcome_class),
            },
            "timeline": [
                {
                    "phase": "main_basis",
                    "branch": self.matrix.reference.source_branch.as_str(),
                    "commit": self.matrix.reference.source_commit.as_str(),
                    "snapshot": self.matrix.reference.main_snapshot.as_str(),
                    "meaning": "ordinary live route basis",
                },
                {
                    "phase": "historical_main_provenance",
                    "branch": self.matrix.reference.source_branch.as_str(),
                    "commit": self.provenance.main_commit.as_str(),
                    "snapshot": self.provenance.main_snapshot.as_str(),
                    "meaning": "retained main-branch pricing provenance",
                },
                {
                    "phase": "speculative_shock",
                    "branch": self.matrix.reference.speculative_truth_branch.as_str(),
                    "commit": self.provenance.shock_commit.as_str(),
                    "snapshot": self.provenance.shock_snapshot.as_str(),
                    "meaning": "branch-local crisis shock",
                },
                {
                    "phase": "main_interleaved_fanout",
                    "branch": self.matrix.reference.source_branch.as_str(),
                    "commit": self.fanout.second_source_commit.as_str(),
                    "snapshot": self.fanout.second_snapshot.as_str(),
                    "meaning": "live high-fanout main-branch continuation",
                },
            ],
            "branch_comparison": {
                "main_snapshot": self.matrix.reference.main_snapshot.as_str(),
                "speculative_snapshot": self.matrix.reference.speculative_snapshot.as_str(),
                "main_rubber_cost_cents": self.matrix.reference.main_rubber_cost_cents,
                "speculative_rubber_cost_cents": self.matrix.reference.speculative_rubber_cost_cents,
                "delta_rubber_cost_cents":
                    self.matrix.reference.speculative_rubber_cost_cents
                    - self.matrix.reference.main_rubber_cost_cents,
                "main_and_speculative_distinct": self.matrix.reference.main_snapshot
                    != self.matrix.reference.speculative_snapshot,
            },
            "portfolio_blast_radius": {
                "product_count": self.portfolio.product_count,
                "main_repricing_count": self.portfolio.main_repricing_count,
                "shock_repricing_count": self.portfolio.shock_repricing_count,
                "main_margin_floor_breach_count": self.portfolio.main_margin_floor_breach_count,
                "shock_margin_floor_breach_count": self.portfolio.shock_margin_floor_breach_count,
                "positive_retail_delta_count": self.portfolio.positive_retail_delta_count,
                "total_retail_delta_cents": self.portfolio.total_retail_delta_cents,
                "max_retail_delta_sku": self.portfolio.max_retail_delta_sku,
                "max_retail_delta_cents": self.portfolio.max_retail_delta_cents,
                "top_margin_erosion_family": self.portfolio.top_margin_erosion_family,
                "top_margin_erosion_cents": self.portfolio.top_margin_erosion_cents,
                "most_shipping_sensitive_family": self.portfolio.most_shipping_sensitive_family,
                "most_shipping_sensitive_delta_cents": self.portfolio.most_shipping_sensitive_delta_cents,
                "most_material_sensitive_family": self.portfolio.most_material_sensitive_family,
                "most_material_sensitive_delta_cents": self.portfolio.most_material_sensitive_delta_cents,
            },
            "multi_factor_crisis": {
                "crisis_name": self.crisis.crisis_name,
                "affected_product_count": self.crisis.affected_product_count,
                "main_total_retail_cents": self.crisis.main_total_retail_cents,
                "crisis_total_retail_cents": self.crisis.crisis_total_retail_cents,
                "total_retail_delta_cents": self.crisis.total_retail_delta_cents,
                "top_impacted_family": self.crisis.top_impacted_family,
                "top_impacted_family_delta_cents": self.crisis.top_impacted_family_delta_cents,
                "dominant_shock_material": self.crisis.dominant_shock_material,
                "dominant_shock_multiplier_per_mille": self.crisis.dominant_shock_multiplier_per_mille,
                "policy_pressure_family": self.crisis.policy_pressure_family,
                "policy_pressure_bps": self.crisis.policy_pressure_bps,
                "top_exposure_material": self.crisis.top_exposure_material,
                "top_exposure_material_delta_cents": self.crisis.top_exposure_material_delta_cents,
            },
            "strategy_comparison": {
                "hold_unprofitable_count": self.strategy.hold_unprofitable_count,
                "partial_absorb_unprofitable_count": self.strategy.partial_absorb_unprofitable_count,
                "targeted_reprice_positive_delta_count": self.strategy.targeted_reprice_positive_delta_count,
                "targeted_reprice_total_delta_cents": self.strategy.targeted_reprice_total_delta_cents,
                "hold_total_margin_delta_cents": self.strategy.hold_total_margin_delta_cents,
                "partial_absorb_total_margin_delta_cents": self.strategy.partial_absorb_total_margin_delta_cents,
                "targeted_reprice_margin_recovery_cents": self.strategy.targeted_reprice_margin_recovery_cents,
                "recommended_strategy": self.strategy.recommended_strategy,
                "recommendation_reason": self.strategy.recommendation_reason,
                "promotion_strategy": self.strategy.promotion_strategy,
            },
            "shock_simulation": {
                "branch_count": self.simulation.branch_count,
                "iterations_per_branch": self.simulation.iterations_per_branch,
                "ranked_materials_by_damage": self.simulation.ranked_materials_by_damage.material_names().collect::<Vec<_>>(),
                "top_damage_summary": self.simulation.material_summaries.first().map(|summary| json!({
                    "material": summary.material,
                    "mean_total_retail_delta_cents": summary.mean_total_retail_delta_cents,
                    "mean_shipping_delta_cents": summary.mean_shipping_delta_cents,
                    "mean_material_delta_cents": summary.mean_material_delta_cents,
                    "damage_score": summary.damage_score,
                    "worst_branch_identity": summary.worst_branch_identity,
                    "worst_branch_mean_total_delta_cents": summary.worst_branch_mean_total_delta_cents,
                })).unwrap_or_else(|| json!({})),
            },
            "retained_commit_explorer": {
                "commit:rubber-main": self.showcase_commit_explorer_json("commit:rubber-main")
                    .expect("main provenance commit should be present"),
                "commit:rubber-shock": self.showcase_commit_explorer_json("commit:rubber-shock")
                    .expect("shock provenance commit should be present"),
            },
            "trust_proof": {
                "replay_route_identity": self.matrix.replay.route_identity.as_str(),
                "restart_replay_route_identity": self.restart_replay.route_identity.as_str(),
                "hostile_failure_class": format!("{:?}", self.hostile_failure.failure_class),
                "restart_failure_kind": format!("{:?}", self.restart_failure.error_kind),
                "writeback_committed_effect_intent_digest": self.writeback.committed_effect_intent_digest,
                "bundle_digest": self.digest(),
                "suite_25": self.suite_25_artifact_json(),
                "suite_26": self.suite_26_artifact_json(),
                "suite_27": self.suite_27_artifact_json(),
            },
            "trust_attack_matrix": self.trust_attack_matrix_json(),
            "demo_flow": [
                "stabilize main live pricing world",
                "fork speculative crisis branch",
                "inspect split reality and retained shock lineage",
                "measure portfolio blast radius",
                "choose discard or promotion path",
                "replay and trust-check canonical evidence",
            ],
            "demo_artifact_family": {
                "control_digest": self.suite_25_digest_evidence().reference_workload_bundle_digest,
                "hostile_digest": self
                    .suite_26_digest_evidence()
                    .reference_workload_failure_bundle_digest,
                "certification_digest": self.suite_27_digest_evidence().certification_bundle_digest,
                "showcase_digest": self.digest(),
            }
        })
    }
}
