//! Exact D scope presence extends the same C conventional authorization owner.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEntityIdentity, WorthQuerySelectedProductOperation,
};
use worth_relational::facade::runtime::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};
pub(super) enum SelectedMutationScope<'a, 'selected, 'runtime, S, E> {
    Issued(&'a WorthQueryIssuedSelectedScope<'selected, 'runtime, S, E>),
    Applied {
        identity: &'a WorthQueryApplicationEntityIdentity<S, E>,
        before: &'a RelationalPinnedRead,
        candidate: &'a RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    },
}
impl<S: ApplicationSchema, E> SelectedMutationScope<'_, '_, '_, S, E> {
    pub(super) fn scope(&self) -> &WorthQueryApplicationEntityIdentity<S, E> {
        match self {
            Self::Issued(scope) => scope.scope(),
            Self::Applied { identity, .. } => identity,
        }
    }
    pub(super) fn admits_selection(
        &self,
        selected: &WorthQuerySelectedProductOperation<'_, S>,
        request: &WorthQueryRequestScope,
    ) -> bool {
        match self {
            Self::Issued(scope) => {
                std::ptr::eq(scope.selected(), selected) && scope.request().same_request(request)
            }
            Self::Applied { candidate, .. } => {
                candidate
                    .candidate_identity()
                    .proposal()
                    .branch_observation()
                    == selected.product().relational_basis().reference()
            }
        }
    }
    /// Runs after C's principal and installed operation admission, never before.
    pub(super) fn require_applied_presence(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        subject: &str,
    ) -> Result<(), SelectedConventionalAdmissionStop> {
        let Self::Applied {
            identity,
            before,
            candidate,
        } = self
        else {
            return Ok(());
        };
        runtime.runtime.primary_graph().ok_or(SelectedConventionalAdmissionStop::ForeignIssuedSource)?.retain_entity_resolution_context().validate_applied_identity(execution, before, candidate, identity)
            .map_err(|cause| {
                use crate::domain_computation::authorization::{WorthQueryOperationAuthorizationDenial as Denial, WorthQueryOperationAuthorizationDenialKind as Kind, WorthQueryOperationAuthorizationRefusalKind as Refusal};
                let denial = match cause.kind() {
                    crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenialKind::Stopped(cause) => crate::domain_computation::authorization::denial::relational_read_denial(cause, subject),
                    _ => Denial::new(Kind::Refused(Refusal::StaleScope), subject),
                };
                SelectedConventionalAdmissionStop::Authorization(denial)
            })
    }
}
pub(in crate::domain_computation) fn authorize_applied_control_mutation_on_selected<S, B, E>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    current_product: &SharedSelectedProductOperation<'_, S>,
    current: &WorthQueryCurrentRetainedMutationBinding<'_, S, B>,
    prepared: &WorthQueryPreparedApplicationOperationGraphWork,
    principal: &WorthQueryIssuedSelectedPrincipal<'_, '_, S, B::Principal, B::PrincipalIdentity>,
    scope: &WorthQueryApplicationEntityIdentity<S, E>,
    before: &RelationalPinnedRead,
    candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    request: &WorthQueryRequestScope,
) -> Result<
    WorthQueryAdmittedApplicationOperation<S, B::Operation, B::Input, E>,
    SelectedConventionalAdmissionStop,
>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
    E: ApplicationEntityMarkerIdentity<S>,
    B::ScopeBinding: ApplicationMutationScopeBinding<S, Scope = E>,
{
    authorize_mutation_products(
        execution,
        runtime,
        SelectedMutationProducts {
            mutation: current_product,
            control: None,
            history: None,
        },
        current,
        prepared,
        principal,
        SelectedMutationScope::Applied {
            identity: scope,
            before,
            candidate,
        },
        request,
    )
}
