use super::*;

/// An installed scope resolved on the selected Product's exact native root.
/// The selected operation is borrowed until the scoped Query read completes.
pub(in crate::domain_computation) struct WorthQueryIssuedSelectedScope<
    'selected,
    'runtime,
    Schema,
    Scope,
> {
    scope: WorthQueryApplicationEntityIdentity<Schema, Scope>,
    selected: &'selected WorthQuerySelectedProductOperation<'runtime, Schema>,
    request: WorthQueryRequestScope,
}

impl<Schema, Scope> WorthQueryIssuedSelectedScope<'_, '_, Schema, Scope> {
    pub(in crate::domain_computation) fn scope(
        &self,
    ) -> &WorthQueryApplicationEntityIdentity<Schema, Scope> {
        &self.scope
    }

    pub(in crate::domain_computation) fn issued_root(
        &self,
    ) -> &worth_relational::facade::branch::AdmittedRelationalBranchBasis {
        self.selected.product().relational_basis()
    }

    pub(in crate::domain_computation) fn issued_snapshot(
        &self,
    ) -> &worth_relational::facade::snapshots::SnapshotHandle {
        self.selected.application_basis().snapshot_handle()
    }

    pub(in crate::domain_computation) fn request(&self) -> &WorthQueryRequestScope {
        &self.request
    }

    pub(in crate::domain_computation) fn selected(
        &self,
    ) -> &WorthQuerySelectedProductOperation<'_, Schema> {
        self.selected
    }
}

impl<'runtime, Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'runtime, Schema> {
    /// Mint the same exact-root custody for a conventional mutation's typed
    /// equality scope. The selected native resolver remains the acceptance
    /// owner; this wrapper only retains its request and selected root.
    pub(in crate::domain_computation) fn resolve_mutation_scope_issued<
        'selected,
        Aspect,
        Entity,
        Field,
        Value,
        Write,
        Unit,
    >(
        &'selected self,
        field: ApplicationFieldRef<
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            EqualityPredicate,
            Unit,
        >,
        value: Value,
        request: &WorthQueryRequestScope,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<
        WorthQueryIssuedSelectedScope<'selected, 'runtime, Schema, Entity>,
        WorthQueryEntityResolutionDenial,
    >
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        let scope = self.resolve_entity_admitted(field, value, request, admission)?;
        admission
            .charge_external_work(1)
            .map_err(|stop| selected_index_admission_denial(stop, scope.entity_name()))?;
        Ok(WorthQueryIssuedSelectedScope {
            scope,
            selected: self,
            request: request.clone(),
        })
    }

    pub(in crate::domain_computation::primary_graph) fn resolve_retained_query_scope_issued<
        'selected,
        Aspect,
        Entity,
        Field,
        Value,
        Write,
        Unit,
    >(
        &'selected self,
        field: ApplicationFieldRef<
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            EqualityPredicate,
            Unit,
        >,
        selector: &WorthQueryObservedScopeSelector,
        request: &WorthQueryRequestScope,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<
        WorthQueryIssuedSelectedScope<'selected, 'runtime, Schema, Entity>,
        WorthQueryEntityResolutionDenial,
    >
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        let scope = self.resolve_retained_query_scope(field, selector, request, admission)?;
        admission
            .charge_external_work(1)
            .map_err(|stop| selected_index_admission_denial(stop, scope.entity_name()))?;
        Ok(WorthQueryIssuedSelectedScope {
            scope,
            selected: self,
            request: request.clone(),
        })
    }
}
