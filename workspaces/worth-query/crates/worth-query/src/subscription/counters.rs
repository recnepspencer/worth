use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct QuerySubscriptionDeclarationCounters {
    pub(super) family_selection_count: u64,
    pub(super) family_denial_count: u64,
    pub(super) family_registry_lookup_count: u64,
    pub(super) view_family_registry_lookup_count: u64,
    pub(super) equivalence_digest_part_count: u64,
    pub(super) admission_dimension_denial_count: u64,
    pub(super) work_budget_denial_count: u64,
    pub(super) unknown_cost_denial_count: u64,
    pub(super) raw_cdc_fallback_denial_count: u64,
    pub(super) host_observer_inference_denial_count: u64,
    pub(super) relationship_proof_drift_denial_count: u64,
    pub(super) declaration_count: u64,
    pub(super) declaration_denial_count: u64,
    pub(super) declared_slice_count: u64,
    pub(super) deduplicated_slice_count: u64,
    pub(super) slice_deduplication_input_count: u64,
    pub(super) slice_sort_comparison_count: u64,
    pub(super) masked_slice_denial_count: u64,
    pub(super) delivery_intent_denial_count: u64,
    pub(super) declaration_digest_part_count: u64,
    pub(super) bridge_lowering_count: u64,
    pub(super) bridge_family_denial_count: u64,
    pub(super) bridge_fallback_denial_count: u64,
    pub(super) bridge_family_registry_lookup_count: u64,
    pub(super) bridge_slice_count: u64,
    pub(super) bridge_slice_denial_count: u64,
    pub(super) bridge_slice_registry_lookup_count: u64,
    pub(super) basis_binding_request_count: u64,
    pub(super) basis_binding_denial_count: u64,
    pub(super) signal_strategy_request_count: u64,
    pub(super) admission_count: u64,
    pub(super) admission_denial_count: u64,
    pub(super) durable_overclaim_denial_count: u64,
    pub(super) activation_input_count: u64,
    pub(super) active_state_allocation_denial_count: u64,
    pub(super) declaration_time_checkpoint_denial_count: u64,
    pub(super) scratch_allocation_count: u64,
    pub(super) forbidden_heap_allocation_denial_count: u64,
}

impl QuerySubscriptionDeclarationCounters {
    pub fn evidence_identity(&self) -> WorthQueryEvidenceIdentity {
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_declaration_counters_v1",
            )
            .field_usize(
                WorthQueryEvidenceTag::new("family_selection"),
                self.family_selection_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("family_denial"),
                self.family_denial_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("family_registry_lookup"),
                self.family_registry_lookup_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("view_family_registry_lookup"),
                self.view_family_registry_lookup_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("equivalence_digest_part"),
                self.equivalence_digest_part_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("admission_dimension_denial"),
                self.admission_dimension_denial_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("work_budget_denial"),
                self.work_budget_denial_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("unknown_cost_denial"),
                self.unknown_cost_denial_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("raw_cdc_fallback_denial"),
                self.raw_cdc_fallback_denial_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("host_observer_inference_denial"),
                self.host_observer_inference_denial_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("relationship_proof_drift_denial"),
                self.relationship_proof_drift_denial_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("declaration"),
                self.declaration_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("declaration_denial"),
                self.declaration_denial_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("declared_slice"),
                self.declared_slice_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("deduplicated_slice"),
                self.deduplicated_slice_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("slice_deduplication_input"),
                self.slice_deduplication_input_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("slice_sort_comparison"),
                self.slice_sort_comparison_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("masked_slice_denial"),
                self.masked_slice_denial_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("delivery_intent_denial"),
                self.delivery_intent_denial_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("declaration_digest_part"),
                self.declaration_digest_part_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("bridge_lowering"),
                self.bridge_lowering_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("bridge_family_denial"),
                self.bridge_family_denial_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("bridge_fallback_denial"),
                self.bridge_fallback_denial_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("bridge_family_registry_lookup"),
                self.bridge_family_registry_lookup_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("bridge_slice"),
                self.bridge_slice_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("bridge_slice_denial"),
                self.bridge_slice_denial_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("bridge_slice_registry_lookup"),
                self.bridge_slice_registry_lookup_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("basis_binding_request"),
                self.basis_binding_request_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("basis_binding_denial"),
                self.basis_binding_denial_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("signal_strategy_request"),
                self.signal_strategy_request_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("admission"),
                self.admission_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("admission_denial"),
                self.admission_denial_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("durable_overclaim_denial"),
                self.durable_overclaim_denial_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("activation_input"),
                self.activation_input_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("active_state_allocation_denial"),
                self.active_state_allocation_denial_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("declaration_time_checkpoint_denial"),
                self.declaration_time_checkpoint_denial_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("scratch_allocation"),
                self.scratch_allocation_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("forbidden_heap_allocation_denial"),
                self.forbidden_heap_allocation_denial_count as usize,
            )
            .seal()
            .expect("fixed counter grammar contains only primitive values and literals")
    }

    pub fn family_selection_count(&self) -> u64 {
        self.family_selection_count
    }

    pub fn family_denial_count(&self) -> u64 {
        self.family_denial_count
    }

    pub fn family_registry_lookup_count(&self) -> u64 {
        self.family_registry_lookup_count
    }

    pub fn view_family_registry_lookup_count(&self) -> u64 {
        self.view_family_registry_lookup_count
    }

    pub fn equivalence_digest_part_count(&self) -> u64 {
        self.equivalence_digest_part_count
    }

    pub fn admission_dimension_denial_count(&self) -> u64 {
        self.admission_dimension_denial_count
    }

    pub fn work_budget_denial_count(&self) -> u64 {
        self.work_budget_denial_count
    }

    pub fn unknown_cost_denial_count(&self) -> u64 {
        self.unknown_cost_denial_count
    }

    pub fn raw_cdc_fallback_denial_count(&self) -> u64 {
        self.raw_cdc_fallback_denial_count
    }

    pub fn host_observer_inference_denial_count(&self) -> u64 {
        self.host_observer_inference_denial_count
    }

    pub fn relationship_proof_drift_denial_count(&self) -> u64 {
        self.relationship_proof_drift_denial_count
    }

    pub fn declaration_count(&self) -> u64 {
        self.declaration_count
    }

    pub fn declaration_denial_count(&self) -> u64 {
        self.declaration_denial_count
    }

    pub fn declared_slice_count(&self) -> u64 {
        self.declared_slice_count
    }

    pub fn deduplicated_slice_count(&self) -> u64 {
        self.deduplicated_slice_count
    }

    pub fn slice_deduplication_input_count(&self) -> u64 {
        self.slice_deduplication_input_count
    }

    pub fn slice_sort_comparison_count(&self) -> u64 {
        self.slice_sort_comparison_count
    }

    pub fn masked_slice_denial_count(&self) -> u64 {
        self.masked_slice_denial_count
    }

    pub fn delivery_intent_denial_count(&self) -> u64 {
        self.delivery_intent_denial_count
    }

    pub fn declaration_digest_part_count(&self) -> u64 {
        self.declaration_digest_part_count
    }

    pub fn bridge_lowering_count(&self) -> u64 {
        self.bridge_lowering_count
    }

    pub fn bridge_family_denial_count(&self) -> u64 {
        self.bridge_family_denial_count
    }

    pub fn bridge_fallback_denial_count(&self) -> u64 {
        self.bridge_fallback_denial_count
    }

    pub fn bridge_family_registry_lookup_count(&self) -> u64 {
        self.bridge_family_registry_lookup_count
    }

    pub fn bridge_slice_count(&self) -> u64 {
        self.bridge_slice_count
    }

    pub fn bridge_slice_denial_count(&self) -> u64 {
        self.bridge_slice_denial_count
    }

    pub fn bridge_slice_registry_lookup_count(&self) -> u64 {
        self.bridge_slice_registry_lookup_count
    }

    pub fn basis_binding_request_count(&self) -> u64 {
        self.basis_binding_request_count
    }

    pub fn basis_binding_denial_count(&self) -> u64 {
        self.basis_binding_denial_count
    }

    pub fn signal_strategy_request_count(&self) -> u64 {
        self.signal_strategy_request_count
    }

    pub fn admission_count(&self) -> u64 {
        self.admission_count
    }

    pub fn admission_denial_count(&self) -> u64 {
        self.admission_denial_count
    }

    pub fn durable_overclaim_denial_count(&self) -> u64 {
        self.durable_overclaim_denial_count
    }

    pub fn activation_input_count(&self) -> u64 {
        self.activation_input_count
    }

    pub fn active_state_allocation_denial_count(&self) -> u64 {
        self.active_state_allocation_denial_count
    }

    pub fn declaration_time_checkpoint_denial_count(&self) -> u64 {
        self.declaration_time_checkpoint_denial_count
    }

    pub fn scratch_allocation_count(&self) -> u64 {
        self.scratch_allocation_count
    }

    pub fn forbidden_heap_allocation_denial_count(&self) -> u64 {
        self.forbidden_heap_allocation_denial_count
    }
}
