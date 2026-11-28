use super::super::{PricingCertificationBasisEntry, PricingWorkloadCertificationBundle};

impl PricingWorkloadCertificationBundle {
    pub(in crate::harness::tests) fn suite_25_causality_basis_entries(
        &self,
    ) -> Vec<PricingCertificationBasisEntry> {
        vec![
            PricingCertificationBasisEntry::new(
                "source_commit",
                &self.matrix.reference.source_commit,
            ),
            PricingCertificationBasisEntry::new(
                "main_snapshot",
                &self.matrix.reference.main_snapshot,
            ),
            PricingCertificationBasisEntry::new(
                "speculative_snapshot",
                &self.matrix.reference.speculative_snapshot,
            ),
            PricingCertificationBasisEntry::new(
                "promotion_session_identity",
                &self.promotion.promotion_session_identity,
            ),
            PricingCertificationBasisEntry::new(
                "writeback_committed_effect_intent_digest",
                &self.writeback.committed_effect_intent_digest,
            ),
        ]
    }

    pub(in crate::harness::tests) fn suite_25_routing_basis_entries(
        &self,
    ) -> Vec<PricingCertificationBasisEntry> {
        vec![
            PricingCertificationBasisEntry::new(
                "route_entry_count",
                self.matrix.reference.route_entry_count,
            ),
            PricingCertificationBasisEntry::new(
                "delivered_target_count",
                self.matrix.reference.delivered_target_count,
            ),
            PricingCertificationBasisEntry::new(
                "route_identity",
                &self.matrix.replay.route_identity,
            ),
            PricingCertificationBasisEntry::new(
                "aspect_route_identity",
                &self.aspect.route_identity,
            ),
            PricingCertificationBasisEntry::new(
                "fanout_target_count",
                self.fanout.second_delivery_target_count,
            ),
        ]
    }

    pub(in crate::harness::tests) fn suite_25_explanation_basis_entries(
        &self,
    ) -> Vec<PricingCertificationBasisEntry> {
        vec![
            PricingCertificationBasisEntry::new(
                "evaluation_record_identity",
                &self.matrix.reference.evaluation_record_identity,
            ),
            PricingCertificationBasisEntry::new(
                "evaluation_selector_identity",
                &self.matrix.reference.evaluation_selector_identity,
            ),
            PricingCertificationBasisEntry::new("shock_commit", &self.provenance.shock_commit),
            PricingCertificationBasisEntry::new("shock_regime", &self.provenance.shock_regime),
            PricingCertificationBasisEntry::new(
                "shock_multiplier_per_mille",
                self.provenance.shock_multiplier_per_mille,
            ),
        ]
    }

    pub(in crate::harness::tests) fn suite_25_replay_basis_entries(
        &self,
    ) -> Vec<PricingCertificationBasisEntry> {
        vec![
            PricingCertificationBasisEntry::new(
                "replay_route_identity",
                &self.matrix.replay.route_identity,
            ),
            PricingCertificationBasisEntry::new(
                "replay_invalidation_identity",
                &self.matrix.replay.invalidation_identity,
            ),
            PricingCertificationBasisEntry::new(
                "restart_route_identity",
                &self.restart_replay.route_identity,
            ),
            PricingCertificationBasisEntry::new(
                "restart_invalidation_identity",
                &self.restart_replay.invalidation_identity,
            ),
        ]
    }

    pub(in crate::harness::tests) fn reference_workload_basis_entries(
        &self,
    ) -> Vec<PricingCertificationBasisEntry> {
        [
            self.suite_25_causality_basis_entries(),
            self.suite_25_routing_basis_entries(),
            self.suite_25_explanation_basis_entries(),
            self.suite_25_replay_basis_entries(),
            self.suite_25_discard_basis_entries(),
            self.suite_25_promotion_basis_entries(),
            self.suite_25_fanout_basis_entries(),
            self.suite_25_writeback_basis_entries(),
            self.suite_25_historical_provenance_basis_entries(),
            self.reference_workload_summary_basis_entries(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    pub(in crate::harness::tests) fn suite_25_discard_basis_entries(
        &self,
    ) -> Vec<PricingCertificationBasisEntry> {
        vec![
            PricingCertificationBasisEntry::new(
                "live_main_snapshot",
                &self.discard.live_main_snapshot,
            ),
            PricingCertificationBasisEntry::new(
                "post_discard_main_snapshot",
                &self.discard.post_discard_main_snapshot,
            ),
            PricingCertificationBasisEntry::debug("lifecycle_state", self.discard.lifecycle_state),
            PricingCertificationBasisEntry::new(
                "discard_record_count",
                self.discard.discard_record_count,
            ),
            PricingCertificationBasisEntry::new(
                "promotion_record_count",
                self.discard.promotion_record_count,
            ),
        ]
    }

    pub(in crate::harness::tests) fn suite_25_promotion_basis_entries(
        &self,
    ) -> Vec<PricingCertificationBasisEntry> {
        vec![
            PricingCertificationBasisEntry::new("main_snapshot", &self.promotion.main_snapshot),
            PricingCertificationBasisEntry::new(
                "speculative_snapshot",
                &self.promotion.speculative_snapshot,
            ),
            PricingCertificationBasisEntry::debug(
                "lifecycle_state",
                self.promotion.lifecycle_state,
            ),
            PricingCertificationBasisEntry::new(
                "promotion_session_identity",
                &self.promotion.promotion_session_identity,
            ),
            PricingCertificationBasisEntry::new(
                "authoritative_artifact_digest",
                &self.promotion.authoritative_artifact_digest,
            ),
        ]
    }

    pub(in crate::harness::tests) fn suite_25_fanout_basis_entries(
        &self,
    ) -> Vec<PricingCertificationBasisEntry> {
        vec![
            PricingCertificationBasisEntry::new("total_deliveries", self.fanout.total_deliveries),
            PricingCertificationBasisEntry::new(
                "second_delivery_target_count",
                self.fanout.second_delivery_target_count,
            ),
            PricingCertificationBasisEntry::new(
                "second_source_commit",
                &self.fanout.second_source_commit,
            ),
            PricingCertificationBasisEntry::new("branch_snapshot", &self.fanout.branch_snapshot),
            PricingCertificationBasisEntry::new(
                "retained_target_count",
                self.fanout.retained_target_count,
            ),
        ]
    }

    pub(in crate::harness::tests) fn suite_25_writeback_basis_entries(
        &self,
    ) -> Vec<PricingCertificationBasisEntry> {
        vec![
            PricingCertificationBasisEntry::debug("family_kind", self.writeback.family_kind),
            PricingCertificationBasisEntry::debug("strategy_class", self.writeback.strategy_class),
            PricingCertificationBasisEntry::debug(
                "commit_outcome",
                self.writeback.commit_outcome_class,
            ),
            PricingCertificationBasisEntry::debug(
                "noop_outcome",
                self.writeback.noop_outcome_class,
            ),
            PricingCertificationBasisEntry::new(
                "committed_effect_intent_digest",
                &self.writeback.committed_effect_intent_digest,
            ),
            PricingCertificationBasisEntry::new(
                "retried_effect_intent_digest",
                &self.writeback.retried_effect_intent_digest,
            ),
        ]
    }

    pub(in crate::harness::tests) fn suite_25_historical_provenance_basis_entries(
        &self,
    ) -> Vec<PricingCertificationBasisEntry> {
        vec![
            PricingCertificationBasisEntry::new("main_commit", &self.provenance.main_commit),
            PricingCertificationBasisEntry::new("main_snapshot", &self.provenance.main_snapshot),
            PricingCertificationBasisEntry::new("shock_commit", &self.provenance.shock_commit),
            PricingCertificationBasisEntry::new("shock_snapshot", &self.provenance.shock_snapshot),
            PricingCertificationBasisEntry::new("shock_regime", &self.provenance.shock_regime),
            PricingCertificationBasisEntry::new(
                "shock_delta",
                self.provenance.shock_delta_microunits,
            ),
            PricingCertificationBasisEntry::new(
                "representative_sku",
                &self.provenance.representative_sku,
            ),
        ]
    }

    fn reference_workload_summary_basis_entries(&self) -> Vec<PricingCertificationBasisEntry> {
        vec![
            PricingCertificationBasisEntry::new(
                "aspect_registration_id",
                &self.aspect.aspect_registration_id,
            ),
            PricingCertificationBasisEntry::debug(
                "discard_lifecycle",
                self.discard.lifecycle_state,
            ),
            PricingCertificationBasisEntry::debug(
                "promotion_lifecycle",
                self.promotion.lifecycle_state,
            ),
            PricingCertificationBasisEntry::new(
                "fanout_second_commit",
                &self.fanout.second_source_commit,
            ),
            PricingCertificationBasisEntry::new(
                "restart_source_commit",
                &self.restart_replay.source_commit,
            ),
            PricingCertificationBasisEntry::debug(
                "writeback_commit",
                self.writeback.commit_outcome_class,
            ),
            PricingCertificationBasisEntry::new(
                "portfolio_product_count",
                self.portfolio.product_count,
            ),
            PricingCertificationBasisEntry::new("crisis_name", &self.crisis.crisis_name),
            PricingCertificationBasisEntry::new("strategy", &self.strategy.recommended_strategy),
            PricingCertificationBasisEntry::new(
                "simulation_trace_count",
                self.simulation.iteration_traces.len(),
            ),
            PricingCertificationBasisEntry::debug(
                "route_policy",
                self.trust_attacks.route_policy_error_kind,
            ),
        ]
    }
}
