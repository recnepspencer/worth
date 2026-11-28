use super::super::PricingWorkloadCertificationBundle;
use serde_json::json;

impl PricingWorkloadCertificationBundle {
    pub(in crate::harness::tests) fn suite_25_artifact_json(&self) -> serde_json::Value {
        let digests = self.suite_25_digest_evidence();
        json!({
            "causality_digest": digests.causality_digest,
            "routing_digest": digests.routing_digest,
            "explanation_digest": digests.explanation_digest,
            "replay_digest": digests.replay_digest,
            "discard_digest": digests.discard_digest,
            "promotion_digest": digests.promotion_digest,
            "fanout_digest": digests.fanout_digest,
            "writeback_digest": digests.writeback_digest,
            "historical_provenance_digest": digests.historical_provenance_digest,
            "reference_workload_bundle_digest": digests.reference_workload_bundle_digest,
            "terminal_export_basis": self.suite_25_terminal_export_basis_json(),
        })
    }

    pub(in crate::harness::tests) fn suite_26_artifact_json(&self) -> serde_json::Value {
        let digests = self.suite_26_digest_evidence();
        json!({
            "failure_digest": digests.failure_digest,
            "failure_localization_matrix": self.failure_localization_matrix_json(),
            "replay_failure_digest": digests.replay_failure_digest,
            "diagnostics_digest": digests.diagnostics_digest,
            "reference_workload_failure_bundle_digest": digests.reference_workload_failure_bundle_digest,
            "terminal_export_basis": {
                "replay_failure": self.replay_failure_basis_json(),
                "reference_workload_failure": self.reference_workload_failure_basis_json(),
            },
        })
    }

    fn suite_25_terminal_export_basis_json(&self) -> serde_json::Value {
        json!({
            "causality": {
                "source_commit": self.matrix.reference.source_commit.as_str(),
                "main_snapshot": self.matrix.reference.main_snapshot.as_str(),
                "speculative_snapshot": self.matrix.reference.speculative_snapshot.as_str(),
                "promotion_session_identity": self.promotion.promotion_session_identity.as_str(),
                "writeback_commit_replay_semantic_digest": self.writeback.commit_replay_semantic_digest,
            },
            "routing": {
                "route_entry_count": self.matrix.reference.route_entry_count,
                "delivered_target_count": self.matrix.reference.delivered_target_count,
                "route_identity": self.matrix.replay.route_identity.as_str(),
                "aspect_route_identity": self.aspect.route_identity.as_str(),
                "fanout_target_count": self.fanout.second_delivery_target_count,
            },
            "explanation": {
                "evaluation_record_identity": self.matrix.reference.evaluation_record_identity.as_str(),
                "evaluation_selector_identity": self.matrix.reference.evaluation_selector_identity.as_str(),
                "shock_commit": self.provenance.shock_commit.as_str(),
                "shock_regime": self.provenance.shock_regime,
                "shock_multiplier_per_mille": self.provenance.shock_multiplier_per_mille,
            },
            "replay": {
                "replay_route_identity": self.matrix.replay.route_identity.as_str(),
                "replay_invalidation_identity": self.matrix.replay.invalidation_identity.as_str(),
                "restart_route_identity": self.restart_replay.route_identity.as_str(),
                "restart_invalidation_identity": self.restart_replay.invalidation_identity.as_str(),
            },
            "reference_workload": self.reference_workload_terminal_export_basis_json(),
        })
    }

    fn reference_workload_terminal_export_basis_json(&self) -> serde_json::Value {
        json!({
            "ordinary_matrix": {
                "source_branch": self.matrix.reference.source_branch.as_str(),
                "source_commit": self.matrix.reference.source_commit.as_str(),
                "route_snapshot": self.matrix.reference.route_snapshot.as_str(),
                "main_snapshot": self.matrix.reference.main_snapshot.as_str(),
                "main_rubber_cost_cents": self.matrix.reference.main_rubber_cost_cents,
                "speculative_snapshot": self.matrix.reference.speculative_snapshot.as_str(),
                "speculative_rubber_cost_cents": self.matrix.reference.speculative_rubber_cost_cents,
                "replay_source_commit": self.matrix.replay.source_commit.as_str(),
                "replay_source_snapshot": self.matrix.replay.source_snapshot.as_str(),
            },
            "aspect_lane": {
                "route_identity": self.aspect.route_identity.as_str(),
                "aspect_registration_id": self.aspect.aspect_registration_id.as_str(),
                "target_canonical_basis": self.aspect.target_canonical_basis,
                "invalidation_target": self.aspect.invalidation_target,
                "truth_surface_kind": format!("{:?}", self.aspect.truth_surface_kind),
                "fine_grained_match_status": format!("{:?}", self.aspect.fine_grained_match_status),
                "subscription_slice_kind": format!("{:?}", self.aspect.subscription_slice_kind),
            },
            "portfolio_blast_radius": {
                "product_count": self.portfolio.product_count,
                "positive_retail_delta_count": self.portfolio.positive_retail_delta_count,
                "total_retail_delta_cents": self.portfolio.total_retail_delta_cents,
                "max_retail_delta_sku": self.portfolio.max_retail_delta_sku,
                "top_margin_erosion_family": self.portfolio.top_margin_erosion_family,
                "most_shipping_sensitive_family": self.portfolio.most_shipping_sensitive_family,
                "most_material_sensitive_family": self.portfolio.most_material_sensitive_family,
            },
            "crisis_lane": {
                "crisis_name": self.crisis.crisis_name,
                "affected_product_count": self.crisis.affected_product_count,
                "top_impacted_family": self.crisis.top_impacted_family,
                "dominant_shock_material": self.crisis.dominant_shock_material,
                "policy_pressure_family": self.crisis.policy_pressure_family,
                "top_exposure_material": self.crisis.top_exposure_material,
            },
            "strategy_lane": {
                "recommended_strategy": self.strategy.recommended_strategy,
                "promotion_strategy": self.strategy.promotion_strategy,
            },
            "simulation_lane": {
                "branch_count": self.simulation.branch_count,
                "iterations_per_branch": self.simulation.iterations_per_branch,
                "top_damage_material": self.simulation.ranked_materials_by_damage.first_material().unwrap_or_default(),
                "trace_count": self.simulation.iteration_traces.len(),
            },
        })
    }

    fn failure_localization_matrix_json(&self) -> serde_json::Value {
        json!({
            "routing_failure": {
                "class": format!("{:?}", self.hostile_failure.failure_class),
                "source_commit": self.hostile_failure.source_commit.as_str(),
                "source_snapshot": self.hostile_failure.source_snapshot.as_str(),
            },
            "historical_provenance_surface": {
                "shock_commit": self.provenance.shock_commit.as_str(),
                "shock_regime": self.provenance.shock_regime,
                "shock_delta_microunits": self.provenance.shock_delta_microunits,
                "representative_sku": self.provenance.representative_sku,
            },
            "portfolio_surface": {
                "product_count": self.portfolio.product_count,
                "positive_retail_delta_count": self.portfolio.positive_retail_delta_count,
                "max_retail_delta_sku": self.portfolio.max_retail_delta_sku,
            },
            "writeback_failure": {
                "error_kind": format!("{:?}", self.writeback.rejection_error_kind),
                "failure_class": format!("{:?}", self.writeback.rejection_failure_class),
            },
            "residue_surface": {
                "discard_record_count": self.discard.discard_record_count,
                "promotion_record_count": self.discard.promotion_record_count,
                "nonzero_residue_detected": false,
            },
            "replay_failure": {
                "error_kind": format!("{:?}", self.restart_failure.error_kind),
                "replay_mismatch_count": self.restart_failure.replay_mismatch_count,
            },
        })
    }

    fn replay_failure_basis_json(&self) -> serde_json::Value {
        json!({
            "routing_failure": {
                "error_kind": format!("{:?}", self.hostile_failure.error_kind),
                "failure_class": format!("{:?}", self.hostile_failure.failure_class),
                "source_commit": self.hostile_failure.source_commit.as_str(),
                "source_snapshot": self.hostile_failure.source_snapshot.as_str(),
            },
            "writeback_failure_kind": format!("{:?}", self.writeback.rejection_error_kind),
            "restart_failure": {
                "error_kind": format!("{:?}", self.restart_failure.error_kind),
                "replay_mismatch_count": self.restart_failure.replay_mismatch_count,
            },
        })
    }

    fn reference_workload_failure_basis_json(&self) -> serde_json::Value {
        json!({
            "hostile_failure": {
                "error_kind": format!("{:?}", self.hostile_failure.error_kind),
                "failure_class": format!("{:?}", self.hostile_failure.failure_class),
                "source_commit": self.hostile_failure.source_commit.as_str(),
                "source_snapshot": self.hostile_failure.source_snapshot.as_str(),
            },
            "restart_failure": {
                "error_kind": format!("{:?}", self.restart_failure.error_kind),
                "replay_mismatch_count": self.restart_failure.replay_mismatch_count,
            },
            "writeback_failure_kind": format!("{:?}", self.writeback.rejection_error_kind),
        })
    }
}
