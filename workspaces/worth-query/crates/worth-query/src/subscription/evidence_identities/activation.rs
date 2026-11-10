use super::super::basis_request::QuerySubscriptionBasisBindingRequestKind;
use super::super::bridge_family::BridgeSubscriptionDeclarationFamilyKind;
use super::super::bridge_slice::BridgeSubscriptionSliceKind;
use super::super::delivery::QuerySubscriptionDeliveryIntent;
use super::super::family::QuerySubscriptionFamily;
use super::super::future_selection::QuerySubscriptionFutureSelection;
use super::super::posture::{
    QuerySubscriptionBasisPosture, QuerySubscriptionBridgePosture, QuerySubscriptionCostPosture,
};
use super::super::selection_live_graph_access::QuerySubscriptionLiveGraphAccessPosture;
use super::super::signal_strategy::QuerySubscriptionSignalStrategyRequestKind;
use super::super::slice::{QuerySubscriptionSliceIntent, QuerySubscriptionSlicePart};
use super::super::slice_budget::QuerySubscriptionSliceBudget;
use super::SUBSCRIPTION_IDENTITY_SCOPE;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

#[allow(clippy::too_many_arguments)]
pub(in crate::subscription) fn query_subscription_declaration_identity(
    family: &QuerySubscriptionFamily,
    live_family: &LiveQueryFamily,
    view_family: Option<LiveViewShapeFamily>,
    cost_posture: &QuerySubscriptionCostPosture,
    basis_posture: &QuerySubscriptionBasisPosture,
    bridge_posture: &QuerySubscriptionBridgePosture,
    live_graph_access_posture: &QuerySubscriptionLiveGraphAccessPosture,
    future_selection: &QuerySubscriptionFutureSelection,
    equivalence_identity: &WorthQueryEvidenceIdentity,
    slice_intent: &QuerySubscriptionSliceIntent,
    delivery_intent: &QuerySubscriptionDeliveryIntent,
    max_admitted_slice_count: usize,
    slice_budget: &QuerySubscriptionSliceBudget,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "query_subscription_declaration_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("family"), family.as_str())
        .field_shape(
            WorthQueryEvidenceTag::new("live_family"),
            live_family.as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("view_family"),
            view_family
                .as_ref()
                .map(LiveViewShapeFamily::as_str)
                .unwrap_or("none"),
        )
        .field_shape(WorthQueryEvidenceTag::new("cost"), cost_posture.as_str())
        .field_shape(WorthQueryEvidenceTag::new("basis"), basis_posture.as_str())
        .field_shape(
            WorthQueryEvidenceTag::new("bridge"),
            bridge_posture.as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("live_graph_access_posture"),
            live_graph_access_posture.as_str(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("future_selection"),
            future_selection.projection_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("equivalence"),
            equivalence_identity,
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("slice_intent"),
            slice_intent
                .parts()
                .iter()
                .map(QuerySubscriptionSlicePart::canonical_part),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("delivery_intent"),
            delivery_intent.as_str(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("work_budget_max_slices"),
            max_admitted_slice_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("slice_budget_projection"),
            slice_budget.projected_slice_width_limit(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("slice_budget_ordering"),
            slice_budget.ordering_slice_width_limit(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("slice_budget_grouping"),
            slice_budget.grouping_slice_width_limit(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("slice_budget_relation"),
            slice_budget.relation_scope_slice_width_limit(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("slice_budget_metadata"),
            slice_budget.metadata_slice_width_limit(),
        )
        .seal()
}

pub(in crate::subscription) fn basis_binding_request_identity(
    request_kind: &QuerySubscriptionBasisBindingRequestKind,
    source_declaration_identity: &WorthQueryEvidenceIdentity,
    source_equivalence_identity: &WorthQueryEvidenceIdentity,
    scoped_declaration_basis_digest: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "query_subscription_basis_binding_request_v1",
        )
        .field_shape(
            WorthQueryEvidenceTag::new("request_kind"),
            request_kind.as_str(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("source_declaration"),
            source_declaration_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("source_equivalence"),
            source_equivalence_identity,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("scoped_declaration_basis"),
            scoped_declaration_basis_digest,
        )
        .seal()
}

pub(in crate::subscription) fn signal_strategy_request_identity(
    request_kind: &QuerySubscriptionSignalStrategyRequestKind,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_signal_strategy_request_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("request_kind"),
                request_kind.as_str(),
            )
            .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens")
}

pub(in crate::subscription) fn bridge_lowering_plan_identity(
    query_declaration_identity: &WorthQueryEvidenceIdentity,
    bridge_family: &BridgeSubscriptionDeclarationFamilyKind,
    basis_request_identity: &WorthQueryEvidenceIdentity,
    signal_strategy_identity: &WorthQueryEvidenceIdentity,
    bridge_slices: &[BridgeSubscriptionSliceKind],
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "query_subscription_bridge_lowering_v1",
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("query_declaration"),
            query_declaration_identity,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("bridge_family"),
            bridge_family.as_str(),
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("basis"), basis_request_identity)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("signal_strategy"),
            signal_strategy_identity,
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("bridge_slices"),
            bridge_slices
                .iter()
                .map(BridgeSubscriptionSliceKind::as_str),
        )
        .seal()
}

pub(in crate::subscription) fn admission_artifact_identity(
    query_declaration_identity: &WorthQueryEvidenceIdentity,
    bridge_declaration_identity: &WorthQueryEvidenceIdentity,
    basis_binding_identity: &WorthQueryEvidenceIdentity,
    signal_strategy_identity: &WorthQueryEvidenceIdentity,
    diagnostics_identity: &WorthQueryEvidenceIdentity,
    support_identity: &WorthQueryEvidenceIdentity,
    declaration_width_limit: usize,
    bridge_width_limit: usize,
    basis_width_limit: usize,
    signal_width_limit: usize,
    activation_width_limit: usize,
    counters_identity: &WorthQueryEvidenceIdentity,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_admission_artifact_v1",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("query_declaration"),
                query_declaration_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("bridge_declaration"),
                bridge_declaration_identity,
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("basis"), basis_binding_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("signal_strategy"),
                signal_strategy_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("diagnostics"),
                diagnostics_identity,
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("support"), support_identity)
            .field_usize(
                WorthQueryEvidenceTag::new("budget_declaration"),
                declaration_width_limit,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("budget_bridge"),
                bridge_width_limit,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("budget_basis"),
                basis_width_limit,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("budget_signal"),
                signal_width_limit,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("budget_activation"),
                activation_width_limit,
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("counters"), counters_identity)
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}

pub(in crate::subscription) fn activation_checkpoint_identity(
    query_declaration_identity: &WorthQueryEvidenceIdentity,
    basis_binding_identity: &WorthQueryEvidenceIdentity,
    future_selection_projection_identity: &WorthQueryEvidenceIdentity,
    signal_strategy_identity: &WorthQueryEvidenceIdentity,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_active_checkpoint_identity_v1",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("query_declaration"),
                query_declaration_identity,
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("basis"), basis_binding_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("future_selection"),
                future_selection_projection_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("signal_strategy"),
                signal_strategy_identity,
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}

pub(in crate::subscription) fn activation_input_identity(
    admission_identity: &WorthQueryEvidenceIdentity,
    query_declaration_identity: &WorthQueryEvidenceIdentity,
    bridge_declaration_identity: &WorthQueryEvidenceIdentity,
    basis_binding_identity: &WorthQueryEvidenceIdentity,
    checkpoint_identity: &WorthQueryEvidenceIdentity,
    signal_strategy_identity: &WorthQueryEvidenceIdentity,
    future_selection_projection_identity: &WorthQueryEvidenceIdentity,
    counters_identity: &WorthQueryEvidenceIdentity,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_activation_input_v1",
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("admission"), admission_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("query_declaration"),
                query_declaration_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("bridge_declaration"),
                bridge_declaration_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("future_selection"),
                future_selection_projection_identity,
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("basis"), basis_binding_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("checkpoint"),
                checkpoint_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("signal_strategy"),
                signal_strategy_identity,
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("counters"), counters_identity)
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}
