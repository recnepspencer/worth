//! Proposed truth uses the exact D identity; all authority remains at C.
use super::*;
use crate::domain_computation::primary_graph::invariant_projection::read_basis::OwnOutputOverlay;
use crate::domain_computation::primary_graph::{
    WorthQueryEntityResolutionDenialKind as Resolution,
    WorthQueryEntityResolutionRefusalKind as Refusal,
};
use worth_relational::facade::runtime::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};
pub struct AppliedSourcePublicationLoan {
    pub source:
        crate::domain_computation::primary_graph::application_contribution::QualifiedOutputSource,
    pub selected: worth_relational::facade::runtime::PositionedRelationalSnapshot,
}
impl<S: ApplicationSchema> SourceQueryReaderAccess<'_, S> {
    pub fn with_applied_source_publication(
        mut self,
        source: crate::domain_computation::primary_graph::application_contribution::QualifiedOutputSource,
        selected: worth_relational::facade::runtime::PositionedRelationalSnapshot,
    ) -> Self {
        self.applied_source_publication = Some(AppliedSourcePublicationLoan { source, selected });
        self
    }
    /// Admit `intent` at the applied candidate. The scope key resolves on the
    /// candidate (over a successor round's own-output view `own` when it has
    /// one); the committed basis issues the scope only while it names that
    /// same live record, so a scope the envelope renamed, retired or replaced
    /// is never read as it was at C.
    pub fn with_applied_plan<B, R, E>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        intent: B::Input,
        before: &RelationalPinnedRead,
        candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        own: Option<&OwnOutputOverlay>,
        native: &mut ApplicationQueryNativeAccess<'_>,
        consume: impl for<'plan> FnOnce(
            SourceReadPlan<'plan, S, B>,
            &mut ApplicationQueryNativeAccess<'_>,
        ) -> Result<R, E>,
    ) -> Result<R, E>
    where
        B: ApplicationQueryBinding<S>,
        B::Input: ApplicationQueryIntent<S, Binding = B>,
        B::ScopeBinding: ApplicationQueryScopeResolution<S, B::PrincipalIdentity>,
        SourceValue<S, B>: WorthQueryApplicationProjection<S, B::Query>,
        E: From<SourceQueryObservationStop>,
    {
        let result = self.with_applied_plan_in_control::<B, R, E>(
            execution, intent, before, candidate, own, native, consume,
        );
        self.confirm_current_control(execution, native, result)
            .map_err(|cause| E::from(self.record_control_refusal(cause)))?
    }
    #[allow(clippy::too_many_arguments)]
    fn with_applied_plan_in_control<B, R, E>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        intent: B::Input,
        before: &RelationalPinnedRead,
        candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        own: Option<&OwnOutputOverlay>,
        native: &mut ApplicationQueryNativeAccess<'_>,
        consume: impl for<'plan> FnOnce(
            SourceReadPlan<'plan, S, B>,
            &mut ApplicationQueryNativeAccess<'_>,
        ) -> Result<R, E>,
    ) -> Result<R, E>
    where
        B: ApplicationQueryBinding<S>,
        B::Input: ApplicationQueryIntent<S, Binding = B>,
        B::ScopeBinding: ApplicationQueryScopeResolution<S, B::PrincipalIdentity>,
        SourceValue<S, B>: WorthQueryApplicationProjection<S, B::Query>,
        E: From<SourceQueryObservationStop>,
    {
        use SourceQueryObservationStop as Stop;
        self.confirm_current_control(execution, native, ())
            .map_err(|cause| self.record_control_refusal(cause))?;
        let selected = self.shared.selected();
        if !std::ptr::eq(self.application, selected.application())
            || candidate
                .candidate_identity()
                .proposal()
                .branch_observation()
                != selected.product().relational_basis_descriptor().reference()
        {
            return Err(Stop::ForeignClosure.into());
        }
        let installed = self
            .application
            .installed_schema()
            .installed_query_binding::<B>()
            .map_err(Stop::Installation)?;
        let principal = selected
            .resolve_authenticated_principal_issued_with_native_access(
                execution,
                installed.principal_binding(),
                self.principal,
                self.request,
                WorthQueryPrincipalResolutionMode::Ordinary,
                None,
                native,
            )
            .map_err(Stop::Principal)?;
        let parameters = intent.parameters();
        let (field, value) = intent
            .into_scope()
            .into_field_parts(principal.principal().principal_identity());
        let committed = match selected.resolve_query_scope_selector_issued_with_native_access(
            execution,
            field,
            &value,
            self.request,
            native,
        ) {
            Ok(scope) => Ok(scope),
            Err(cause)
                if matches!(
                    cause.kind(),
                    Resolution::Refused(Refusal::UnknownEntity | Refusal::AmbiguousEntity)
                ) =>
            {
                Err(cause)
            }
            Err(cause) => return Err(Stop::Scope(cause).into()),
        };
        let graph = self
            .application
            .runtime
            .primary_graph()
            .ok_or(Stop::ForeignClosure)?;
        // The candidate is the authority for which record the key names.
        let scope = graph
            .retain_entity_resolution_context()
            .resolve_applied_field::<S, _>(execution, before, candidate, own, field, value);
        let absent = match (committed, &scope) {
            (Ok(issued), Ok(scope)) if issued.scope().entity_id() == scope.entity_id() => {
                return self.with_resolved_plan::<B, R, E>(
                    execution,
                    &installed,
                    parameters,
                    &principal,
                    &issued,
                    native,
                    consume,
                )
            }
            (Ok(_), _) => None,
            (Err(absent), _) => Some(absent),
        };
        if let Some(lapse) = principal.request_lapse() {
            return Err(
                Stop::Admission(WorthQueryApplicationQueryAdmissionDenial::new(
                    WorthQueryApplicationQueryAdmissionDenialKind::Authorization(
                        lapse.denial_kind(),
                    ),
                    B::IDENTITY,
                ))
                .into(),
            );
        }
        // A scope absent at C and at the candidate keeps C's refusal. Any
        // other candidate refusal stands as the candidate resolved it: a
        // retired scope is unknown, two new scopes sharing the key ambiguous.
        let scope = scope.map_err(|cause| {
            Stop::Scope(match &absent {
                Some(absent) if cause.kind() == absent.kind() => absent.clone(),
                _ => cause,
            })
        })?;
        let controls = self.applied_source_controls(&installed)?;
        let access = WorthQueryApplicationQueryAccessContext::new(principal.principal(), &scope);
        let plan = self
            .application
            .admit_fresh_applied_source_query(
                execution,
                installed.query(),
                access,
                parameters,
                controls,
                selected,
                &principal,
                before,
                candidate,
                self.request,
                self.read_control(),
                native,
            )
            .map_err(|cause| {
                use crate::domain_computation::authorization::{
                    WorthQueryOperationAuthorizationDenialKind as Authority,
                    WorthQueryOperationAuthorizationRefusalKind as Refused,
                };
                match absent {
                    Some(absent)
                        if matches!(
                            cause.kind(),
                            WorthQueryApplicationQueryAdmissionDenialKind::Authorization(
                                Authority::Refused(
                                    Refused::PermissionDenied
                                        | Refused::ScopeMismatch
                                        | Refused::RelationalObservationRejected
                                )
                            )
                        ) =>
                    {
                        Stop::Scope(absent)
                    }
                    _ => Stop::Admission(cause),
                }
            })?;
        consume(plan, native)
    }

    /// The one-shot controls of an applied Source admission, built exactly as
    /// the committed admission builds them (`with_resolved_plan`).
    pub(super) fn applied_source_controls<B: ApplicationQueryBinding<S>>(
        &self,
        installed: &worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding<
            S,
            B,
        >,
    ) -> Result<WorthQueryApplicationQueryControls<'_, S>, SourceQueryObservationStop> {
        use SourceQueryObservationStop as Stop;
        let selected = self.shared.selected();
        let (product, basis) = self
            .application
            .retain_selected_query_basis_admitted(self.shared)
            .map_err(Stop::Basis)?;
        let limits = self
            .application
            .resolve_application_query_limits(installed.limits());
        let query_maximum =
            NonZeroUsize::new(installed.query().read_graph().maximum_result_count())
                .unwrap_or(NonZeroUsize::MAX);
        Ok(WorthQueryApplicationQueryControls::selected_read_one_shot(
            product,
            basis,
            limits.maximum_results().min(query_maximum),
            self.request,
        )
        .map_err(|cause| Stop::Read(WorthQueryAdmittedOneShotStop::Admission(cause)))?
        .with_selection_origin(selected.selection_origin())
        .with_original_settled_producer(selected.original_settled_producer().cloned()))
    }
}
