use std::sync::Arc;

use super::call::WorthQueryGraphProviderCallSpec;
use super::call_identity::WorthQueryGraphCallAuthorityIdentity;
use super::WorthQueryGraphProviderCall;
use crate::domain_computation::domain_evidence_binding::WorthQueryCandidateOccurrenceBinding;
use crate::domain_computation::managed_run::{
    WorthQueryCompletedDirectEvidenceOwner, WorthQueryCompletedWorkflowEvidenceOwner,
};
use crate::domain_computation::provider_session::{
    WorthQueryClosedExecutionAttemptIdentity, WorthQueryExecutionProviderSessionIdentity,
};
use crate::domain_computation::WorthQueryConvergenceDomainEvidenceBindingDenial;
use crate::execution_digest::hash_parts;

#[derive(Clone, Eq, PartialEq)]
pub(in crate::domain_computation) struct WorthQueryBoundGraphExecutionAssociation {
    call_authority: WorthQueryGraphCallAuthorityIdentity,
    call_identity: Arc<str>,
    provider_session: WorthQueryExecutionProviderSessionIdentity,
    attempt: WorthQueryClosedExecutionAttemptIdentity,
    call: WorthQueryGraphProviderCallSpec,
}

pub(in crate::domain_computation) struct WorthQueryCompletedDomainEvidenceDerivation {
    contract: Arc<worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>,
    execution: WorthQueryBoundGraphExecutionAssociation,
    candidate_selection_key: Arc<str>,
    candidate_occurrence: WorthQueryCandidateOccurrenceBinding,
}

impl WorthQueryBoundGraphExecutionAssociation {
    pub(super) fn capture(call: &WorthQueryGraphProviderCall) -> Self {
        Self {
            call_authority: call.authority_identity(),
            call_identity: Arc::from(call.call_identity()),
            provider_session: call.closed_provider_session_identity(),
            attempt: call.closed_attempt_identity(),
            call: call.spec().clone(),
        }
    }

    pub(in crate::domain_computation) fn derive_direct(
        &self,
        owner: WorthQueryCompletedDirectEvidenceOwner<'_>,
        candidate_selection_key: &str,
    ) -> Result<
        WorthQueryCompletedDomainEvidenceDerivation,
        WorthQueryConvergenceDomainEvidenceBindingDenial,
    > {
        let authority = owner.authority();
        if authority.is_workflow_operation() {
            return Err(WorthQueryConvergenceDomainEvidenceBindingDenial::DirectOperationRequired);
        }
        let contract = authority
            .operation_evidence_contract()
            .cloned()
            .ok_or(WorthQueryConvergenceDomainEvidenceBindingDenial::ArtifactContractRequired)?;
        self.derive(
            authority,
            owner.session(),
            owner.logical_run_identity(),
            None,
            owner.execution_snapshot(),
            candidate_selection_key,
            contract,
        )
    }

    pub(in crate::domain_computation) fn derive_workflow(
        &self,
        owner: WorthQueryCompletedWorkflowEvidenceOwner<'_>,
        candidate_selection_key: &str,
    ) -> Result<
        WorthQueryCompletedDomainEvidenceDerivation,
        WorthQueryConvergenceDomainEvidenceBindingDenial,
    > {
        let authority = owner.authority();
        if !authority.is_workflow_operation() {
            return Err(
                WorthQueryConvergenceDomainEvidenceBindingDenial::WorkflowOperationRequired,
            );
        }
        let contract = authority
            .workflow_stage_artifact_contracts(owner.stage_identity())
            .ok_or(WorthQueryConvergenceDomainEvidenceBindingDenial::StageNotInstalled)?
            .evidence()
            .cloned()
            .ok_or(WorthQueryConvergenceDomainEvidenceBindingDenial::ArtifactContractRequired)?;
        self.derive(
            authority,
            owner.session(),
            owner.logical_run_identity(),
            Some(owner.stage_identity()),
            owner.execution_snapshot(),
            candidate_selection_key,
            contract,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn derive(
        &self,
        authority: &crate::domain_computation::WorthQueryExecutionBoundOperationAuthority,
        session: &crate::domain_computation::WorthQueryExecutionProviderSession,
        logical_run_identity: &str,
        stage_identity: Option<&str>,
        execution_snapshot: &crate::domain_computation::domain_evidence_binding::WorthQueryBoundExecutionSnapshotIdentity,
        candidate_selection_key: &str,
        contract: Arc<
            worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority,
        >,
    ) -> Result<
        WorthQueryCompletedDomainEvidenceDerivation,
        WorthQueryConvergenceDomainEvidenceBindingDenial,
    > {
        if !authority.is_current_installation_generation() {
            return Err(
                WorthQueryConvergenceDomainEvidenceBindingDenial::StaleInstallationGeneration,
            );
        }
        if logical_run_identity.trim().is_empty() {
            return Err(WorthQueryConvergenceDomainEvidenceBindingDenial::EmptyRunIdentity);
        }
        if candidate_selection_key.trim().is_empty() {
            return Err(
                WorthQueryConvergenceDomainEvidenceBindingDenial::EmptyCandidateSelectionKey,
            );
        }
        if !self.admits_execution(authority, session, stage_identity, execution_snapshot) {
            return Err(
                WorthQueryConvergenceDomainEvidenceBindingDenial::ExecutionAssociationMismatch,
            );
        }
        Ok(WorthQueryCompletedDomainEvidenceDerivation {
            contract: Arc::clone(&contract),
            execution: self.clone(),
            candidate_selection_key: Arc::from(candidate_selection_key),
            candidate_occurrence: WorthQueryCandidateOccurrenceBinding::owner_derived(
                self.occurrence_identity(&contract, logical_run_identity, candidate_selection_key),
            ),
        })
    }

    fn admits_execution(
        &self,
        authority: &crate::domain_computation::WorthQueryExecutionBoundOperationAuthority,
        session: &crate::domain_computation::WorthQueryExecutionProviderSession,
        stage_identity: Option<&str>,
        execution_snapshot: &crate::domain_computation::domain_evidence_binding::WorthQueryBoundExecutionSnapshotIdentity,
    ) -> bool {
        self.provider_session == session.closed_identity()
            && self.attempt == session.closed_attempt_identity()
            && self.call.scope.operation_identity.as_ref() == authority.operation_identity()
            && self.call.scope.binding_identity.as_ref() == authority.binding_identity()
            && self.call.read_binding.basis_identity.as_ref() == authority.basis_identity()
            && self.call.scope.stage_identity.as_deref() == stage_identity
            && self.call.read_binding.snapshot_identity == *execution_snapshot
    }

    fn occurrence_identity(
        &self,
        contract: &worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority,
        logical_run_identity: &str,
        candidate_selection_key: &str,
    ) -> Arc<str> {
        Arc::from(hash_parts(&[
            "worth_query_candidate_occurrence_v1".into(),
            format!("call-authority:{}", self.call_authority.as_u64()),
            format!("call:{}", self.call_identity),
            format!("session:{}", self.provider_session.as_str()),
            format!("attempt:{}", self.attempt.as_str()),
            format!(
                "snapshot:{}",
                self.call.read_binding.snapshot_identity.as_str()
            ),
            format!("operation:{}", self.call.scope.operation_identity),
            format!("binding:{}", self.call.scope.binding_identity),
            format!("basis:{}", self.call.read_binding.basis_identity),
            format!(
                "stage:{}",
                self.call
                    .scope
                    .stage_identity
                    .as_deref()
                    .unwrap_or("direct")
            ),
            format!("logical-run:{logical_run_identity}"),
            format!(
                "artifact-admission:{}",
                contract.admission_identity().render_support_hex()
            ),
            format!(
                "artifact-contract:{}",
                contract.contract().identity().as_str()
            ),
            format!("candidate-selection:{candidate_selection_key}"),
        ]))
    }
}

impl WorthQueryCompletedDomainEvidenceDerivation {
    pub(in crate::domain_computation) fn contract(
        &self,
    ) -> &worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority {
        &self.contract
    }

    pub(in crate::domain_computation) fn execution(
        &self,
    ) -> &WorthQueryBoundGraphExecutionAssociation {
        &self.execution
    }

    pub(in crate::domain_computation) fn candidate_selection_key(&self) -> &str {
        &self.candidate_selection_key
    }

    pub(in crate::domain_computation) fn candidate_occurrence_identity(&self) -> &str {
        self.candidate_occurrence.identity()
    }
}

impl std::fmt::Debug for WorthQueryBoundGraphExecutionAssociation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryBoundGraphExecutionAssociation")
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{attempt, call};
    use super::*;
    use crate::domain_computation::domain_evidence_binding::WorthQueryBoundExecutionSnapshotIdentity;

    #[test]
    fn exact_execution_association_denies_each_independent_owner_axis_swap() {
        let exact_attempt = attempt();
        let foreign_attempt = attempt();
        let exact =
            WorthQueryBoundGraphExecutionAssociation::capture(&call(&exact_attempt, "same-scope"));
        let foreign = WorthQueryBoundGraphExecutionAssociation::capture(&call(
            &foreign_attempt,
            "same-scope",
        ));
        let expected_snapshot =
            WorthQueryBoundExecutionSnapshotIdentity::capture(Arc::from("snapshot"));
        let authority = exact_attempt.attempt.binding_authority();
        let session = exact_attempt.attempt.provider_session_for_test();

        assert!(exact.admits_execution(authority, session, None, &expected_snapshot));

        let mut session_swap = exact.clone();
        session_swap.provider_session = foreign.provider_session.clone();
        assert!(!session_swap.admits_execution(authority, session, None, &expected_snapshot));

        let mut attempt_swap = exact.clone();
        attempt_swap.attempt = foreign.attempt.clone();
        assert!(!attempt_swap.admits_execution(authority, session, None, &expected_snapshot));

        assert!(!exact.admits_execution(
            authority,
            session,
            Some("foreign-stage"),
            &expected_snapshot,
        ));

        let foreign_snapshot =
            WorthQueryBoundExecutionSnapshotIdentity::capture(Arc::from("foreign-snapshot"));
        assert!(!exact.admits_execution(authority, session, None, &foreign_snapshot));
    }
}
