//! SameAdmittedScope resolves the Source's own selector and fresh READ audience.
use super::*;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryParameterSet, ApplicationQueryScopeResolutionMode,
};

impl<Schema: ApplicationSchema> SourceQueryReaderAccess<'_, Schema> {
    pub(in crate::domain_computation::primary_graph::application_query) fn with_same_scope_plan<
        B,
        R,
        E,
    >(
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
        let _temporary = super::super::managed_index::source_acquisition::temporary::scope(
            execution,
            field.entity(),
            field.aspect(),
            field.field(),
            selector,
        )?;
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
            ApplicationQueryParameterSet::new(),
            &principal,
            &scope,
            native,
            consume,
        )
    }
}
