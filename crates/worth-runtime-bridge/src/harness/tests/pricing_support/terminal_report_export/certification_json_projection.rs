use super::super::PricingWorkloadCertificationBundle;
use serde_json::json;

impl PricingWorkloadCertificationBundle {
    pub(in crate::harness::tests) fn trust_attack_matrix_json(&self) -> serde_json::Value {
        json!([
            {
                "attack": "missing_snapshot_basis",
                "classification": format!("{:?}", self.hostile_failure.failure_class),
                "result": "typed_fail_closed",
            },
            {
                "attack": "restart_replay_drift",
                "classification": format!("{:?}", self.restart_failure.error_kind),
                "result": "typed_fail_closed",
            },
            {
                "attack": "stale_historical_basis",
                "classification": format!("{:?}", self.restart_failure.error_kind),
                "result": "typed_fail_closed",
            },
            {
                "attack": "replay_policy_mismatch",
                "classification": format!("{:?}", self.trust_attacks.replay_policy_error_kind),
                "result": "typed_fail_closed",
            },
            {
                "attack": "route_policy_projection_conflict",
                "classification": format!("{:?}", self.trust_attacks.route_policy_error_kind),
                "result": "typed_fail_closed",
            },
            {
                "attack": "simulation_damaging_material_ranked",
                "classification": self
                    .simulation
                    .ranked_materials_by_damage
                    .first_material()
                    .map(str::to_owned)
                    .unwrap_or_default(),
                "result": "portfolio_risk_explained",
            }
        ])
    }

    pub(in crate::harness::tests) fn counter_snapshot_json(&self) -> serde_json::Value {
        let counters = self.certification_counter_evidence();
        json!({
            "causality_bundle_count": counters.causality_bundle_count,
            "causality_bundle_replay_match_count": counters.causality_bundle_replay_match_count,
            "causality_bundle_replay_mismatch_count": counters.causality_bundle_replay_mismatch_count,
            "failure_taxonomy_classification_count": counters.failure_taxonomy_classification_count,
            "failure_taxonomy_unclassified_count": counters.failure_taxonomy_unclassified_count,
            "diagnostics_entrypoint_request_count": counters.diagnostics_entrypoint_request_count,
            "showcase_entrypoint_request_count": counters.showcase_entrypoint_request_count,
            "simulation_trace_bundle_count": counters.simulation_trace_bundle_count,
            "trust_attack_classification_count": counters.trust_attack_classification_count,
            "diagnostics_entrypoint_reconstruction_count": counters.diagnostics_entrypoint_reconstruction_count,
            "speculative_branch_bundle_count": counters.speculative_branch_bundle_count,
            "branch_comparison_bundle_count": counters.branch_comparison_bundle_count,
            "offline_bundle_diagnosis_count": counters.offline_bundle_diagnosis_count,
            "offline_bundle_insufficiency_count": counters.offline_bundle_insufficiency_count,
        })
    }

    pub(in crate::harness::tests) fn suite_27_artifact_json(&self) -> serde_json::Value {
        let digests = self.suite_27_digest_evidence();
        json!({
            "certification_bundle_digest": digests.certification_bundle_digest,
            "bundle_completeness_report": self.bundle_completeness_report_json(),
            "diagnostics_entrypoint_matrix": self.diagnostics_entrypoint_matrix_json(),
            "counter_snapshot": self.counter_snapshot_json(),
            "reference_workload_bundle_digest": digests.reference_workload_bundle_digest,
            "reference_workload_bundle_comparison": self.reference_workload_bundle_comparison_json(),
        })
    }

    pub(in crate::harness::tests) fn digest(&self) -> String {
        self.retained_bundle_digest()
    }

    fn diagnostics_entrypoint_matrix_json(&self) -> serde_json::Value {
        let entrypoints = self.diagnostics_entrypoint_evidence();
        json!({
            "routing": entrypoints.routing,
            "branch_isolation": entrypoints.branch_isolation,
            "policy": entrypoints.policy,
            "source": entrypoints.source,
            "preview": entrypoints.preview,
            "writeback": entrypoints.writeback,
            "historical_provenance": entrypoints.historical_provenance,
            "portfolio": entrypoints.portfolio,
            "crisis": entrypoints.crisis,
            "strategy": entrypoints.strategy,
            "simulation": entrypoints.simulation,
            "trust_attacks": entrypoints.trust_attacks,
        })
    }

    fn bundle_completeness_report_json(&self) -> serde_json::Value {
        let completeness = self.bundle_completeness_evidence();
        json!({
            "has_routing_artifact": completeness.has_routing_artifact,
            "has_branch_comparison_artifact": completeness.has_branch_comparison_artifact,
            "has_policy_artifact": completeness.has_policy_artifact,
            "has_source_artifact": completeness.has_source_artifact,
            "has_preview_artifact": completeness.has_preview_artifact,
            "has_writeback_artifact": completeness.has_writeback_artifact,
            "has_historical_provenance_artifact": completeness.has_historical_provenance_artifact,
            "has_portfolio_artifact": completeness.has_portfolio_artifact,
            "has_crisis_artifact": completeness.has_crisis_artifact,
            "has_strategy_artifact": completeness.has_strategy_artifact,
            "has_simulation_artifact": completeness.has_simulation_artifact,
            "has_trust_attack_artifact": completeness.has_trust_attack_artifact,
            "offline_sufficient": completeness.offline_sufficient,
            "insufficiency_count": completeness.insufficiency_count,
        })
    }

    fn reference_workload_bundle_comparison_json(&self) -> serde_json::Value {
        let comparison = self.reference_workload_comparison_evidence();
        json!({
            "main_vs_speculative_snapshot_distinct": comparison.main_vs_speculative_snapshot_distinct,
            "main_vs_speculative_rubber_cost_distinct": comparison.main_vs_speculative_rubber_cost_distinct,
            "hostile_failure_vs_restart_failure_distinct": comparison.hostile_failure_vs_restart_failure_distinct,
            "historical_provenance_commit_matches_shock": comparison.historical_provenance_commit_matches_shock,
            "portfolio_reports_positive_blast_radius": comparison.portfolio_reports_positive_blast_radius,
            "crisis_affects_portfolio_breadth": comparison.crisis_affects_portfolio_breadth,
            "strategy_recommends_non_hold_response": comparison.strategy_recommends_non_hold_response,
            "promotion_strategy_prefers_authoritative_action": comparison.promotion_strategy_prefers_authoritative_action,
            "simulation_identifies_at_least_one_damaging_material": comparison.simulation_identifies_at_least_one_damaging_material,
            "trust_attack_matrix_is_typed": comparison.trust_attack_matrix_is_typed,
        })
    }
}
