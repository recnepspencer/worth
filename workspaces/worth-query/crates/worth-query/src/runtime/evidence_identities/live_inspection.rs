use super::*;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

pub(in crate::runtime) struct RuntimeLiveViewInspectionIdentityParts<'a> {
    pub view_name: &'a str,
    pub authority_lane: WorthQueryAuthorityLane,
    pub query_identity: &'a WorthQueryEvidenceIdentity,
    pub view_shape_identity: &'a WorthQueryEvidenceIdentity,
    pub subscription_family: &'a str,
    pub subscription_family_identity: &'a WorthQueryEvidenceIdentity,
    pub subscription_declaration_identity: &'a WorthQueryEvidenceIdentity,
    pub bridge_declaration_identity: &'a WorthQueryEvidenceIdentity,
    pub admission_identity: &'a WorthQueryEvidenceIdentity,
    pub activation_identity: &'a WorthQueryEvidenceIdentity,
    pub basis_binding_identity: &'a WorthQueryEvidenceIdentity,
    pub signal_strategy_identity: &'a WorthQueryEvidenceIdentity,
    pub active_lane_identity: &'a WorthQueryEvidenceIdentity,
    pub consumer_attachment_identity: &'a WorthQueryEvidenceIdentity,
    pub consumer_identity: &'a WorthQueryEvidenceIdentity,
    pub delivery_cursor_identity: &'a WorthQueryEvidenceIdentity,
    pub subscription_budget_policy: &'a str,
    pub active_lifecycle_budget_policy: &'a str,
    pub consumer_attachment_budget_policy: &'a str,
    pub runtime_budget_identity: &'a WorthQueryEvidenceIdentity,
    pub support_identity: &'a WorthQueryEvidenceIdentity,
    pub last_delivery_cause_kind: Option<QuerySubscriptionDeliveryCauseKind>,
    pub last_delivery_cause_identity: Option<&'a WorthQueryEvidenceIdentity>,
    pub last_delivery_had_relational_patch: bool,
    pub mixed_cause_identity: Option<&'a WorthQueryEvidenceIdentity>,
    pub ordinary_runtime_posture: Option<&'a WorthQueryOrdinaryRuntimePosture>,
    pub async_result_state: Option<&'a WorthQueryRuntimeAsyncResultState>,
    pub remask_posture: Option<&'a WorthQueryRuntimeRemaskPosture>,
    pub installation_identity: &'a WorthQueryEvidenceIdentity,
    pub counter_inspection_identity: &'a WorthQueryEvidenceIdentity,
}

pub(in crate::runtime) fn runtime_live_view_inspection_identity(
    parts: RuntimeLiveViewInspectionIdentityParts<'_>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_live_view_inspection_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("view"), parts.view_name)
            .field_shape(
                WorthQueryEvidenceTag::new("authority_lane"),
                parts.authority_lane.as_str(),
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("query"), parts.query_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("view_shape"),
                parts.view_shape_identity,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                parts.subscription_family,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("subscription_family"),
                parts.subscription_family_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("subscription_declaration"),
                parts.subscription_declaration_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("bridge_declaration"),
                parts.bridge_declaration_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("admission"),
                parts.admission_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("activation"),
                parts.activation_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("basis_binding"),
                parts.basis_binding_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("signal_strategy"),
                parts.signal_strategy_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("active_lane"),
                parts.active_lane_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("consumer_attachment"),
                parts.consumer_attachment_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("consumer"),
                parts.consumer_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("delivery_cursor"),
                parts.delivery_cursor_identity,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("subscription_budget_policy"),
                parts.subscription_budget_policy,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("active_lifecycle_budget_policy"),
                parts.active_lifecycle_budget_policy,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("consumer_attachment_budget_policy"),
                parts.consumer_attachment_budget_policy,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("runtime_budget"),
                parts.runtime_budget_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("support"),
                parts.support_identity,
            )
            .optional_shape(
                WorthQueryEvidenceTag::new("last_delivery_cause"),
                parts
                    .last_delivery_cause_kind
                    .map(QuerySubscriptionDeliveryCauseKind::as_str),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("last_delivery_digest"),
                parts.last_delivery_cause_identity,
            )
            .field_bool(
                WorthQueryEvidenceTag::new("last_delivery_relational"),
                parts.last_delivery_had_relational_patch,
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("mixed_cause"),
                parts.mixed_cause_identity,
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("ordinary_runtime_posture"),
                parts
                    .ordinary_runtime_posture
                    .map(WorthQueryOrdinaryRuntimePosture::evidence_identity),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("async_result_state"),
                parts
                    .async_result_state
                    .map(WorthQueryRuntimeAsyncResultState::result_state_identity),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("remask"),
                parts
                    .remask_posture
                    .map(WorthQueryRuntimeRemaskPosture::remask_identity),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("installation"),
                parts.installation_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("counters"),
                parts.counter_inspection_identity,
            )
            .seal()?,
    )
}

pub(in crate::runtime) fn runtime_live_subscription_counter_inspection_identity(
    declaration_counter_identity: &WorthQueryEvidenceIdentity,
    active_lane_counter_identity: &WorthQueryEvidenceIdentity,
    consumer_attachment_counter_identity: &WorthQueryEvidenceIdentity,
    family_selection_count: u64,
    declaration_count: u64,
    bridge_lowering_count: u64,
    admission_count: u64,
    activation_input_count: u64,
    active_lane_admission_count: u64,
    active_lane_creation_count: u64,
    active_lane_join_count: u64,
    active_lane_handle_issue_count: u64,
    consumer_attachment_count: u64,
    consumer_attachment_denial_count: u64,
) -> WorthQueryEvidenceIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_live_subscription_inspection_counters_v1",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("declaration_counters"),
                declaration_counter_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("active_lane_counters"),
                active_lane_counter_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("consumer_attachment_counters"),
                consumer_attachment_counter_identity,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("family_selection"),
                family_selection_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("declaration"),
                declaration_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("bridge_lowering"),
                bridge_lowering_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("admission"),
                admission_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("activation_input"),
                activation_input_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("active_lane_admission"),
                active_lane_admission_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("active_lane_creation"),
                active_lane_creation_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("active_lane_join"),
                active_lane_join_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("active_lane_handle_issue"),
                active_lane_handle_issue_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("consumer_attachment"),
                consumer_attachment_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("consumer_attachment_denial"),
                consumer_attachment_denial_count as usize,
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}
