use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::lower_runtime_routing::{
    WorthQueryLowerRuntimeAuthorityOwner, WorthQueryLowerRuntimeBoundaryEnvelope,
    WorthQueryLowerRuntimeBoundaryExecutionReceipt, WorthQueryLowerRuntimeCapabilityEligibility,
    WorthQueryLowerRuntimeCapabilityRequest, WorthQueryLowerRuntimeRetainedEvidenceIdentity,
    WorthQueryLowerRuntimeRouteKind, WorthQueryLowerRuntimeRoutePlan,
    WorthQueryLowerRuntimeRouteSubjectIdentity, WorthQueryLowerRuntimeSeamKey,
    WorthQueryLowerRuntimeSubjectIdentity,
};
use crate::memory_workspace::WorthQueryMutationReceipt;
use crate::runtime::{WorthQueryBackendAdmissibleMutation, WorthQueryWriteCommand};

use super::subject_digest::{
    backend_admissible_mutation_subject_identity, write_command_subject_identity,
};

pub(super) const WRITE_AUTHORITY_CAPABILITY_LABEL: &str = "write-authority-backend-execution";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WriteAuthorityExecutionReceipt {
    mutation_receipt: WorthQueryMutationReceipt,
    route_plan: WorthQueryLowerRuntimeRoutePlan,
    boundary_execution_receipt: WorthQueryLowerRuntimeBoundaryExecutionReceipt,
    boundary_envelope: WorthQueryLowerRuntimeBoundaryEnvelope,
}

impl WriteAuthorityExecutionReceipt {
    pub(crate) fn from_backend_admissible_mutation(
        mutation: &WorthQueryBackendAdmissibleMutation,
        mutation_receipt: WorthQueryMutationReceipt,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::from_subject_identity(
            backend_admissible_mutation_subject_identity(mutation)?,
            mutation_receipt,
        )?)
    }

    pub(crate) fn from_command(
        command: &WorthQueryWriteCommand,
        mutation_receipt: WorthQueryMutationReceipt,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::from_subject_identity(
            write_command_subject_identity(command)?,
            mutation_receipt,
        )?)
    }

    fn from_subject_identity(
        subject_identity: WorthQueryLowerRuntimeSubjectIdentity,
        mutation_receipt: WorthQueryMutationReceipt,
    ) -> Result<Self, IdentityDerivationDenial> {
        let capability_request = WorthQueryLowerRuntimeCapabilityRequest::new(
            WorthQueryLowerRuntimeSeamKey::WriteAuthorityBackendExecution,
            WorthQueryLowerRuntimeRouteKind::RoutePlanning,
            WorthQueryLowerRuntimeAuthorityOwner::Query,
            WRITE_AUTHORITY_CAPABILITY_LABEL,
            subject_identity,
        )?;
        let commit_evidence_identity = mutation_receipt.commit_identity.evidence_identity();
        let retained_evidence_identity =
            WorthQueryLowerRuntimeRetainedEvidenceIdentity::from_evidence_identity(
                "write-authority-commit",
                &commit_evidence_identity,
            )?;
        let eligibility =
            WorthQueryLowerRuntimeCapabilityEligibility::admitted_with_evidence_identity(
                capability_request,
                &commit_evidence_identity,
            )?;
        let route_plan = WorthQueryLowerRuntimeRoutePlan::new(
            eligibility,
            WorthQueryLowerRuntimeRouteSubjectIdentity::from_evidence_identity(
                "write-authority-route",
                &commit_evidence_identity,
            )?,
        )?;
        let boundary_execution_receipt =
            WorthQueryLowerRuntimeBoundaryExecutionReceipt::from_route_plan_with_retained_evidence_identity(
                &route_plan,
                &retained_evidence_identity,
            )?;
        let boundary_envelope =
            WorthQueryLowerRuntimeBoundaryEnvelope::from_route_plan_with_retained_evidence_identity(
                WorthQueryLowerRuntimeSeamKey::WriteAuthorityBackendExecution,
                &route_plan,
                &boundary_execution_receipt,
                &retained_evidence_identity,
            )?;
        Ok(Self {
            mutation_receipt,
            route_plan,
            boundary_execution_receipt,
            boundary_envelope,
        })
    }

    pub fn mutation_receipt(&self) -> &WorthQueryMutationReceipt {
        &self.mutation_receipt
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

    pub(crate) fn drift_from_backend_admissible_mutation(
        &self,
        mutation: &WorthQueryBackendAdmissibleMutation,
    ) -> Result<Option<String>, IdentityDerivationDenial> {
        Ok(self
            .drift_from_subject_identity(backend_admissible_mutation_subject_identity(mutation)?)?)
    }

    fn drift_from_subject_identity(
        &self,
        expected_subject: WorthQueryLowerRuntimeSubjectIdentity,
    ) -> Result<Option<String>, IdentityDerivationDenial> {
        if let Some(message) = self.route_plan.eligibility().request().drift_from_contract(
            WorthQueryLowerRuntimeSeamKey::WriteAuthorityBackendExecution,
            WorthQueryLowerRuntimeRouteKind::RoutePlanning,
            WorthQueryLowerRuntimeAuthorityOwner::Query,
            WRITE_AUTHORITY_CAPABILITY_LABEL,
            &expected_subject,
        ) {
            return Ok(Some(message));
        }
        let commit_evidence_identity = self.mutation_receipt.commit_identity.evidence_identity();
        let retained_evidence_identity =
            WorthQueryLowerRuntimeRetainedEvidenceIdentity::from_evidence_identity(
                "write-authority-commit",
                &commit_evidence_identity,
            )?;
        Ok(self
            .boundary_execution_receipt
            .drift_from_route_plan_with_retained_evidence_identity(
                &self.route_plan,
                &retained_evidence_identity,
            ))
    }
}
