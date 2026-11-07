use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::super::identities::{
    coverage_evidence_variation_set_identity, lifecycle_class_variation_set_identity,
};
use super::row::QuerySubscriptionLifecycleCoverageClass;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionBasisVariationSet {
    digests: Vec<String>,
    identities: Vec<WorthQueryEvidenceIdentity>,
    variation_identity: WorthQueryEvidenceIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionPolicyVariationSet {
    digests: Vec<String>,
    identities: Vec<WorthQueryEvidenceIdentity>,
    variation_identity: WorthQueryEvidenceIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionTenantVariationSet {
    digests: Vec<String>,
    identities: Vec<WorthQueryEvidenceIdentity>,
    variation_identity: WorthQueryEvidenceIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionRelationshipProofVariationSet {
    digests: Vec<String>,
    identities: Vec<WorthQueryEvidenceIdentity>,
    variation_identity: WorthQueryEvidenceIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionViewShapeVariationSet {
    digests: Vec<String>,
    identities: Vec<WorthQueryEvidenceIdentity>,
    variation_identity: WorthQueryEvidenceIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionLifecycleClassVariationSet {
    classes: Vec<QuerySubscriptionLifecycleCoverageClass>,
    variation_identity: WorthQueryEvidenceIdentity,
}

macro_rules! variation_set_impl {
    ($name:ident, $prefix:literal) => {
        impl $name {
            pub(super) fn from_identities<'a, I>(
                values: I,
            ) -> Result<Self, IdentityDerivationDenial>
            where
                I: IntoIterator<Item = &'a WorthQueryEvidenceIdentity>,
            {
                let mut identities = values.into_iter().cloned().collect::<Vec<_>>();
                identities.sort_by(compare_evidence_identity);
                identities.dedup_by(same_evidence_identity);
                let digests = identities
                    .iter()
                    .map(|identity| identity.as_str().to_string())
                    .collect::<Vec<_>>();
                let variation_identity =
                    coverage_evidence_variation_set_identity($prefix, identities.iter())?;
                Ok(Self {
                    digests,
                    identities,
                    variation_identity,
                })
            }

            pub(crate) fn digests(&self) -> &[String] {
                &self.digests
            }
            pub fn identities(&self) -> &[WorthQueryEvidenceIdentity] {
                &self.identities
            }

            pub fn variation_identity(&self) -> &WorthQueryEvidenceIdentity {
                &self.variation_identity
            }
        }
    };
}

fn compare_evidence_identity(
    left: &WorthQueryEvidenceIdentity,
    right: &WorthQueryEvidenceIdentity,
) -> std::cmp::Ordering {
    left.scope()
        .cmp(&right.scope())
        .then_with(|| left.scheme().cmp(&right.scheme()))
        .then_with(|| left.as_str().cmp(right.as_str()))
}

fn same_evidence_identity(
    left: &mut WorthQueryEvidenceIdentity,
    right: &mut WorthQueryEvidenceIdentity,
) -> bool {
    left.scope() == right.scope()
        && left.scheme() == right.scheme()
        && left.as_str() == right.as_str()
}

variation_set_impl!(
    QuerySubscriptionBasisVariationSet,
    "query_subscription_basis_variation_set_v1"
);
variation_set_impl!(
    QuerySubscriptionPolicyVariationSet,
    "query_subscription_policy_variation_set_v1"
);
variation_set_impl!(
    QuerySubscriptionTenantVariationSet,
    "query_subscription_tenant_variation_set_v1"
);
variation_set_impl!(
    QuerySubscriptionRelationshipProofVariationSet,
    "query_subscription_relationship_proof_variation_set_v1"
);
variation_set_impl!(
    QuerySubscriptionViewShapeVariationSet,
    "query_subscription_view_shape_variation_set_v1"
);

impl QuerySubscriptionLifecycleClassVariationSet {
    pub(super) fn from_set(
        values: std::collections::BTreeSet<QuerySubscriptionLifecycleCoverageClass>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let classes = values.into_iter().collect::<Vec<_>>();
        let variation_identity =
            lifecycle_class_variation_set_identity(classes.iter().map(|value| value.as_str()))?;
        Ok(Self {
            classes,
            variation_identity,
        })
    }

    pub fn classes(&self) -> &[QuerySubscriptionLifecycleCoverageClass] {
        &self.classes
    }
    pub fn variation_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.variation_identity
    }
}
