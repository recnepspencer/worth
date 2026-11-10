use crate::basis_lifecycle::{
    activate_subscription_basis, ScopedSubscriptionActivationBasis,
    ScopedSubscriptionDeclarationBasis,
};
use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::identity_authority::{
    project_query_subscription_evidence, QueryProjectionIdentity, QuerySubscriptionIdentityKind,
};

use super::admission::QuerySubscriptionAdmissionArtifact;
use super::counters::QuerySubscriptionDeclarationCounters;
use super::evidence_identities::{activation_checkpoint_identity, activation_input_identity};
use super::future_selection::QuerySubscriptionFutureSelection;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionActivationInput {
    activation_identity: WorthQueryEvidenceIdentity,
    admission_identity: WorthQueryEvidenceIdentity,
    query_declaration_identity: WorthQueryEvidenceIdentity,
    bridge_declaration_identity: WorthQueryEvidenceIdentity,
    future_selection: QuerySubscriptionFutureSelection,
    basis_binding_identity: WorthQueryEvidenceIdentity,
    scoped_declaration_basis: ScopedSubscriptionDeclarationBasis,
    scoped_activation_basis: ScopedSubscriptionActivationBasis,
    checkpoint_identity: WorthQueryEvidenceIdentity,
    signal_strategy_identity: WorthQueryEvidenceIdentity,
    counters: QuerySubscriptionDeclarationCounters,
}

impl SubscriptionActivationInput {
    pub fn activation_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        project_query_subscription_evidence(&self.activation_identity)
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.activation_identity
    }

    pub fn admission_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        project_query_subscription_evidence(&self.admission_identity)
    }

    pub fn admission_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.admission_identity
    }

    pub fn query_declaration_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        project_query_subscription_evidence(&self.query_declaration_identity)
    }

    pub fn query_declaration_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.query_declaration_identity
    }

    pub fn bridge_declaration_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        project_query_subscription_evidence(&self.bridge_declaration_identity)
    }

    pub fn bridge_declaration_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.bridge_declaration_identity
    }

    pub fn future_selection(&self) -> &QuerySubscriptionFutureSelection {
        &self.future_selection
    }

    pub fn basis_binding_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        project_query_subscription_evidence(&self.basis_binding_identity)
    }

    pub fn basis_binding_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.basis_binding_identity
    }

    pub fn scoped_declaration_basis(&self) -> &ScopedSubscriptionDeclarationBasis {
        &self.scoped_declaration_basis
    }

    pub fn scoped_activation_basis(&self) -> &ScopedSubscriptionActivationBasis {
        &self.scoped_activation_basis
    }

    pub fn checkpoint_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        project_query_subscription_evidence(&self.checkpoint_identity)
    }

    pub fn checkpoint_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.checkpoint_identity
    }

    pub fn signal_strategy_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        project_query_subscription_evidence(&self.signal_strategy_identity)
    }

    pub fn signal_strategy_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.signal_strategy_identity
    }

    pub fn counters(&self) -> &QuerySubscriptionDeclarationCounters {
        &self.counters
    }
}

pub fn prepare_subscription_activation(
    admission: QuerySubscriptionAdmissionArtifact,
) -> SubscriptionActivationInput {
    let mut counters = admission.counters().clone();
    counters.activation_input_count = 1;
    let checkpoint_identity = activation_checkpoint_identity(
        admission.query_declaration_identity(),
        admission.basis_binding_identity(),
        admission.future_selection().projection_identity(),
        admission.signal_strategy_identity(),
    );
    let activation_identity = activation_input_identity(
        admission.evidence_identity(),
        admission.query_declaration_identity(),
        admission.bridge_declaration_identity(),
        admission.basis_binding_identity(),
        &checkpoint_identity,
        admission.signal_strategy_identity(),
        admission.future_selection().projection_identity(),
        &counters.evidence_identity(),
    );
    let scoped_activation_basis = activate_subscription_basis(admission.scoped_declaration_basis());
    SubscriptionActivationInput {
        activation_identity,
        admission_identity: admission.evidence_identity().clone(),
        query_declaration_identity: admission.query_declaration_identity().clone(),
        bridge_declaration_identity: admission.bridge_declaration_identity().clone(),
        future_selection: admission.future_selection().clone(),
        basis_binding_identity: admission.basis_binding_identity().clone(),
        scoped_declaration_basis: admission.scoped_declaration_basis().clone(),
        scoped_activation_basis,
        checkpoint_identity,
        signal_strategy_identity: admission.signal_strategy_identity().clone(),
        counters,
    }
}
