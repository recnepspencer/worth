//! SameAdmittedScope resolves the Source's own selector and fresh READ audience.
use super::*;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryParameterSet, ApplicationQueryScopeResolutionMode,
};
use worth_query_execution_attempt::facade::ports::AuthenticatedPrincipalInternalPort as _;

impl<Schema: ApplicationSchema> SourceQueryReaderAccess<'_, Schema> {
    pub fn with_same_scope_plan<B, R, E>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        selector: &WorthQueryObservedScopeSelector,
        parent_principal: worth_relational::facade::identity::EntityId,
        allocation: &Arc<
            crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation,
        >,
        native: &mut ApplicationQueryNativeAccess<'_>,
        consume: impl for<'plan> FnOnce(
            SourceReadPlan<'plan, Schema, B>,
            &mut ApplicationQueryNativeAccess<'_>,
        ) -> Result<R, E>,
    ) -> Result<R, E>
    where
        B: ApplicationQueryBinding<Schema>,
        E: From<SourceQueryObservationStop>
            + From<worth_relational::facade::transactions::RelationalExecutionDenialCause>,
    {
        self.with_same_scope_plan_on::<B, R, E>(
            execution,
            selector,
            parent_principal,
            allocation,
            None,
            native,
            consume,
        )
    }

    pub fn with_same_scope_plan_on<B, R, E>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        selector: &WorthQueryObservedScopeSelector,
        parent_principal: worth_relational::facade::identity::EntityId,
        allocation: &Arc<
            crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation,
        >,
        applied: Option<(
            &worth_relational::facade::runtime::RelationalPinnedRead,
            &worth_relational::facade::runtime::RelationalPinnedRead<
                worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
            >,
        )>,
        native: &mut ApplicationQueryNativeAccess<'_>,
        consume: impl for<'plan> FnOnce(
            SourceReadPlan<'plan, Schema, B>,
            &mut ApplicationQueryNativeAccess<'_>,
        ) -> Result<R, E>,
    ) -> Result<R, E>
    where
        B: ApplicationQueryBinding<Schema>,
        E: From<SourceQueryObservationStop>
            + From<worth_relational::facade::transactions::RelationalExecutionDenialCause>,
    {
        use SourceQueryObservationStop as Stop;
        let installed = self
            .application
            .installed_schema()
            .installed_query_binding::<B>()
            .map_err(Stop::Installation)?;
        let selected = self.shared.selected();
        if !std::ptr::eq(self.application, selected.application()) {
            return Err(Stop::ForeignClosure.into());
        }
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
        if principal.principal().principal_entity_id() != parent_principal {
            return Err(Stop::ForeignClosure.into());
        }
        // PrincipalIdentity scopes still require the actual identity value;
        // a retained selected scope alone cannot change that resolution policy.
        if B::ScopeBinding::RESOLUTION_MODE
            == ApplicationQueryScopeResolutionMode::PrincipalIdentity
        {
            let storage = super::super::one_shot::outcome::material::MaterialCompletion::new(
                execution, allocation,
            );
            let same = principal.principal().freshness().matches_principal_identity(
                selector.value(), &mut |units| storage.work(units),
                || worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkCounterOverflow,
            )?;
            if !same {
                return Err(Stop::ForeignClosure.into());
            }
        }
        let field = B::scope_field();
        let column = selected.record_selection(field).map_err(Stop::Scope)?;
        let locus = column.recipe().locus();
        let _temporary = super::super::managed_index::source_acquisition::temporary::scope(
            execution,
            locus.entity(),
            locus.aspect(),
            locus.field(),
            selector,
        )?;
        if let Some((before, candidate)) = applied {
            if self.history.is_some()
                || candidate
                    .candidate_identity()
                    .proposal()
                    .branch_observation()
                    != selected.product().relational_basis_descriptor().reference()
            {
                return Err(Stop::ForeignClosure.into());
            }
            let scope = selected
                .resolve_retained_applied_query_scope(
                    execution,
                    field,
                    selector,
                    before,
                    candidate,
                    self.request,
                )
                .map_err(Stop::Scope)?;
            if scope.entity_id() != selector.subject() {
                return Err(Stop::ForeignClosure.into());
            }
            let (product, basis) = self
                .application
                .retain_selected_query_basis_admitted(self.shared)
                .map_err(Stop::Basis)?;
            let limits = self
                .application
                .resolve_application_query_limits(installed.limits());
            let controls = WorthQueryApplicationQueryControls::selected_read_one_shot(
                product,
                basis,
                limits.maximum_results(),
                self.request,
            )
            .map_err(|cause| Stop::Read(WorthQueryAdmittedOneShotStop::Admission(cause)))?
            .with_selection_origin(selected.selection_origin())
            .with_original_settled_producer(selected.original_settled_producer().cloned());
            let access =
                WorthQueryApplicationQueryAccessContext::new(principal.principal(), &scope);
            let plan = self
                .application
                .admit_fresh_applied_source_query(
                    execution,
                    installed.query(),
                    access,
                    ApplicationQueryParameterSet::new(),
                    controls,
                    selected,
                    &principal,
                    before,
                    candidate,
                    self.request,
                    native,
                )
                .map_err(Stop::Admission)?;
            return consume(plan, native);
        }
        let scope = selected
            .resolve_retained_query_scope_issued_with_native_access(
                execution,
                field,
                selector,
                self.request,
                native,
            )
            .map_err(Stop::Scope)?;
        if scope.scope().entity_id() != selector.subject() {
            return Err(Stop::ForeignClosure.into());
        }
        self.with_resolved_plan::<B, _, _>(
            execution,
            &installed,
            super::permission::SourceQueryParameters::Live(ApplicationQueryParameterSet::new()),
            &principal,
            &scope,
            super::permission::SourceScopePosture::Current,
            native,
            consume,
        )
    }
}
