use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use super::family::QuerySubscriptionFamily;

const DELIVERY_INTENT_IDENTITY_SCOPE: WorthQueryEvidenceScope =
    WorthQueryEvidenceScope::SubscriptionActivationReceipt;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum QuerySubscriptionDeliveryIntent {
    ExactDetailReplacement,
    OrderedMembershipDelta,
    GroupedMembershipDelta,
    InspectorFocusedDetailReplacement,
    BoundedMaterializationMembershipDelta,
}

impl QuerySubscriptionDeliveryIntent {
    pub(super) fn for_family(family: &QuerySubscriptionFamily) -> Self {
        match family {
            QuerySubscriptionFamily::DetailExact => Self::ExactDetailReplacement,
            QuerySubscriptionFamily::CollectionMembership => Self::OrderedMembershipDelta,
            QuerySubscriptionFamily::GroupedCollectionMembership => Self::GroupedMembershipDelta,
            QuerySubscriptionFamily::InspectorDetailExact => {
                Self::InspectorFocusedDetailReplacement
            }
            QuerySubscriptionFamily::BoundedMaterialization => {
                Self::BoundedMaterializationMembershipDelta
            }
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ExactDetailReplacement => "exact_detail_replacement",
            Self::OrderedMembershipDelta => "ordered_membership_delta",
            Self::GroupedMembershipDelta => "grouped_membership_delta",
            Self::InspectorFocusedDetailReplacement => "inspector_focused_detail_replacement",
            Self::BoundedMaterializationMembershipDelta => {
                "bounded_materialization_membership_delta"
            }
        }
    }

    pub fn evidence_identity(&self) -> WorthQueryEvidenceIdentity {
        WorthQueryEvidenceIdentity::compose(DELIVERY_INTENT_IDENTITY_SCOPE)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "query_subscription_delivery_intent_v1",
                )
                .field_shape(WorthQueryEvidenceTag::new("intent"), self.as_str())
                .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens")
    }
}
