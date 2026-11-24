//! Every envelope stage is enrolled in the history and scoped inside its model.
use super::super::{
    HistoryModelIdentity, HistoryOperationScope, WorthQueryDocumentHistory,
    WorthQueryDocumentHistoryDenial as Denial,
};
use super::stage_constraint::WorthQueryDocumentHistoryEnvelopeConstraint;
use crate::{
    domain_computation::primary_graph::{
        WorthQueryAdmittedApplicationOperation, WorthQueryAdvancementPhase,
        WorthQueryExpectedProductBranch, WorthQueryPrimaryGraphApplicationRuntime,
    },
    publication_boundary::WorthQueryProgramPublicationAccess,
};
use std::sync::{Mutex, MutexGuard, PoisonError};
use worth_query_declaration::facade::{
    application_history::ApplicationHistoryBinding,
    application_operation::{
        ApplicationMutationBinding, ApplicationMutationBindingDescriptor,
        ApplicationMutationScopeBinding,
    },
    application_schema::ApplicationSchema,
};
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
use worth_relational::facade::{
    branch::{
        AdmittedRelationalBranchBasis, RelationalModelOwnership, RelationalModelOwnershipDenial,
    },
    identity::EntityId,
};

type Model<S, M> =
    <<M as ApplicationMutationBinding<S>>::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope;

/// What the installed history enrolls, and the owned scopes this envelope's
/// stages were admitted on.
pub(super) struct EnvelopeStageEnrollment {
    operations: Vec<ApplicationMutationBindingDescriptor>,
    model: HistoryModelIdentity,
    /// Absent when the history owns no relation: the model is the only scope.
    ownership: Option<RelationalModelOwnership>,
    owned_scopes: Mutex<Vec<(usize, EntityId)>>,
}

impl EnvelopeStageEnrollment {
    pub(super) fn of<S: ApplicationSchema, H: ApplicationHistoryBinding<S>>(
        document: &WorthQueryDocumentHistory<'_, S, H>,
    ) -> Result<Self, Denial> {
        let runtime = document.application();
        let installed = runtime
            .installed_schema()
            .installed_history::<H>()
            .map_err(|_| Denial::InstalledBindingMismatch)?;
        if document.binding != runtime.installed_schema().binding_identity()
            || document.binding != *installed.binding_identity()
        {
            return Err(Denial::InstalledBindingMismatch);
        }
        let parts = installed.definition().parts();
        let ownership = if parts.ownership.is_empty() {
            None
        } else {
            let installed_ownership = runtime
                .inner
                .runtime
                .primary_provider_ref()
                .graph_ref()
                .layout_ref()
                .history_ownership(&parts.identity)
                .ok_or(Denial::InstalledBindingMismatch)?;
            Some(installed_ownership.clone())
        };
        Ok(Self {
            operations: installed.operation_descriptors().to_vec(),
            model: document.model.clone(),
            ownership,
            owned_scopes: Mutex::new(Vec::new()),
        })
    }

    fn owned_scopes(&self) -> MutexGuard<'_, Vec<(usize, EntityId)>> {
        self.owned_scopes
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    /// Before any stage runs: the operation is enrolled, records without
    /// workflow or external effects, and its scope is the model or an entity
    /// the model owns at the head the candidate was forked from.
    pub(super) fn require_history_envelope_stage_enrollment<M>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        enrollment: &EnvelopeStageEnrollment,
        head: &AdmittedRelationalBranchBasis,
        ordinal: usize,
        admitted: &WorthQueryAdmittedApplicationOperation<S, M::Operation, M::Input, Model<S, M>>,
    ) -> Result<(), Denial>
    where
        M: ApplicationMutationBinding<S>,
    {
        if !enrollment.operations.contains(&M::descriptor()) {
            return Err(Denial::OperationNotDeclared);
        }
        if M::WORKFLOW_CONTROL
            || M::REQUIRES_WORKFLOW_AUTHORITY
            || M::workflow_execution_requirements().is_applicable()
            || admitted
                .allowed_graph_contract()
                .external_effect()
                .is_declared()
        {
            return Err(Denial::UnsupportedOperation);
        }
        if enrollment.model.matches_operation(admitted) {
            return Ok(());
        }
        let Some(ownership) = &enrollment.ownership else {
            return Err(Denial::ForeignModel);
        };
        if !HistoryOperationScope::from_admitted(admitted)
            .belongs_to_model_runtime(&enrollment.model)
        {
            return Err(Denial::ForeignRuntime);
        }
        let subject = admitted.scope_entity_id();
        // The head is unchanged while the envelope runs, so a stage admitted
        // again keeps the answer its first admission observed.
        if enrollment.owned_scopes().contains(&(ordinal, subject)) {
            return Ok(());
        }
        let model = enrollment.model.entity();
        let owner = self
            .observe_history_scope_owner(phase, head, ownership, subject)?
            .map_err(Denial::Ownership)?;
        if owner != Some(model) {
            return Err(Denial::UnownedOperationScope { subject, model });
        }
        enrollment.owned_scopes().push((ordinal, subject));
        Ok(())
    }

    fn observe_history_scope_owner(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        basis: &AdmittedRelationalBranchBasis,
        ownership: &RelationalModelOwnership,
        subject: EntityId,
    ) -> Result<Result<Option<EntityId>, RelationalModelOwnershipDenial>, Denial> {
        let execution = phase
            .checked_execution_request_for(self.inner.runtime.product_runtime_ref())
            .map_err(Denial::Stopped)?;
        Ok(self
            .inner
            .runtime
            .primary_provider_ref()
            .graph_ref()
            .with_runtime(&execution, |native| {
                native.observe_model_owner(&execution, basis, ownership, subject)
            }))
    }
}

impl<'runtime, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryDocumentHistory<'runtime, S, H>
{
    /// After the envelope's commit and before adoption: every owned stage
    /// scope is still inside this model at the candidate's endpoint, or the
    /// step itself retired it. A refusal names the stage whose scope left.
    pub fn confirm_history_envelope_scopes(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &WorthQueryProgramPublicationAccess,
        candidate: &WorthQueryExpectedProductBranch<'runtime, S>,
        constraint: &WorthQueryDocumentHistoryEnvelopeConstraint,
    ) -> Result<(), (Option<usize>, Denial)> {
        let enrollment = constraint.enrollment();
        let scopes = enrollment.owned_scopes().clone();
        let (Some(ownership), false) = (&enrollment.ownership, scopes.is_empty()) else {
            return Ok(());
        };
        let runtime = self.application();
        if !std::ptr::eq(runtime, candidate.application())
            || candidate.branch() != constraint.candidate()
        {
            return Err((None, Denial::ForeignRuntime));
        }
        let selected = runtime
            .on_branch(candidate.branch())
            .select(phase)
            .map_err(|cause| (None, Denial::CandidateSelection(cause)))?;
        let endpoint = selected.product().relational_basis();
        let model = enrollment.model.entity();
        for (ordinal, subject) in scopes {
            match runtime
                .observe_history_scope_owner(phase, endpoint, ownership, subject)
                .map_err(|denial| (Some(ordinal), denial))?
            {
                Ok(Some(owner)) if owner == model => {}
                // The head owned this scope, so the step itself retired it.
                Err(RelationalModelOwnershipDenial::EntityUnavailable(_)) => {}
                Ok(_) => {
                    return Err((
                        Some(ordinal),
                        Denial::UnownedOperationScope { subject, model },
                    ))
                }
                Err(cause) => return Err((Some(ordinal), Denial::Ownership(cause))),
            }
        }
        Ok(())
    }
}
