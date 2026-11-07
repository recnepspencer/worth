use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::identity::{CanonicalQueryDigest, CanonicalResultShapeDigest};
use crate::subscription::{
    ActiveSubscriptionCounters, QuerySubscriptionDeclarationCounters, QuerySubscriptionFamily,
    SubscriptionConsumerAttachment,
};

use super::WorthQueryAuthorityLane;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRuntimeLiveSubscriptionBudgetPolicyIdentity {
    policy_label: String,
    policy_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryRuntimeLiveSubscriptionBudgetPolicyIdentity {
    pub fn subscription_policy(
        policy_label: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new("subscription_budget_policy", policy_label)?)
    }

    pub fn active_lifecycle_policy(
        policy_label: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new("active_lifecycle_budget_policy", policy_label)?)
    }

    pub fn consumer_attachment_policy(
        policy_label: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            "consumer_attachment_budget_policy",
            policy_label,
        )?)
    }

    fn new(
        role: &'static str,
        policy_label: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let policy_label = policy_label.into();
        let policy_identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "runtime_live_subscription_budget_policy_member_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("role"), role)
        .field_shape(WorthQueryEvidenceTag::new("policy"), &policy_label)
        .seal()?;

        Ok(Self {
            policy_label,
            policy_identity,
        })
    }

    pub fn policy_label(&self) -> &str {
        &self.policy_label
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.policy_identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRuntimeLiveSubscriptionInstallation {
    pub(super) view_name: String,
    pub(super) authority_lane: WorthQueryAuthorityLane,
    pub(super) query_identity: WorthQueryEvidenceIdentity,
    pub(super) canonical_query_digest: CanonicalQueryDigest,
    pub(super) view_shape_identity: WorthQueryEvidenceIdentity,
    pub(super) canonical_result_shape_digest: CanonicalResultShapeDigest,
    pub(super) canonical_result_shape_identity: WorthQueryEvidenceIdentity,
    pub(super) subscription_family: QuerySubscriptionFamily,
    pub(super) subscription_family_identity: WorthQueryEvidenceIdentity,
    pub(super) subscription_declaration_identity: WorthQueryEvidenceIdentity,
    pub(super) bridge_declaration_identity: WorthQueryEvidenceIdentity,
    pub(super) admission_identity: WorthQueryEvidenceIdentity,
    pub(super) activation_identity: WorthQueryEvidenceIdentity,
    pub(super) basis_binding_identity: WorthQueryEvidenceIdentity,
    pub(super) signal_strategy_identity: WorthQueryEvidenceIdentity,
    pub(super) active_lane_identity: WorthQueryEvidenceIdentity,
    pub(super) consumer_attachment_identity: WorthQueryEvidenceIdentity,
    pub(super) consumer_identity: WorthQueryEvidenceIdentity,
    pub(super) delivery_cursor_identity: WorthQueryEvidenceIdentity,
    pub(super) subscription_budget_policy: WorthQueryRuntimeLiveSubscriptionBudgetPolicyIdentity,
    pub(super) active_lifecycle_budget_policy:
        WorthQueryRuntimeLiveSubscriptionBudgetPolicyIdentity,
    pub(super) consumer_attachment_budget_policy:
        WorthQueryRuntimeLiveSubscriptionBudgetPolicyIdentity,
    pub(super) runtime_budget_identity: WorthQueryEvidenceIdentity,
    pub(super) support_identity: WorthQueryEvidenceIdentity,
    pub(super) counters: QuerySubscriptionDeclarationCounters,
    pub(super) active_lane_counters: ActiveSubscriptionCounters,
    pub(super) consumer_attachment_counters: ActiveSubscriptionCounters,
    pub(super) installation_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryRuntimeLiveSubscriptionInstallation {
    pub(crate) fn new(
        view_name: impl Into<String>,
        query_source_identity: WorthQueryEvidenceIdentity,
        canonical_query_digest: CanonicalQueryDigest,
        view_shape_source_identity: WorthQueryEvidenceIdentity,
        canonical_result_shape_digest: CanonicalResultShapeDigest,
        subscription_family: QuerySubscriptionFamily,
        subscription_declaration_source_identity: WorthQueryEvidenceIdentity,
        bridge_declaration_source_identity: WorthQueryEvidenceIdentity,
        admission_source_identity: WorthQueryEvidenceIdentity,
        activation_source_identity: WorthQueryEvidenceIdentity,
        basis_binding_source_identity: WorthQueryEvidenceIdentity,
        signal_strategy_source_identity: WorthQueryEvidenceIdentity,
        active_lane_source_identity: WorthQueryEvidenceIdentity,
        consumer_attachment: &SubscriptionConsumerAttachment,
        subscription_budget_policy: WorthQueryRuntimeLiveSubscriptionBudgetPolicyIdentity,
        active_lifecycle_budget_policy: WorthQueryRuntimeLiveSubscriptionBudgetPolicyIdentity,
        consumer_attachment_budget_policy: WorthQueryRuntimeLiveSubscriptionBudgetPolicyIdentity,
        active_lane_counters: ActiveSubscriptionCounters,
        consumer_attachment_counters: ActiveSubscriptionCounters,
        support_source_identity: WorthQueryEvidenceIdentity,
        counters: QuerySubscriptionDeclarationCounters,
    ) -> Result<Self, IdentityDerivationDenial> {
        let view_name = view_name.into();
        let query_identity = live_subscription_input_identity("query", &query_source_identity)?;
        let view_shape_identity =
            live_subscription_input_identity("view_shape", &view_shape_source_identity)?;
        let canonical_result_shape_identity =
            crate::identity::canonical_result_shape_evidence_identity(
                &canonical_result_shape_digest,
            )?;
        let subscription_declaration_identity = live_subscription_input_identity(
            "subscription_declaration",
            &subscription_declaration_source_identity,
        )?;
        let bridge_declaration_identity = live_subscription_input_identity(
            "bridge_declaration",
            &bridge_declaration_source_identity,
        )?;
        let admission_identity =
            live_subscription_input_identity("admission", &admission_source_identity)?;
        let activation_identity =
            live_subscription_input_identity("activation", &activation_source_identity)?;
        let basis_binding_identity =
            live_subscription_input_identity("basis_binding", &basis_binding_source_identity)?;
        let signal_strategy_identity =
            live_subscription_input_identity("signal_strategy", &signal_strategy_source_identity)?;
        let active_lane_identity =
            live_subscription_input_identity("active_lane", &active_lane_source_identity)?;
        let consumer_attachment_identity = live_subscription_input_identity(
            "consumer_attachment",
            consumer_attachment.attachment_digest().evidence_identity(),
        )?;
        let consumer_identity =
            live_subscription_input_identity("consumer", consumer_attachment.consumer_identity())?;
        let delivery_cursor_identity = live_subscription_input_identity(
            "delivery_cursor",
            consumer_attachment.delivery_cursor_identity(),
        )?;
        let runtime_budget_identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "runtime_live_subscription_budget_policy_v1",
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("subscription_budget_policy"),
            subscription_budget_policy.evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("active_lifecycle_budget_policy"),
            active_lifecycle_budget_policy.evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("consumer_attachment_budget_policy"),
            consumer_attachment_budget_policy.evidence_identity(),
        )
        .seal()?;
        let support_identity =
            live_subscription_input_identity("support", &support_source_identity)?;
        let subscription_family_source_identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "runtime_live_subscription_family_source_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("role"), "subscription_family")
        .field_shape(
            WorthQueryEvidenceTag::new("family"),
            subscription_family.as_str(),
        )
        .seal()?;
        let subscription_family_identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "runtime_live_subscription_family_v1",
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("subscription_family"),
            &subscription_family_source_identity,
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("query"), &query_identity)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("view_shape"),
            &view_shape_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("canonical_result_shape"),
            &canonical_result_shape_identity,
        )
        .seal()?;
        let counter_identity =
            live_subscription_source_identity("counters", &counters.evidence_identity())?;
        let active_lane_counter_identity = live_subscription_source_identity(
            "active_lane_counters",
            &active_lane_counters.evidence_identity(),
        )?;
        let consumer_attachment_counter_identity = live_subscription_source_identity(
            "consumer_attachment_counters",
            &consumer_attachment_counters.evidence_identity(),
        )?;
        let installation_identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "runtime_live_subscription_installation_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("view"), view_name.as_str())
        .field_evidence_identity(WorthQueryEvidenceTag::new("query"), &query_identity)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("view_shape"),
            &view_shape_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("canonical_result_shape"),
            &canonical_result_shape_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("subscription_family"),
            &subscription_family_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("subscription_declaration"),
            &subscription_declaration_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("bridge_declaration"),
            &bridge_declaration_identity,
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("admission"), &admission_identity)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("activation"),
            &activation_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("basis_binding"),
            &basis_binding_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("signal_strategy"),
            &signal_strategy_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("active_lane"),
            &active_lane_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("consumer_attachment"),
            &consumer_attachment_identity,
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("consumer"), &consumer_identity)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("delivery_cursor"),
            &delivery_cursor_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("runtime_budget"),
            &runtime_budget_identity,
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("support"), &support_identity)
        .field_evidence_identity(WorthQueryEvidenceTag::new("counters"), &counter_identity)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("active_lane_counters"),
            &active_lane_counter_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("consumer_attachment_counters"),
            &consumer_attachment_counter_identity,
        )
        .seal()?;

        Ok(Self {
            view_name,
            authority_lane: WorthQueryAuthorityLane::AuthoritativeTruth,
            query_identity,
            canonical_query_digest,
            view_shape_identity,
            canonical_result_shape_digest,
            canonical_result_shape_identity,
            subscription_family,
            subscription_family_identity,
            subscription_declaration_identity,
            bridge_declaration_identity,
            admission_identity,
            activation_identity,
            basis_binding_identity,
            signal_strategy_identity,
            active_lane_identity,
            consumer_attachment_identity,
            consumer_identity,
            delivery_cursor_identity,
            subscription_budget_policy,
            active_lifecycle_budget_policy,
            consumer_attachment_budget_policy,
            runtime_budget_identity,
            support_identity,
            counters,
            active_lane_counters,
            consumer_attachment_counters,
            installation_identity,
        })
    }
}

fn live_subscription_input_identity(
    role: &str,
    source_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "runtime_live_subscription_input_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("role"), role)
            .field_evidence_identity(WorthQueryEvidenceTag::new("source"), source_identity)
            .seal()?,
    )
}

pub(crate) fn live_subscription_view_shape_source_identity(
    family: crate::view_shape_live::LiveViewShapeFamily,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "runtime_live_subscription_view_shape_source_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("role"), "live_view")
            .field_shape(WorthQueryEvidenceTag::new("family"), family.as_str())
            .field_shape(
                WorthQueryEvidenceTag::new("underlying"),
                family.underlying_live_family().as_str(),
            )
            .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens")
}

pub(crate) fn live_subscription_source_identity(
    role: &str,
    source_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "runtime_live_subscription_source_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("role"), role)
            .field_evidence_identity(WorthQueryEvidenceTag::new("source"), source_identity)
            .seal()?,
    )
}
