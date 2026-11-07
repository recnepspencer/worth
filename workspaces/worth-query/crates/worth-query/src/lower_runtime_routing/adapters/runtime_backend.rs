use crate::declarative_live::DeclarativeLiveQueryRequest;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::memory_workspace::WorthQueryMutationReceipt;
use crate::subscription::SubscriptionActivationInput;

use crate::lower_runtime_routing::{
    WorthQueryLowerRuntimeAuthorityOwner, WorthQueryLowerRuntimeBoundaryEnvelope,
    WorthQueryLowerRuntimeBoundaryExecutionReceipt, WorthQueryLowerRuntimeCapabilityEligibility,
    WorthQueryLowerRuntimeCapabilityRequest, WorthQueryLowerRuntimeReadmissionReceipt,
    WorthQueryLowerRuntimeRetainedEvidenceIdentity, WorthQueryLowerRuntimeRouteKind,
    WorthQueryLowerRuntimeRoutePlan, WorthQueryLowerRuntimeRouteSubjectIdentity,
    WorthQueryLowerRuntimeSeamKey,
};
use crate::runtime::{
    LiveViewDeclarationAdmissionReceipt, SignalInvalidationRoutingReceipt,
    SubscriptionActivationReceipt,
};

use self::subject_digest::{
    activation_subject_identity, live_view_subject_identity, signal_invalidation_subject_identity,
};

mod subject_digest;
mod write_authority_receipt;

pub use write_authority_receipt::WriteAuthorityExecutionReceipt;

const LIVE_VIEW_CAPABILITY_LABEL: &str = "live-view-schema-admission";
const SIGNAL_INVALIDATION_CAPABILITY_LABEL: &str = "signal-invalidation-routing";
const SUBSCRIPTION_ACTIVATION_CAPABILITY_LABEL: &str = "subscription-activation";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveViewDeclarationAdmissionBoundaryReceipt {
    admission_receipt: LiveViewDeclarationAdmissionReceipt,
    readmission_receipt: WorthQueryLowerRuntimeReadmissionReceipt,
    boundary_execution_receipt: WorthQueryLowerRuntimeBoundaryExecutionReceipt,
    boundary_envelope: WorthQueryLowerRuntimeBoundaryEnvelope,
}

impl LiveViewDeclarationAdmissionBoundaryReceipt {
    pub(crate) fn from_request(
        view_name: &str,
        request: &DeclarativeLiveQueryRequest,
        admission_receipt: LiveViewDeclarationAdmissionReceipt,
    ) -> Result<Self, IdentityDerivationDenial> {
        let capability_request = WorthQueryLowerRuntimeCapabilityRequest::new(
            WorthQueryLowerRuntimeSeamKey::LiveViewSchemaAdmission,
            WorthQueryLowerRuntimeRouteKind::ReadmissionHandoff,
            WorthQueryLowerRuntimeAuthorityOwner::Query,
            LIVE_VIEW_CAPABILITY_LABEL,
            live_view_subject_identity(view_name, request)?,
        )?;
        let eligibility =
            WorthQueryLowerRuntimeCapabilityEligibility::admitted_with_evidence_identity(
                capability_request,
                admission_receipt.receipt_identity(),
            )?;
        let readmission_receipt = WorthQueryLowerRuntimeReadmissionReceipt::new(
            eligibility,
            &WorthQueryLowerRuntimeRetainedEvidenceIdentity::from_evidence_identity(
                "live-view-declaration-admission",
                admission_receipt.receipt_identity(),
            )?,
        );
        let boundary_execution_receipt =
            WorthQueryLowerRuntimeBoundaryExecutionReceipt::from_readmission_receipt(
                &readmission_receipt,
            );
        let boundary_envelope = WorthQueryLowerRuntimeBoundaryEnvelope::from_readmission_receipt(
            WorthQueryLowerRuntimeSeamKey::LiveViewSchemaAdmission,
            &readmission_receipt,
            &boundary_execution_receipt,
        );
        Ok(Self {
            admission_receipt,
            readmission_receipt,
            boundary_execution_receipt,
            boundary_envelope,
        })
    }

    pub fn admission_receipt(&self) -> &LiveViewDeclarationAdmissionReceipt {
        &self.admission_receipt
    }

    pub fn readmission_receipt(&self) -> &WorthQueryLowerRuntimeReadmissionReceipt {
        &self.readmission_receipt
    }

    pub fn boundary_execution_receipt(&self) -> &WorthQueryLowerRuntimeBoundaryExecutionReceipt {
        &self.boundary_execution_receipt
    }

    pub fn boundary_envelope(&self) -> &WorthQueryLowerRuntimeBoundaryEnvelope {
        &self.boundary_envelope
    }

    pub(crate) fn drift_from_request(
        &self,
        view_name: &str,
        request: &DeclarativeLiveQueryRequest,
    ) -> Result<Option<String>, IdentityDerivationDenial> {
        if let Some(message) = self
            .admission_receipt
            .drift_from_request(view_name, request)?
        {
            return Ok(Some(message));
        }
        let expected_subject = live_view_subject_identity(view_name, request)?;
        if let Some(message) = self
            .readmission_receipt
            .eligibility()
            .request()
            .drift_from_contract(
                WorthQueryLowerRuntimeSeamKey::LiveViewSchemaAdmission,
                WorthQueryLowerRuntimeRouteKind::ReadmissionHandoff,
                WorthQueryLowerRuntimeAuthorityOwner::Query,
                LIVE_VIEW_CAPABILITY_LABEL,
                &expected_subject,
            )
        {
            return Ok(Some(message));
        }
        Ok(self
            .boundary_execution_receipt
            .drift_from_readmission_receipt(&self.readmission_receipt))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalInvalidationBoundaryReceipt {
    routing_receipt: SignalInvalidationRoutingReceipt,
    route_plan: WorthQueryLowerRuntimeRoutePlan,
    boundary_execution_receipt: WorthQueryLowerRuntimeBoundaryExecutionReceipt,
    boundary_envelope: WorthQueryLowerRuntimeBoundaryEnvelope,
}

impl SignalInvalidationBoundaryReceipt {
    pub(crate) fn from_mutation_receipt(
        _mutation_receipt: &WorthQueryMutationReceipt,
        routing_receipt: SignalInvalidationRoutingReceipt,
    ) -> Result<Self, IdentityDerivationDenial> {
        let capability_request = WorthQueryLowerRuntimeCapabilityRequest::new(
            WorthQueryLowerRuntimeSeamKey::SignalInvalidationRouting,
            WorthQueryLowerRuntimeRouteKind::RoutePlanning,
            WorthQueryLowerRuntimeAuthorityOwner::Query,
            SIGNAL_INVALIDATION_CAPABILITY_LABEL,
            signal_invalidation_subject_identity(&routing_receipt)?,
        )?;
        let eligibility =
            WorthQueryLowerRuntimeCapabilityEligibility::admitted_with_evidence_identity(
                capability_request,
                routing_receipt.receipt_identity(),
            )?;
        let route_plan = WorthQueryLowerRuntimeRoutePlan::new(
            eligibility,
            WorthQueryLowerRuntimeRouteSubjectIdentity::from_evidence_identity(
                "signal-invalidation-route",
                routing_receipt.receipt_identity(),
            )?,
        );
        let retained_evidence_identity =
            WorthQueryLowerRuntimeRetainedEvidenceIdentity::from_evidence_identity(
                "signal-invalidation-routing",
                routing_receipt.receipt_identity(),
            )?;
        let boundary_execution_receipt =
            WorthQueryLowerRuntimeBoundaryExecutionReceipt::from_route_plan_with_retained_evidence_identity(
                &route_plan,
                &retained_evidence_identity,
            );
        let boundary_envelope =
            WorthQueryLowerRuntimeBoundaryEnvelope::from_route_plan_with_retained_evidence_identity(
                WorthQueryLowerRuntimeSeamKey::SignalInvalidationRouting,
                &route_plan,
                &boundary_execution_receipt,
                &retained_evidence_identity,
            );
        Ok(Self {
            routing_receipt,
            route_plan,
            boundary_execution_receipt,
            boundary_envelope,
        })
    }

    pub fn routing_receipt(&self) -> &SignalInvalidationRoutingReceipt {
        &self.routing_receipt
    }

    pub fn route_plan(&self) -> &WorthQueryLowerRuntimeRoutePlan {
        &self.route_plan
    }

    pub fn boundary_execution_receipt(&self) -> &WorthQueryLowerRuntimeBoundaryExecutionReceipt {
        &self.boundary_execution_receipt
    }

    pub fn boundary_envelope(&self) -> &WorthQueryLowerRuntimeBoundaryEnvelope {
        &self.boundary_envelope
    }

    pub(crate) fn drift_from_mutation_receipt(
        &self,
        mutation_receipt: &WorthQueryMutationReceipt,
    ) -> Result<Option<String>, IdentityDerivationDenial> {
        if let Some(message) = self
            .routing_receipt
            .drift_from_mutation_receipt(mutation_receipt)?
        {
            return Ok(Some(message));
        }
        if let Some(message) = self.route_plan.eligibility().request().drift_from_contract(
            WorthQueryLowerRuntimeSeamKey::SignalInvalidationRouting,
            WorthQueryLowerRuntimeRouteKind::RoutePlanning,
            WorthQueryLowerRuntimeAuthorityOwner::Query,
            SIGNAL_INVALIDATION_CAPABILITY_LABEL,
            &signal_invalidation_subject_identity(&self.routing_receipt)?,
        ) {
            return Ok(Some(message));
        }
        Ok(self
            .boundary_execution_receipt
            .drift_from_route_plan_with_retained_evidence_identity(
                &self.route_plan,
                &WorthQueryLowerRuntimeRetainedEvidenceIdentity::from_evidence_identity(
                    "signal-invalidation-routing",
                    self.routing_receipt.receipt_identity(),
                )?,
            ))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionActivationBoundaryReceipt {
    activation_receipt: SubscriptionActivationReceipt,
    route_plan: WorthQueryLowerRuntimeRoutePlan,
    boundary_execution_receipt: WorthQueryLowerRuntimeBoundaryExecutionReceipt,
    boundary_envelope: WorthQueryLowerRuntimeBoundaryEnvelope,
}

impl SubscriptionActivationBoundaryReceipt {
    pub(crate) fn from_activation(
        view_name: &str,
        activation: &SubscriptionActivationInput,
        activation_receipt: SubscriptionActivationReceipt,
    ) -> Result<Self, IdentityDerivationDenial> {
        let capability_request = WorthQueryLowerRuntimeCapabilityRequest::new(
            WorthQueryLowerRuntimeSeamKey::SubscriptionActivation,
            WorthQueryLowerRuntimeRouteKind::RoutePlanning,
            WorthQueryLowerRuntimeAuthorityOwner::Query,
            SUBSCRIPTION_ACTIVATION_CAPABILITY_LABEL,
            activation_subject_identity(view_name, activation, &activation_receipt)?,
        )?;
        let eligibility =
            WorthQueryLowerRuntimeCapabilityEligibility::admitted_with_evidence_identity(
                capability_request,
                activation_receipt.receipt_identity(),
            )?;
        let route_plan = WorthQueryLowerRuntimeRoutePlan::new(
            eligibility,
            WorthQueryLowerRuntimeRouteSubjectIdentity::from_evidence_identity(
                "subscription-activation-route",
                activation_receipt.receipt_identity(),
            )?,
        );
        let retained_evidence_identity =
            WorthQueryLowerRuntimeRetainedEvidenceIdentity::from_evidence_identity(
                "subscription-activation",
                activation_receipt.receipt_identity(),
            )?;
        let boundary_execution_receipt =
            WorthQueryLowerRuntimeBoundaryExecutionReceipt::from_route_plan_with_retained_evidence_identity(
                &route_plan,
                &retained_evidence_identity,
            );
        let boundary_envelope =
            WorthQueryLowerRuntimeBoundaryEnvelope::from_route_plan_with_retained_evidence_identity(
                WorthQueryLowerRuntimeSeamKey::SubscriptionActivation,
                &route_plan,
                &boundary_execution_receipt,
                &retained_evidence_identity,
            );
        Ok(Self {
            activation_receipt,
            route_plan,
            boundary_execution_receipt,
            boundary_envelope,
        })
    }

    pub fn activation_receipt(&self) -> &SubscriptionActivationReceipt {
        &self.activation_receipt
    }

    pub fn route_plan(&self) -> &WorthQueryLowerRuntimeRoutePlan {
        &self.route_plan
    }

    pub fn boundary_execution_receipt(&self) -> &WorthQueryLowerRuntimeBoundaryExecutionReceipt {
        &self.boundary_execution_receipt
    }

    pub fn boundary_envelope(&self) -> &WorthQueryLowerRuntimeBoundaryEnvelope {
        &self.boundary_envelope
    }

    pub(crate) fn drift_from_activation(
        &self,
        view_name: &str,
        activation: &SubscriptionActivationInput,
    ) -> Result<Option<String>, IdentityDerivationDenial> {
        if let Some(message) = self
            .activation_receipt
            .drift_from_activation(view_name, activation)
        {
            return Ok(Some(message));
        }
        let expected_subject =
            activation_subject_identity(view_name, activation, &self.activation_receipt)?;
        if let Some(message) = self.route_plan.eligibility().request().drift_from_contract(
            WorthQueryLowerRuntimeSeamKey::SubscriptionActivation,
            WorthQueryLowerRuntimeRouteKind::RoutePlanning,
            WorthQueryLowerRuntimeAuthorityOwner::Query,
            SUBSCRIPTION_ACTIVATION_CAPABILITY_LABEL,
            &expected_subject,
        ) {
            return Ok(Some(message));
        }
        Ok(self
            .boundary_execution_receipt
            .drift_from_route_plan_with_retained_evidence_identity(
                &self.route_plan,
                &WorthQueryLowerRuntimeRetainedEvidenceIdentity::from_evidence_identity(
                    "subscription-activation",
                    self.activation_receipt.receipt_identity(),
                )?,
            ))
    }
}

#[cfg(test)]
mod tests;
