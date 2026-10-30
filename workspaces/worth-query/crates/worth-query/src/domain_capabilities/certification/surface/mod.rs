use crate::domain_capabilities::aftermath::DOMAIN_CAPABILITY_AFTERMATH_CATEGORY_MODULE;
use crate::domain_capabilities::continuity::DOMAIN_CAPABILITY_CONTINUITY_CATEGORY_MODULE;
use crate::domain_capabilities::explanation::DOMAIN_CAPABILITY_EXPLANATION_CATEGORY_MODULE;
use crate::domain_capabilities::identity::{
    compose_certified_surface_row_digest, compose_public_surface_digest,
};
use crate::domain_capabilities::workflow::DOMAIN_CAPABILITY_WORKFLOW_CATEGORY_MODULE;
use crate::domain_capabilities::WorthQueryDomainCapabilityCategory;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryDomainCapabilityCertifiedSurfaceRow {
    category: WorthQueryDomainCapabilityCategory,
    ordinary_lane: &'static str,
    inspectable_lane: &'static str,
    proof_lane: &'static str,
    raw_lane: &'static str,
    implementation_path: &'static str,
}

impl WorthQueryDomainCapabilityCertifiedSurfaceRow {
    pub(crate) const fn new(
        category: WorthQueryDomainCapabilityCategory,
        ordinary_lane: &'static str,
        inspectable_lane: &'static str,
        proof_lane: &'static str,
        raw_lane: &'static str,
        implementation_path: &'static str,
    ) -> Self {
        Self {
            category,
            ordinary_lane,
            inspectable_lane,
            proof_lane,
            raw_lane,
            implementation_path,
        }
    }

    pub fn category(&self) -> WorthQueryDomainCapabilityCategory {
        self.category
    }

    pub fn ordinary_lane(&self) -> &'static str {
        self.ordinary_lane
    }

    pub fn inspectable_lane(&self) -> &'static str {
        self.inspectable_lane
    }

    pub fn proof_lane(&self) -> &'static str {
        self.proof_lane
    }

    pub fn raw_lane(&self) -> &'static str {
        self.raw_lane
    }

    pub fn implementation_path(&self) -> &'static str {
        self.implementation_path
    }

    pub fn row_digest(&self) -> String {
        compose_certified_surface_row_digest(
            self.category.as_str(),
            self.ordinary_lane,
            self.inspectable_lane,
            self.proof_lane,
            self.raw_lane,
            self.implementation_path,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryDomainCapabilityCertifiedSurfaceInventory {
    rows: &'static [WorthQueryDomainCapabilityCertifiedSurfaceRow],
}

impl WorthQueryDomainCapabilityCertifiedSurfaceInventory {
    pub(crate) const fn new(
        rows: &'static [WorthQueryDomainCapabilityCertifiedSurfaceRow],
    ) -> Self {
        Self { rows }
    }

    pub fn rows(&self) -> &'static [WorthQueryDomainCapabilityCertifiedSurfaceRow] {
        self.rows
    }

    pub fn public_surface_digest(&self) -> String {
        compose_public_surface_digest(
            self.rows
                .iter()
                .map(WorthQueryDomainCapabilityCertifiedSurfaceRow::row_digest),
        )
    }
}

pub fn worth_query_domain_capability_public_surface_inventory(
) -> WorthQueryDomainCapabilityCertifiedSurfaceInventory {
    WorthQueryDomainCapabilityCertifiedSurfaceInventory::new(CERTIFIED_SURFACE_ROWS)
}

const CERTIFIED_SURFACE_ROWS: &[WorthQueryDomainCapabilityCertifiedSurfaceRow] = &[
    WorthQueryDomainCapabilityCertifiedSurfaceRow::new(
        WorthQueryDomainCapabilityCategory::Admission,
        "handle.contributions_in(&workspace)?.for_admitted_intent_plan(&plan)?.advises(...).because(...).materialize()",
        "handle.contributions_in(&workspace)?.for_admitted_intent_plan(&plan)?.advises(...).because(...).try_materialize()",
        "WorthQueryAdmissionContributionAuthoring::advisory(...).bind_to_installed_target(target)",
        "materialize_runtime_admission_decision(...)",
        "crates/worth-query/src/domain_capabilities/dx/common/intent_admission.rs",
    ),
    WorthQueryDomainCapabilityCertifiedSurfaceRow::new(
        WorthQueryDomainCapabilityCategory::SupportTraceability,
        "handle.contributions_in(&workspace)?.for_intent(&intent)?.supports_traceability(...).because(...).materialize()",
        "handle.contributions_in(&workspace)?.for_intent(&intent)?.supports_traceability(...).because(...).try_materialize()",
        "WorthQuerySupportContributionAuthoring::declaration_traceability(...).bind_to_installed_target(target)",
        "materialize_intent_declaration_support_traceability_artifact(...)",
        "crates/worth-query/src/domain_capabilities/dx/common/intent.rs",
    ),
    WorthQueryDomainCapabilityCertifiedSurfaceRow::new(
        WorthQueryDomainCapabilityCategory::InvariantCapability,
        "handle.contributions_in(&workspace)?.for_intent(&intent)?.register_invariant_catalog(...).because(...).materialize()",
        "handle.contributions_in(&workspace)?.for_intent(&intent)?.register_invariant_catalog(...).because(...).try_materialize()",
        "WorthQueryInvariantCapabilityContributionAuthoring::invariant_registration(...).bind_to_installed_target(target)",
        "materialize_query_invariant_catalog_registration_artifact(...)",
        "crates/worth-query/src/domain_capabilities/dx/common/intent.rs",
    ),
    WorthQueryDomainCapabilityCertifiedSurfaceRow::new(
        WorthQueryDomainCapabilityCategory::WorkflowPreview,
        "handle.contributions_in(&workspace)?.for_intent(&intent)?.plans_preview_mutation(...).because(...).materialize()",
        "handle.contributions_in(&workspace)?.for_intent(&intent)?.plans_preview_mutation(...).because(...).try_materialize()",
        "WorthQueryWorkflowContributionAuthoring::promotion_eligible_mutation_lowering(...).bind_to_installed_target(target)",
        "materialize_query_workflow_declaration(...)",
        DOMAIN_CAPABILITY_WORKFLOW_CATEGORY_MODULE,
    ),
    WorthQueryDomainCapabilityCertifiedSurfaceRow::new(
        WorthQueryDomainCapabilityCategory::ContinuityLineage,
        "handle.contributions_in(&workspace)?.for_admitted_intent_plan(&plan)?.preserves_continuity(...).because(...).materialize()",
        "handle.contributions_in(&workspace)?.for_admitted_intent_plan(&plan)?.preserves_continuity(...).because(...).try_materialize()",
        "WorthQueryContinuityContributionAuthoring::preserved_rebind(...).bind_to_installed_target(target)",
        "materialize_runtime_continuity_evidence(...)",
        DOMAIN_CAPABILITY_CONTINUITY_CATEGORY_MODULE,
    ),
    WorthQueryDomainCapabilityCertifiedSurfaceRow::new(
        WorthQueryDomainCapabilityCategory::ConsequenceAftermath,
        "handle.contributions_in(&workspace)?.for_admitted_intent_plan(&plan)?.consumes_projection_contract(...).because(...).materialize()",
        "handle.contributions_in(&workspace)?.for_admitted_intent_plan(&plan)?.consumes_projection_contract(...).because(...).review()",
        "WorthQueryAftermathContributionAuthoring::projection_contract_consumption(...).bind_to_installed_target(target)",
        "materialize_projection_consumption_contract(...)",
        DOMAIN_CAPABILITY_AFTERMATH_CATEGORY_MODULE,
    ),
    WorthQueryDomainCapabilityCertifiedSurfaceRow::new(
        WorthQueryDomainCapabilityCategory::ExplanationInspection,
        "handle.contributions_in(&workspace)?.for_lower_runtime_boundary_envelope(&envelope)?.explains_store_backed_replay_gap(...).because(...).materialize_artifact()",
        "handle.contributions_in(&workspace)?.for_lower_runtime_boundary_envelope(&envelope)?.explains_store_backed_replay_gap(...).because(...).review()",
        "WorthQueryExplanationContributionAuthoring::store_backed_replay_gap_explanation(...).bind_to_installed_target(target)",
        "materialize_query_causal_inspection_artifact(...)",
        DOMAIN_CAPABILITY_EXPLANATION_CATEGORY_MODULE,
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn certified_surface_inventory_covers_each_named_category_once() {
        let inventory = worth_query_domain_capability_public_surface_inventory();
        let categories = inventory
            .rows()
            .iter()
            .map(|row| row.category().as_str())
            .collect::<Vec<_>>();

        assert_eq!(inventory.rows().len(), 7);
        assert_eq!(
            categories.len(),
            categories.iter().copied().collect::<BTreeSet<_>>().len()
        );
        for category in [
            "admission",
            "support-traceability",
            "invariant-capability",
            "workflow-preview",
            "continuity-lineage",
            "consequence-aftermath",
            "explanation-inspection",
        ] {
            assert!(categories.contains(&category));
        }
    }

    #[test]
    fn certified_surface_rows_show_one_lane_at_a_time_degradation() {
        for row in worth_query_domain_capability_public_surface_inventory().rows() {
            assert!(!row.ordinary_lane().is_empty());
            assert!(!row.inspectable_lane().is_empty());
            assert!(!row.proof_lane().is_empty());
            assert!(!row.raw_lane().is_empty());
            assert_ne!(row.ordinary_lane(), row.inspectable_lane());
            assert_ne!(row.inspectable_lane(), row.proof_lane());
            assert_ne!(row.proof_lane(), row.raw_lane());
            assert!(row.ordinary_lane().contains("contributions_in"));
            assert!(!row.ordinary_lane().contains("worth_query_domain"));
            assert!(row.implementation_path().contains("domain_capabilities"));
        }
    }

    #[test]
    fn certified_surface_inventory_digest_is_row_order_stable() {
        let inventory = worth_query_domain_capability_public_surface_inventory();
        let expected = compose_public_surface_digest(
            inventory
                .rows()
                .iter()
                .map(WorthQueryDomainCapabilityCertifiedSurfaceRow::row_digest),
        );

        assert_eq!(inventory.public_surface_digest(), expected);
    }
}
