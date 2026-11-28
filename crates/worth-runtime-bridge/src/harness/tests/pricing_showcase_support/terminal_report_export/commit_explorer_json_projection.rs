use super::super::super::pricing_support::PricingWorkloadCertificationBundle;
use serde_json::json;

impl PricingWorkloadCertificationBundle {
    pub(in crate::harness::tests) fn showcase_commit_explorer_json(
        &self,
        commit_identity: &str,
    ) -> Option<serde_json::Value> {
        match commit_identity {
            "commit:rubber-main" => Some(json!({
                "branch": self.matrix.reference.source_branch.canonical_label(),
                "snapshot": self.provenance.main_snapshot.canonical_label(),
                "regime": self.provenance.main_regime,
                "external_factor_microunits": self.provenance.main_external_factor_microunits,
                "shock_delta_microunits": 0,
                "shock_multiplier_per_mille": 1000,
                "representative_sku": self.provenance.representative_sku,
                "representative_retail_price_cents": self.provenance.representative_retail_price_cents,
                "representative_shipping_cost_cents": self.provenance.representative_shipping_cost_cents,
                "representative_fuel_shipping_component_cents":
                    self.provenance.representative_fuel_shipping_component_cents,
            })),
            "commit:rubber-shock" => Some(json!({
                "branch": self.matrix.reference.speculative_truth_branch.canonical_label(),
                "snapshot": self.provenance.shock_snapshot.canonical_label(),
                "regime": self.provenance.shock_regime,
                "external_factor_microunits": self.provenance.shock_external_factor_microunits,
                "factor_delta_microunits": self.provenance.shock_factor_delta_microunits,
                "trend_delta_microunits": self.provenance.shock_trend_delta_microunits,
                "jump_delta_microunits": self.provenance.shock_jump_delta_microunits,
                "shock_delta_microunits": self.provenance.shock_delta_microunits,
                "shock_multiplier_per_mille": self.provenance.shock_multiplier_per_mille,
                "representative_sku": self.provenance.representative_sku,
                "representative_retail_price_cents": self.provenance.representative_retail_price_cents,
                "representative_shipping_cost_cents": self.provenance.representative_shipping_cost_cents,
                "representative_fuel_shipping_component_cents":
                    self.provenance.representative_fuel_shipping_component_cents,
            })),
            _ => None,
        }
    }
}
