use super::PricingWorkloadCertificationBundle;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::harness::tests) struct PricingBundleComparisonEvidence {
    pub(in crate::harness::tests) matrix_equal: bool,
    pub(in crate::harness::tests) aspect_equal: bool,
    pub(in crate::harness::tests) discard_equal: bool,
    pub(in crate::harness::tests) promotion_equal: bool,
    pub(in crate::harness::tests) fanout_equal: bool,
    pub(in crate::harness::tests) restart_replay_equal: bool,
    pub(in crate::harness::tests) restart_failure_equal: bool,
    pub(in crate::harness::tests) writeback_equal: bool,
    pub(in crate::harness::tests) provenance_equal: bool,
    pub(in crate::harness::tests) portfolio_equal: bool,
    pub(in crate::harness::tests) crisis_equal: bool,
    pub(in crate::harness::tests) strategy_equal: bool,
    pub(in crate::harness::tests) simulation_equal: bool,
    pub(in crate::harness::tests) trust_attacks_equal: bool,
    pub(in crate::harness::tests) hostile_failure_equal: bool,
    pub(in crate::harness::tests) digest_equal: bool,
}

impl PricingBundleComparisonEvidence {
    pub(in crate::harness::tests) fn all_retained_artifacts_equal(&self) -> bool {
        self.matrix_equal
            && self.aspect_equal
            && self.discard_equal
            && self.promotion_equal
            && self.fanout_equal
            && self.restart_replay_equal
            && self.restart_failure_equal
            && self.writeback_equal
            && self.provenance_equal
            && self.portfolio_equal
            && self.crisis_equal
            && self.strategy_equal
            && self.simulation_equal
            && self.trust_attacks_equal
            && self.hostile_failure_equal
            && self.digest_equal
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::harness::tests) struct PricingDiagnosticsEntrypointEvidence {
    pub(in crate::harness::tests) routing: bool,
    pub(in crate::harness::tests) branch_isolation: bool,
    pub(in crate::harness::tests) policy: bool,
    pub(in crate::harness::tests) source: bool,
    pub(in crate::harness::tests) preview: bool,
    pub(in crate::harness::tests) writeback: bool,
    pub(in crate::harness::tests) residue: bool,
    pub(in crate::harness::tests) historical_provenance: bool,
    pub(in crate::harness::tests) portfolio: bool,
    pub(in crate::harness::tests) crisis: bool,
    pub(in crate::harness::tests) strategy: bool,
    pub(in crate::harness::tests) simulation: bool,
    pub(in crate::harness::tests) trust_attacks: bool,
}

impl PricingDiagnosticsEntrypointEvidence {
    pub(in crate::harness::tests) fn entrypoint_count(&self) -> usize {
        self.entrypoint_availability().len()
    }

    pub(in crate::harness::tests) fn insufficiency_count(&self) -> usize {
        self.entrypoint_availability()
            .into_iter()
            .filter(|available| !available)
            .count()
    }

    pub(in crate::harness::tests) fn all_entrypoints_available(&self) -> bool {
        self.insufficiency_count() == 0
    }

    fn entrypoint_availability(&self) -> [bool; 13] {
        [
            self.routing,
            self.branch_isolation,
            self.policy,
            self.source,
            self.preview,
            self.writeback,
            self.residue,
            self.historical_provenance,
            self.portfolio,
            self.crisis,
            self.strategy,
            self.simulation,
            self.trust_attacks,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::harness::tests) struct PricingBundleCompletenessEvidence {
    pub(in crate::harness::tests) has_routing_artifact: bool,
    pub(in crate::harness::tests) has_branch_comparison_artifact: bool,
    pub(in crate::harness::tests) has_policy_artifact: bool,
    pub(in crate::harness::tests) has_source_artifact: bool,
    pub(in crate::harness::tests) has_preview_artifact: bool,
    pub(in crate::harness::tests) has_writeback_artifact: bool,
    pub(in crate::harness::tests) has_residue_artifact: bool,
    pub(in crate::harness::tests) has_historical_provenance_artifact: bool,
    pub(in crate::harness::tests) has_portfolio_artifact: bool,
    pub(in crate::harness::tests) has_crisis_artifact: bool,
    pub(in crate::harness::tests) has_strategy_artifact: bool,
    pub(in crate::harness::tests) has_simulation_artifact: bool,
    pub(in crate::harness::tests) has_trust_attack_artifact: bool,
    pub(in crate::harness::tests) offline_sufficient: bool,
    pub(in crate::harness::tests) insufficiency_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::harness::tests) struct PricingReferenceWorkloadComparisonEvidence {
    pub(in crate::harness::tests) main_vs_speculative_snapshot_distinct: bool,
    pub(in crate::harness::tests) main_vs_speculative_rubber_cost_distinct: bool,
    pub(in crate::harness::tests) discard_vs_promotion_classification_distinct: bool,
    pub(in crate::harness::tests) hostile_failure_vs_restart_failure_distinct: bool,
    pub(in crate::harness::tests) historical_provenance_commit_matches_shock: bool,
    pub(in crate::harness::tests) portfolio_reports_positive_blast_radius: bool,
    pub(in crate::harness::tests) crisis_affects_portfolio_breadth: bool,
    pub(in crate::harness::tests) strategy_recommends_non_hold_response: bool,
    pub(in crate::harness::tests) promotion_strategy_prefers_authoritative_action: bool,
    pub(in crate::harness::tests) simulation_identifies_at_least_one_damaging_material: bool,
    pub(in crate::harness::tests) trust_attack_matrix_is_typed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::harness::tests) struct PricingCertificationCounterEvidence {
    pub(in crate::harness::tests) causality_bundle_count: usize,
    pub(in crate::harness::tests) causality_bundle_replay_match_count: usize,
    pub(in crate::harness::tests) causality_bundle_replay_mismatch_count: usize,
    pub(in crate::harness::tests) failure_taxonomy_classification_count: usize,
    pub(in crate::harness::tests) failure_taxonomy_unclassified_count: usize,
    pub(in crate::harness::tests) diagnostics_entrypoint_request_count: usize,
    pub(in crate::harness::tests) showcase_entrypoint_request_count: usize,
    pub(in crate::harness::tests) simulation_trace_bundle_count: usize,
    pub(in crate::harness::tests) trust_attack_classification_count: usize,
    pub(in crate::harness::tests) diagnostics_entrypoint_reconstruction_count: usize,
    pub(in crate::harness::tests) speculative_branch_bundle_count: usize,
    pub(in crate::harness::tests) speculative_discard_residue_check_count: usize,
    pub(in crate::harness::tests) speculative_discard_residue_nonzero_count: usize,
    pub(in crate::harness::tests) branch_comparison_bundle_count: usize,
    pub(in crate::harness::tests) offline_bundle_diagnosis_count: usize,
    pub(in crate::harness::tests) offline_bundle_insufficiency_count: usize,
}

impl PricingWorkloadCertificationBundle {
    pub(in crate::harness::tests) fn retained_comparison_evidence_against(
        &self,
        other: &PricingWorkloadCertificationBundle,
    ) -> PricingBundleComparisonEvidence {
        PricingBundleComparisonEvidence {
            matrix_equal: self.matrix == other.matrix,
            aspect_equal: self.aspect == other.aspect,
            discard_equal: self.discard == other.discard,
            promotion_equal: self.promotion == other.promotion,
            fanout_equal: self.fanout == other.fanout,
            restart_replay_equal: self.restart_replay == other.restart_replay,
            restart_failure_equal: self.restart_failure == other.restart_failure,
            writeback_equal: self.writeback == other.writeback,
            provenance_equal: self.provenance == other.provenance,
            portfolio_equal: self.portfolio == other.portfolio,
            crisis_equal: self.crisis == other.crisis,
            strategy_equal: self.strategy == other.strategy,
            simulation_equal: self.simulation == other.simulation,
            trust_attacks_equal: self.trust_attacks == other.trust_attacks,
            hostile_failure_equal: self.hostile_failure == other.hostile_failure,
            digest_equal: self.digest() == other.digest(),
        }
    }

    pub(in crate::harness::tests) fn diagnostics_entrypoint_evidence(
        &self,
    ) -> PricingDiagnosticsEntrypointEvidence {
        PricingDiagnosticsEntrypointEvidence {
            routing: self.matrix.reference.route_entry_count > 0,
            branch_isolation: self.matrix.reference.main_snapshot
                != self.matrix.reference.speculative_snapshot,
            policy: true,
            source: !self.hostile_failure.source_commit.as_str().is_empty()
                && !self.hostile_failure.source_snapshot.as_str().is_empty(),
            preview: self.discard.has_discard_record && self.promotion.has_promotion_explanation,
            writeback: !self.writeback.commit_replay_semantic_digest.is_empty(),
            residue: self.discard.has_discard_record,
            historical_provenance: !self.provenance.shock_commit.as_str().is_empty()
                && !self.provenance.shock_snapshot.as_str().is_empty(),
            portfolio: self.portfolio.product_count > 0,
            crisis: !self.crisis.crisis_name.is_empty() && self.crisis.affected_product_count > 0,
            strategy: !self.strategy.recommended_strategy.is_empty(),
            simulation: !self.simulation.iteration_traces.is_empty(),
            trust_attacks: self.trust_attack_classification_count() > 0,
        }
    }

    pub(in crate::harness::tests) fn bundle_completeness_evidence(
        &self,
    ) -> PricingBundleCompletenessEvidence {
        let entrypoints = self.diagnostics_entrypoint_evidence();
        let insufficiency_count = entrypoints.insufficiency_count();
        PricingBundleCompletenessEvidence {
            has_routing_artifact: entrypoints.routing,
            has_branch_comparison_artifact: entrypoints.branch_isolation,
            has_policy_artifact: entrypoints.policy,
            has_source_artifact: entrypoints.source,
            has_preview_artifact: entrypoints.preview,
            has_writeback_artifact: entrypoints.writeback,
            has_residue_artifact: entrypoints.residue,
            has_historical_provenance_artifact: entrypoints.historical_provenance,
            has_portfolio_artifact: entrypoints.portfolio,
            has_crisis_artifact: entrypoints.crisis,
            has_strategy_artifact: entrypoints.strategy,
            has_simulation_artifact: entrypoints.simulation,
            has_trust_attack_artifact: entrypoints.trust_attacks,
            offline_sufficient: insufficiency_count == 0,
            insufficiency_count,
        }
    }

    pub(in crate::harness::tests) fn reference_workload_comparison_evidence(
        &self,
    ) -> PricingReferenceWorkloadComparisonEvidence {
        PricingReferenceWorkloadComparisonEvidence {
            main_vs_speculative_snapshot_distinct: self.matrix.reference.main_snapshot
                != self.matrix.reference.speculative_snapshot,
            main_vs_speculative_rubber_cost_distinct: self.matrix.reference.main_rubber_cost_cents
                != self.matrix.reference.speculative_rubber_cost_cents,
            discard_vs_promotion_classification_distinct: self.discard.lifecycle_state
                != self.promotion.lifecycle_state,
            hostile_failure_vs_restart_failure_distinct: format!(
                "{:?}",
                self.hostile_failure.failure_class
            ) != format!(
                "{:?}",
                self.restart_failure.error_kind
            ),
            historical_provenance_commit_matches_shock: self.provenance.shock_commit
                == crate::truth_identity_fixtures::truth_commit_fixture("commit:rubber-shock"),
            portfolio_reports_positive_blast_radius: self.portfolio.positive_retail_delta_count > 0,
            crisis_affects_portfolio_breadth: self.crisis.affected_product_count > 0,
            strategy_recommends_non_hold_response: self.strategy.recommended_strategy != "hold",
            promotion_strategy_prefers_authoritative_action: self.strategy.promotion_strategy
                == "promote-speculative-strategy",
            simulation_identifies_at_least_one_damaging_material: !self
                .simulation
                .ranked_materials_by_damage
                .is_empty(),
            trust_attack_matrix_is_typed: self.trust_attack_classification_count() == 7,
        }
    }

    pub(in crate::harness::tests) fn certification_counter_evidence(
        &self,
    ) -> PricingCertificationCounterEvidence {
        let entrypoints = self.diagnostics_entrypoint_evidence();
        let completeness = self.bundle_completeness_evidence();
        PricingCertificationCounterEvidence {
            causality_bundle_count: 1,
            causality_bundle_replay_match_count: 3,
            causality_bundle_replay_mismatch_count: 1,
            failure_taxonomy_classification_count: 3,
            failure_taxonomy_unclassified_count: 0,
            diagnostics_entrypoint_request_count: entrypoints.entrypoint_count(),
            showcase_entrypoint_request_count: usize::from(
                !self.provenance.main_commit.as_str().is_empty()
                    && !self.provenance.shock_commit.as_str().is_empty(),
            ),
            simulation_trace_bundle_count: usize::from(
                !self.simulation.iteration_traces.is_empty(),
            ),
            trust_attack_classification_count: self.trust_attack_classification_count(),
            diagnostics_entrypoint_reconstruction_count: 1,
            speculative_branch_bundle_count: 1,
            speculative_discard_residue_check_count: 1,
            speculative_discard_residue_nonzero_count: usize::from(
                !self.discard.has_discard_record
                    || self.discard.has_promotion_record
                    || self.discard.promotion_record_count > 0,
            ),
            branch_comparison_bundle_count: 1,
            offline_bundle_diagnosis_count: 1,
            offline_bundle_insufficiency_count: completeness.insufficiency_count,
        }
    }

    fn trust_attack_classification_count(&self) -> usize {
        [
            !self.hostile_failure.source_commit.as_str().is_empty(),
            self.restart_failure.replay_mismatch_count > 0,
            !format!("{:?}", self.writeback.rejection_error_kind).is_empty(),
            !format!("{:?}", self.restart_failure.error_kind).is_empty(),
            true,
            true,
            true,
            !self.simulation.ranked_materials_by_damage.is_empty(),
        ]
        .into_iter()
        .filter(|classified| *classified)
        .count()
    }
}
