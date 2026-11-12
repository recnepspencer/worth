//! C issues authority; only the exact unpublished candidate proves source presence.
use super::*;
use worth_relational::facade::runtime::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};

pub(super) struct AppliedSourceTruth<'a> {
    before: &'a RelationalPinnedRead,
    candidate: &'a RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
}
impl<'a, S: ApplicationSchema, P, I, E> SelectedIssuedAccessRoot<'a, S, P, I, E> {
    pub(in crate::domain_computation::primary_graph::application_query) fn from_applied_source(
        current: &'a WorthQuerySelectedProductOperation<'_, S>,
        access: &WorthQueryApplicationQueryAccessContext<'a, S, P, I, E>,
        principal: &'a WorthQueryIssuedSelectedPrincipal<'_, '_, S, P, I>,
        before: &'a RelationalPinnedRead,
        candidate: &'a RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        subject: &str,
    ) -> Result<Self, WorthQueryApplicationQueryAdmissionDenial> {
        if !std::ptr::eq(principal.selected(), current)
            || !std::ptr::eq(principal.principal(), access.principal())
            || !principal.request().same_request(request)
            || candidate
                .candidate_identity()
                .proposal()
                .branch_observation()
                != current.product().relational_basis().reference()
        {
            return Err(super::super::denial(
                WorthQueryApplicationQueryAdmissionDenialKind::StaleBasis,
                subject,
            ));
        }
        Ok(Self {
            principal: principal.principal(),
            scope: access.scope(),
            root: principal.issued_root(),
            snapshot: principal.issued_snapshot(),
            product: current.product().observation(),
            request: principal.request(),
            truth: None,
            historical_source: None,
            applied_source: Some(AppliedSourceTruth { before, candidate }),
        })
    }
    pub(super) fn require_applied_source_after_authorization(
        &self,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<S>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
        let truth = self
            .applied_source
            .as_ref()
            .expect("applied source posture");
        let graph = runtime.runtime.primary_graph().ok_or_else(|| {
            super::super::denial(
                WorthQueryApplicationQueryAdmissionDenialKind::RuntimeSupportUnavailable,
                subject,
            )
        })?;
        graph
            .retain_entity_resolution_context()
            .validate_applied_identity(execution, truth.before, truth.candidate, self.scope)
            .map_err(|cause| {
                super::super::denial(
                    super::super::super::access_observation::freshness_denial::entity(cause.kind()),
                    subject,
                )
            })
    }
}
