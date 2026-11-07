use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::identity_authority::{QueryProjectionIdentity, QuerySubscriptionIdentityKind};

#[cfg(test)]
use super::super::activation::SubscriptionActivationInput;
#[cfg(test)]
use super::super::bridge_lowering::BridgeSubscriptionLoweringPlan;
#[cfg(test)]
use super::super::declaration::QuerySubscriptionDeclarationArtifact;
use super::super::evidence_projection::subscription_evidence_projection;
#[cfg(test)]
use super::identities::{bridge_parity_comparison_identity, bridge_parity_explanation_identity};
#[cfg(test)]
use super::support::{
    BridgeParityReceipt, QuerySubscriptionBridgeParityError, SubscriptionBridgeParityWidth,
};
use super::support::{QuerySubscriptionBridgeParityClass, QuerySubscriptionBridgeParityCounters};
#[cfg(test)]
use super::validation::{
    parity_class_for_family, validate_parity_sources, CanonicalBridgeParitySemantics,
};
use super::witness::QuerySubscriptionManualBridgeWitness;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionBridgeParityComparison {
    parity_class: QuerySubscriptionBridgeParityClass,
    query_declaration_identity: WorthQueryEvidenceIdentity,
    bridge_declaration_identity: WorthQueryEvidenceIdentity,
    witness_identity: WorthQueryEvidenceIdentity,
    activation_identity: WorthQueryEvidenceIdentity,
    comparison_identity: WorthQueryEvidenceIdentity,
}

impl QuerySubscriptionBridgeParityComparison {
    pub fn parity_class(&self) -> &QuerySubscriptionBridgeParityClass {
        &self.parity_class
    }

    pub fn query_declaration_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.query_declaration_identity)
    }

    pub fn query_declaration_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.query_declaration_identity
    }

    pub fn bridge_declaration_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.bridge_declaration_identity)
    }

    pub fn bridge_declaration_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.bridge_declaration_identity
    }

    pub fn witness_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.witness_identity)
    }

    pub fn witness_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.witness_identity
    }

    pub fn activation_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.activation_identity)
    }

    pub fn activation_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.activation_identity
    }

    pub fn comparison_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.comparison_identity)
    }

    pub fn comparison_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.comparison_identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionBridgeParityExplanation {
    comparison: QuerySubscriptionBridgeParityComparison,
    witness: QuerySubscriptionManualBridgeWitness,
    query_family_label: String,
    declaration_family_label: String,
    bridge_family_label: String,
    bridge_slice_labels: Vec<String>,
    basis_posture_label: String,
    signal_strategy_class_label: String,
    counter_identity: WorthQueryEvidenceIdentity,
    explanation_identity: WorthQueryEvidenceIdentity,
    counters: QuerySubscriptionBridgeParityCounters,
}

impl QuerySubscriptionBridgeParityExplanation {
    pub fn comparison(&self) -> &QuerySubscriptionBridgeParityComparison {
        &self.comparison
    }

    pub fn witness(&self) -> &QuerySubscriptionManualBridgeWitness {
        &self.witness
    }

    pub fn query_family_label(&self) -> &str {
        &self.query_family_label
    }

    pub fn declaration_family_label(&self) -> &str {
        &self.declaration_family_label
    }

    pub fn bridge_family_label(&self) -> &str {
        &self.bridge_family_label
    }

    pub fn bridge_slice_labels(&self) -> &[String] {
        &self.bridge_slice_labels
    }

    pub fn basis_posture_label(&self) -> &str {
        &self.basis_posture_label
    }

    pub fn signal_strategy_class_label(&self) -> &str {
        &self.signal_strategy_class_label
    }

    pub fn counter_snapshot_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.counter_identity)
    }

    pub fn counter_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.counter_identity
    }

    pub fn explanation_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.explanation_identity)
    }

    pub fn explanation_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.explanation_identity
    }

    pub fn counters(&self) -> &QuerySubscriptionBridgeParityCounters {
        &self.counters
    }
}

#[cfg(test)]
pub fn explain_query_subscription_bridge_parity(
    declaration: &QuerySubscriptionDeclarationArtifact,
    lowering: &BridgeSubscriptionLoweringPlan,
    activation: &SubscriptionActivationInput,
    witness: QuerySubscriptionManualBridgeWitness,
) -> Result<
    (
        QuerySubscriptionBridgeParityExplanation,
        BridgeParityReceipt,
    ),
    QuerySubscriptionBridgeParityError,
> {
    let semantics =
        CanonicalBridgeParitySemantics::from_authoritative_sources(declaration, lowering);
    validate_parity_sources(declaration, lowering, activation, &witness, &semantics)?;

    let parity_class = parity_class_for_family(declaration.family());
    let comparison_width =
        SubscriptionBridgeParityWidth::new(3, lowering.bridge_slices().len(), 1, 1).map_err(
            |cause| {
                QuerySubscriptionBridgeParityError::identity_refused(
                    cause,
                    witness.evidence_identity().clone(),
                )
            },
        )?;
    let receipt = BridgeParityReceipt::new(
        *witness.assembly_posture(),
        parity_class,
        comparison_width,
        0,
    )
    .map_err(|cause| {
        QuerySubscriptionBridgeParityError::identity_refused(
            cause,
            witness.evidence_identity().clone(),
        )
    })?;
    let counters = QuerySubscriptionBridgeParityCounters::admitted(parity_class);
    let counter_identity = counters.counter_identity();
    let comparison_identity = bridge_parity_comparison_identity(
        parity_class,
        declaration.declaration_identity(),
        lowering.bridge_declaration_identity(),
        witness.evidence_identity(),
        activation.evidence_identity(),
    );
    let comparison = QuerySubscriptionBridgeParityComparison {
        parity_class,
        query_declaration_identity: declaration.declaration_identity().clone(),
        bridge_declaration_identity: lowering.bridge_declaration_identity().clone(),
        witness_identity: witness.evidence_identity().clone(),
        activation_identity: activation.evidence_identity().clone(),
        comparison_identity,
    };
    let explanation_identity = bridge_parity_explanation_identity(
        comparison.comparison_identity(),
        witness.evidence_identity(),
        receipt.receipt_identity(),
        &counter_identity,
    );

    Ok((
        QuerySubscriptionBridgeParityExplanation {
            comparison,
            query_family_label: semantics.query_family_label,
            declaration_family_label: semantics.declaration_family_label,
            bridge_family_label: semantics.bridge_family_label,
            bridge_slice_labels: semantics.bridge_slice_labels,
            basis_posture_label: semantics.basis_posture_label,
            signal_strategy_class_label: semantics.signal_strategy_class_label,
            counter_identity,
            explanation_identity,
            counters,
            witness,
        },
        receipt,
    ))
}
