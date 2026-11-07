use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::identity_authority::{QueryProjectionIdentity, QuerySubscriptionIdentityKind};

use super::activation::SubscriptionActivationInput;
use super::certification::{
    QuerySubscriptionCertificationDenialKind, QuerySubscriptionCertificationError,
};
use super::counters::QuerySubscriptionDeclarationCounters;
use super::evidence_identities::typed_identity_drift;
use super::evidence_identities::{scale_counter_snapshot_identity, scale_slope_report_identity};
use super::evidence_projection::subscription_evidence_projection;
use super::validation_evidence::{
    validation_role_evidence_identity, validation_u64_role_evidence_identity,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuerySubscriptionScaleFixtureSize {
    Small,
    Medium,
    Large,
}

impl QuerySubscriptionScaleFixtureSize {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionScaleCounterSnapshot {
    fixture_size: QuerySubscriptionScaleFixtureSize,
    fixture_row_count: u64,
    activation_identity: WorthQueryEvidenceIdentity,
    admission_identity: WorthQueryEvidenceIdentity,
    counter_identity: WorthQueryEvidenceIdentity,
    counters: QuerySubscriptionDeclarationCounters,
    snapshot_identity: WorthQueryEvidenceIdentity,
}

impl QuerySubscriptionScaleCounterSnapshot {
    pub fn from_activation(
        fixture_size: QuerySubscriptionScaleFixtureSize,
        fixture_row_count: u64,
        activation: &SubscriptionActivationInput,
    ) -> Result<Self, IdentityDerivationDenial> {
        let activation_identity = activation.evidence_identity().clone();
        let admission_identity = activation.admission_identity().clone();
        let counters = activation.counters().clone();
        let counter_identity = counters.evidence_identity();
        let snapshot_identity = scale_counter_snapshot_identity(
            fixture_size.as_str(),
            fixture_row_count,
            &activation_identity,
            &admission_identity,
            &counter_identity,
        )?;
        Ok(Self {
            fixture_size,
            fixture_row_count,
            activation_identity,
            admission_identity,
            counter_identity,
            counters,
            snapshot_identity,
        })
    }

    pub fn fixture_size(&self) -> &QuerySubscriptionScaleFixtureSize {
        &self.fixture_size
    }

    pub fn fixture_row_count(&self) -> u64 {
        self.fixture_row_count
    }

    pub fn activation_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.activation_identity)
    }

    pub fn activation_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.activation_identity
    }

    pub fn admission_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.admission_identity)
    }

    pub fn admission_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.admission_identity
    }

    pub fn counter_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.counter_identity)
    }

    pub fn counter_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.counter_identity
    }

    pub fn counters(&self) -> &QuerySubscriptionDeclarationCounters {
        &self.counters
    }

    pub fn snapshot_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.snapshot_identity)
    }

    pub fn snapshot_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.snapshot_identity
    }

    #[cfg(test)]
    pub(super) fn with_bridge_slice_count_for_test(
        mut self,
        activation: &SubscriptionActivationInput,
        bridge_slice_count: u64,
    ) -> Result<Self, IdentityDerivationDenial> {
        self.counters.bridge_slice_count = bridge_slice_count;
        self.counter_identity = self.counters.evidence_identity();
        let snapshot_identity = scale_counter_snapshot_identity(
            self.fixture_size.as_str(),
            self.fixture_row_count,
            activation.evidence_identity(),
            activation.admission_identity(),
            &self.counter_identity,
        )?;
        self.snapshot_identity = snapshot_identity;
        Ok(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionScaleSlopeReport {
    report_identity: WorthQueryEvidenceIdentity,
    activation_identity: WorthQueryEvidenceIdentity,
    admission_identity: WorthQueryEvidenceIdentity,
    small_snapshot_identity: WorthQueryEvidenceIdentity,
    medium_snapshot_identity: WorthQueryEvidenceIdentity,
    large_snapshot_identity: WorthQueryEvidenceIdentity,
    small_row_count: u64,
    medium_row_count: u64,
    large_row_count: u64,
    structural_counter_identity: WorthQueryEvidenceIdentity,
}

impl QuerySubscriptionScaleSlopeReport {
    pub fn report_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.report_identity)
    }

    pub fn evidence_identity_ref(&self) -> &WorthQueryEvidenceIdentity {
        &self.report_identity
    }

    pub fn activation_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.activation_identity)
    }

    pub fn activation_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.activation_identity
    }

    pub fn admission_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.admission_identity)
    }

    pub fn admission_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.admission_identity
    }

    pub fn small_snapshot_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.small_snapshot_identity)
    }

    pub fn medium_snapshot_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.medium_snapshot_identity)
    }

    pub fn large_snapshot_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.large_snapshot_identity)
    }

    pub fn small_row_count(&self) -> u64 {
        self.small_row_count
    }

    pub fn medium_row_count(&self) -> u64 {
        self.medium_row_count
    }

    pub fn large_row_count(&self) -> u64 {
        self.large_row_count
    }

    pub fn structural_counter_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.structural_counter_identity)
    }

    pub fn structural_counter_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.structural_counter_identity
    }

    pub fn evidence_identity(&self) -> WorthQueryEvidenceIdentity {
        self.report_identity.clone()
    }
}

pub fn certify_query_subscription_scale_slope(
    small: QuerySubscriptionScaleCounterSnapshot,
    medium: QuerySubscriptionScaleCounterSnapshot,
    large: QuerySubscriptionScaleCounterSnapshot,
) -> Result<QuerySubscriptionScaleSlopeReport, QuerySubscriptionCertificationError> {
    if small.fixture_size != QuerySubscriptionScaleFixtureSize::Small
        || medium.fixture_size != QuerySubscriptionScaleFixtureSize::Medium
        || large.fixture_size != QuerySubscriptionScaleFixtureSize::Large
        || small.fixture_row_count == 0
        || !(small.fixture_row_count < medium.fixture_row_count
            && medium.fixture_row_count < large.fixture_row_count)
        || typed_identity_drift(small.activation_identity(), medium.activation_identity())
        || typed_identity_drift(medium.activation_identity(), large.activation_identity())
        || typed_identity_drift(small.admission_identity(), medium.admission_identity())
        || typed_identity_drift(medium.admission_identity(), large.admission_identity())
        || small.counters != medium.counters
        || medium.counters != large.counters
    {
        return Err(QuerySubscriptionCertificationError::new(
            QuerySubscriptionCertificationDenialKind::ScaleSlopeDrift,
            "subscription structural counters must remain stable across row-count-only fixture scale",
            &[
                validation_role_evidence_identity("small", small.snapshot_identity()),
                validation_role_evidence_identity("medium", medium.snapshot_identity()),
                validation_role_evidence_identity("large", large.snapshot_identity()),
                validation_u64_role_evidence_identity("small_row_count", small.fixture_row_count()),
                validation_u64_role_evidence_identity(
                    "medium_row_count",
                    medium.fixture_row_count(),
                ),
                validation_u64_role_evidence_identity("large_row_count", large.fixture_row_count()),
                validation_role_evidence_identity("small_activation", small.activation_identity()),
                validation_role_evidence_identity(
                    "medium_activation",
                    medium.activation_identity(),
                ),
                validation_role_evidence_identity("large_activation", large.activation_identity()),
                validation_role_evidence_identity("small_admission", small.admission_identity()),
                validation_role_evidence_identity("medium_admission", medium.admission_identity()),
                validation_role_evidence_identity("large_admission", large.admission_identity()),
                validation_role_evidence_identity("small_counters", small.counter_identity()),
                validation_role_evidence_identity("medium_counters", medium.counter_identity()),
                validation_role_evidence_identity("large_counters", large.counter_identity()),
            ],
        ));
    }

    let structural_counter_identity = small.counter_identity.clone();
    let activation_identity = small.activation_identity.clone();
    let admission_identity = small.admission_identity.clone();
    let report_identity = scale_slope_report_identity(
        &activation_identity,
        &admission_identity,
        &small.snapshot_identity,
        &medium.snapshot_identity,
        &large.snapshot_identity,
        small.fixture_row_count,
        medium.fixture_row_count,
        large.fixture_row_count,
        &structural_counter_identity,
    );

    Ok(QuerySubscriptionScaleSlopeReport {
        report_identity,
        activation_identity,
        admission_identity,
        small_snapshot_identity: small.snapshot_identity,
        medium_snapshot_identity: medium.snapshot_identity,
        large_snapshot_identity: large.snapshot_identity,
        small_row_count: small.fixture_row_count,
        medium_row_count: medium.fixture_row_count,
        large_row_count: large.fixture_row_count,
        structural_counter_identity,
    })
}
