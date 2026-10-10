use super::super::{PricingCertificationBasisEntry, PricingWorkloadCertificationBundle};

impl PricingWorkloadCertificationBundle {
    pub(in crate::harness::tests) fn failure_localization_basis_entries(
        &self,
    ) -> Vec<PricingCertificationBasisEntry> {
        vec![
            PricingCertificationBasisEntry::debug(
                "routing_failure_class",
                &self.hostile_failure.failure_class,
            ),
            PricingCertificationBasisEntry::new(
                "source_commit",
                &self.hostile_failure.source_commit,
            ),
            PricingCertificationBasisEntry::new(
                "source_snapshot",
                &self.hostile_failure.source_snapshot,
            ),
            PricingCertificationBasisEntry::new("shock_commit", &self.provenance.shock_commit),
            PricingCertificationBasisEntry::new(
                "portfolio_positive_retail_delta_count",
                self.portfolio.positive_retail_delta_count,
            ),
            PricingCertificationBasisEntry::new("crisis_name", &self.crisis.crisis_name),
            PricingCertificationBasisEntry::new(
                "recommended_strategy",
                &self.strategy.recommended_strategy,
            ),
            PricingCertificationBasisEntry::new(
                "simulation_trace_count",
                self.simulation.iteration_traces.len(),
            ),
            PricingCertificationBasisEntry::debug("merge_denial_class", self.merge.denial_class),
            PricingCertificationBasisEntry::debug(
                "writeback_rejection",
                self.writeback.rejection_error_kind,
            ),
            PricingCertificationBasisEntry::debug("restart_error", self.restart_failure.error_kind),
        ]
    }

    pub(in crate::harness::tests) fn replay_failure_basis_entries(
        &self,
    ) -> Vec<PricingCertificationBasisEntry> {
        vec![
            PricingCertificationBasisEntry::debug(
                "hostile_error",
                self.hostile_failure.error_kind.clone(),
            ),
            PricingCertificationBasisEntry::debug(
                "hostile_failure",
                &self.hostile_failure.failure_class,
            ),
            PricingCertificationBasisEntry::new(
                "hostile_commit",
                &self.hostile_failure.source_commit,
            ),
            PricingCertificationBasisEntry::debug(
                "writeback_failure",
                self.writeback.rejection_error_kind,
            ),
            PricingCertificationBasisEntry::debug("restart_error", self.restart_failure.error_kind),
            PricingCertificationBasisEntry::new(
                "restart_mismatch_count",
                self.restart_failure.replay_mismatch_count,
            ),
        ]
    }

    pub(in crate::harness::tests) fn reference_workload_failure_basis_entries(
        &self,
    ) -> Vec<PricingCertificationBasisEntry> {
        vec![
            PricingCertificationBasisEntry::debug(
                "hostile_error",
                self.hostile_failure.error_kind.clone(),
            ),
            PricingCertificationBasisEntry::debug(
                "hostile_failure",
                &self.hostile_failure.failure_class,
            ),
            PricingCertificationBasisEntry::new(
                "hostile_commit",
                &self.hostile_failure.source_commit,
            ),
            PricingCertificationBasisEntry::new(
                "hostile_snapshot",
                &self.hostile_failure.source_snapshot,
            ),
            PricingCertificationBasisEntry::debug("restart_error", self.restart_failure.error_kind),
            PricingCertificationBasisEntry::new(
                "restart_mismatch_count",
                self.restart_failure.replay_mismatch_count,
            ),
            PricingCertificationBasisEntry::debug(
                "writeback_failure",
                self.writeback.rejection_error_kind,
            ),
            PricingCertificationBasisEntry::debug("merge_denial_class", self.merge.denial_class),
        ]
    }

    pub(in crate::harness::tests) fn core_summary_basis_entries(
        &self,
    ) -> Vec<PricingCertificationBasisEntry> {
        let mut basis = self.reference_workload_basis_entries();
        basis.extend(self.failure_localization_basis_entries());
        basis.extend([
            PricingCertificationBasisEntry::new("main_commit", &self.provenance.main_commit),
            PricingCertificationBasisEntry::new(
                "shock_delta",
                self.provenance.shock_delta_microunits,
            ),
            PricingCertificationBasisEntry::new("top_family", &self.crisis.top_impacted_family),
            PricingCertificationBasisEntry::new(
                "damage_materials",
                self.simulation
                    .ranked_materials_by_damage
                    .canonical_material_list(),
            ),
        ]);
        basis
    }
}
