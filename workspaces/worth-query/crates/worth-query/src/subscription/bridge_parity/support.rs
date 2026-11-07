use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::identity_authority::{QueryProjectionIdentity, QuerySubscriptionIdentityKind};

use super::super::evidence_projection::subscription_evidence_projection;
use super::identities::{bridge_parity_counter_identity, bridge_parity_failure_identity};
#[cfg(test)]
use super::identities::{bridge_parity_receipt_identity, bridge_parity_width_identity};
use super::witness::BridgeWitnessAssemblyPosture;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuerySubscriptionBridgeParityClass {
    ExactParity,
    FamilyDistinctBridgeShared,
    DeniedSourceMismatch,
    DeniedUnsupported,
}

impl QuerySubscriptionBridgeParityClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ExactParity => "exact_parity",
            Self::FamilyDistinctBridgeShared => "family_distinct_bridge_shared",
            Self::DeniedSourceMismatch => "denied_source_mismatch",
            Self::DeniedUnsupported => "denied_unsupported",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuerySubscriptionBridgeParityFailureKind {
    DeclarationMismatch,
    BridgeMismatch,
    BasisMismatch,
    SignalStrategyMismatch,
    ActivationMismatch,
}

impl QuerySubscriptionBridgeParityFailureKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DeclarationMismatch => "declaration_mismatch",
            Self::BridgeMismatch => "bridge_mismatch",
            Self::BasisMismatch => "basis_mismatch",
            Self::SignalStrategyMismatch => "signal_strategy_mismatch",
            Self::ActivationMismatch => "activation_mismatch",
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct QuerySubscriptionBridgeParityCounters {
    subscription_bridge_parity_comparison_count: u64,
    subscription_bridge_parity_admitted_count: u64,
    subscription_bridge_parity_denial_count: u64,
    subscription_bridge_family_distinction_preservation_count: u64,
}

impl QuerySubscriptionBridgeParityCounters {
    pub fn counter_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.counter_identity())
    }

    pub fn counter_identity(&self) -> WorthQueryEvidenceIdentity {
        bridge_parity_counter_identity(self)
    }

    pub fn subscription_bridge_parity_comparison_count(&self) -> u64 {
        self.subscription_bridge_parity_comparison_count
    }

    pub fn subscription_bridge_parity_admitted_count(&self) -> u64 {
        self.subscription_bridge_parity_admitted_count
    }

    pub fn subscription_bridge_parity_denial_count(&self) -> u64 {
        self.subscription_bridge_parity_denial_count
    }

    pub fn subscription_bridge_family_distinction_preservation_count(&self) -> u64 {
        self.subscription_bridge_family_distinction_preservation_count
    }

    #[cfg(test)]
    pub(crate) fn admitted(parity_class: QuerySubscriptionBridgeParityClass) -> Self {
        Self {
            subscription_bridge_parity_comparison_count: 1,
            subscription_bridge_parity_admitted_count: 1,
            subscription_bridge_family_distinction_preservation_count: u64::from(
                parity_class == QuerySubscriptionBridgeParityClass::FamilyDistinctBridgeShared,
            ),
            ..Default::default()
        }
    }

    pub(crate) fn denied() -> Self {
        Self {
            subscription_bridge_parity_comparison_count: 1,
            subscription_bridge_parity_denial_count: 1,
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionBridgeParityWidth {
    compared_family_dimension_count: usize,
    compared_slice_dimension_count: usize,
    compared_basis_dimension_count: usize,
    compared_signal_dimension_count: usize,
    width_identity: WorthQueryEvidenceIdentity,
}

impl SubscriptionBridgeParityWidth {
    #[cfg(test)]
    pub(super) fn new(
        compared_family_dimension_count: usize,
        compared_slice_dimension_count: usize,
        compared_basis_dimension_count: usize,
        compared_signal_dimension_count: usize,
    ) -> Result<Self, IdentityDerivationDenial> {
        let width_identity = bridge_parity_width_identity(
            compared_family_dimension_count,
            compared_slice_dimension_count,
            compared_basis_dimension_count,
            compared_signal_dimension_count,
        );
        Ok(Self {
            compared_family_dimension_count,
            compared_slice_dimension_count,
            compared_basis_dimension_count,
            compared_signal_dimension_count,
            width_identity,
        })
    }

    pub fn compared_family_dimension_count(&self) -> usize {
        self.compared_family_dimension_count
    }

    pub fn compared_slice_dimension_count(&self) -> usize {
        self.compared_slice_dimension_count
    }

    pub fn compared_basis_dimension_count(&self) -> usize {
        self.compared_basis_dimension_count
    }

    pub fn compared_signal_dimension_count(&self) -> usize {
        self.compared_signal_dimension_count
    }

    pub fn width_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.width_identity)
    }

    pub fn width_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.width_identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeParityReceipt {
    witness_assembly_posture: BridgeWitnessAssemblyPosture,
    parity_class: QuerySubscriptionBridgeParityClass,
    comparison_width: SubscriptionBridgeParityWidth,
    semantic_rebuild_count: usize,
    receipt_identity: WorthQueryEvidenceIdentity,
}

impl BridgeParityReceipt {
    #[cfg(test)]
    pub(super) fn new(
        witness_assembly_posture: BridgeWitnessAssemblyPosture,
        parity_class: QuerySubscriptionBridgeParityClass,
        comparison_width: SubscriptionBridgeParityWidth,
        semantic_rebuild_count: usize,
    ) -> Result<Self, IdentityDerivationDenial> {
        let receipt_identity = bridge_parity_receipt_identity(
            witness_assembly_posture,
            parity_class,
            comparison_width.width_identity(),
            semantic_rebuild_count,
        );
        Ok(Self {
            witness_assembly_posture,
            parity_class,
            comparison_width,
            semantic_rebuild_count,
            receipt_identity,
        })
    }

    pub fn witness_assembly_posture(&self) -> &BridgeWitnessAssemblyPosture {
        &self.witness_assembly_posture
    }

    pub fn parity_class(&self) -> &QuerySubscriptionBridgeParityClass {
        &self.parity_class
    }

    pub fn comparison_width(&self) -> &SubscriptionBridgeParityWidth {
        &self.comparison_width
    }

    pub fn semantic_rebuild_count(&self) -> usize {
        self.semantic_rebuild_count
    }

    pub fn receipt_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.receipt_identity)
    }

    pub fn receipt_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.receipt_identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionBridgeParityFailure {
    failure_kind: QuerySubscriptionBridgeParityFailureKind,
    parity_class: QuerySubscriptionBridgeParityClass,
    reason: String,
    pub(in crate::subscription) source_identity: WorthQueryEvidenceIdentity,
    failure_identity: WorthQueryEvidenceIdentity,
}

impl QuerySubscriptionBridgeParityFailure {
    pub(crate) fn new(
        failure_kind: QuerySubscriptionBridgeParityFailureKind,
        parity_class: QuerySubscriptionBridgeParityClass,
        reason: impl Into<String>,
        source_identity: WorthQueryEvidenceIdentity,
        evidence: &[WorthQueryEvidenceIdentity],
    ) -> Result<Self, IdentityDerivationDenial> {
        let reason = reason.into();
        let failure_identity = bridge_parity_failure_identity(
            failure_kind,
            parity_class,
            &reason,
            &source_identity,
            evidence,
        )?;
        Ok(Self {
            failure_kind,
            parity_class,
            reason,
            source_identity,
            failure_identity,
        })
    }

    pub fn failure_kind(&self) -> &QuerySubscriptionBridgeParityFailureKind {
        &self.failure_kind
    }

    pub fn parity_class(&self) -> &QuerySubscriptionBridgeParityClass {
        &self.parity_class
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }

    pub fn failure_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.failure_identity)
    }

    pub fn failure_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.failure_identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionBridgeParityError {
    failure: QuerySubscriptionBridgeParityFailure,
    counters: QuerySubscriptionBridgeParityCounters,
}

impl QuerySubscriptionBridgeParityError {
    pub(crate) fn new(
        failure: QuerySubscriptionBridgeParityFailure,
        counters: QuerySubscriptionBridgeParityCounters,
    ) -> Self {
        Self { failure, counters }
    }

    pub fn failure(&self) -> &QuerySubscriptionBridgeParityFailure {
        &self.failure
    }

    pub fn counters(&self) -> &QuerySubscriptionBridgeParityCounters {
        &self.counters
    }

    pub fn message(&self) -> &str {
        self.failure.reason()
    }
}
