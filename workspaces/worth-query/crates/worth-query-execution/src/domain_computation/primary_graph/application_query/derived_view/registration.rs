use std::sync::Arc;

use worth_query_declaration::facade::{
    application_query::ApplicationDerivedViewDefinition, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationQuery, WorthQueryInstalledApplicationQueryAuthorization,
};

use super::dependency::source_dependencies;
use super::registry::RegisteredDerivedView;
use super::retention::{ManagedDerivedViewState, WorthQueryManagedDerivedViewDenial as Denial};
use super::{
    WorthQueryManagedDerivedValue, WorthQueryManagedDerivedView,
    WorthQueryManagedDerivedViewSnapshot,
};
use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::primary_graph::application_query::{
    WorthQueryApplicationOneShotResult, WorthQueryObservedSource,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

mod collection;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn open_managed_derived_view<Query, Parameters, QueryResult, Scope, Value>(
        &self,
        definition: &ApplicationDerivedViewDefinition<
            Schema,
            Query,
            Parameters,
            QueryResult,
            Scope,
        >,
        query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
        product: &WorthQueryProductBranchLease,
    ) -> Result<WorthQueryManagedDerivedView<Query, Value>, Denial>
    where
        Value: WorthQueryManagedDerivedValue,
    {
        self.open_managed_derived_view_with_entry_query(definition, query, None, None, product)
    }

    fn open_managed_derived_view_with_entry_query<Query, Parameters, QueryResult, Scope, Value>(
        &self,
        definition: &ApplicationDerivedViewDefinition<
            Schema,
            Query,
            Parameters,
            QueryResult,
            Scope,
        >,
        query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
        entry_query: Option<
            worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity,
        >,
        secondary_entry_query: Option<
            worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity,
        >,
        product: &WorthQueryProductBranchLease,
    ) -> Result<WorthQueryManagedDerivedView<Query, Value>, Denial>
    where
        Value: WorthQueryManagedDerivedValue,
    {
        let limits = definition.limits();
        if definition.name().is_empty()
            || limits.maximum_entries() == 0
            || limits.maximum_retained_bytes() == 0
        {
            return Err(Denial::InvalidLimits);
        }
        self.installed_schema
            .validate_installed_query(query)
            .map_err(|_| Denial::ForeignInstallation)?;
        if !matches!(
            query.authorization(),
            WorthQueryInstalledApplicationQueryAuthorization::Public
        ) {
            return Err(Denial::AuthorizationRequired);
        }
        if definition.query().name() != query.name() {
            return Err(Denial::ForeignQuery);
        }
        self.admit_current_view_product(product)?;
        let graph = self
            .runtime
            .primary_graph()
            .ok_or(Denial::ForeignApplication)?;
        let observation = product.observation();
        let state = Arc::new(ManagedDerivedViewState::new(
            self.runtime.authority_identity().as_u64(),
            self.installed_schema.binding_identity().clone(),
            query.identity().clone(),
            entry_query,
            secondary_entry_query,
            product.relational_basis().identity().branch_id().clone(),
            observation.branch_identity().clone(),
            observation.lifecycle_incarnation(),
            observation.selected_commit().clone(),
            limits,
        ));
        let registry = graph.managed_derived_views();
        let erased: Arc<dyn RegisteredDerivedView> = state.clone();
        let id = registry
            .register(&erased)
            .ok_or(Denial::ViewCapacityExceeded)?;
        Ok(WorthQueryManagedDerivedView {
            state,
            id,
            registry: Arc::downgrade(&registry),
            _query: std::marker::PhantomData,
        })
    }

    pub fn observe_managed_derived_view<Query, Value>(
        &self,
        view: &WorthQueryManagedDerivedView<Query, Value>,
    ) -> Result<WorthQueryManagedDerivedViewSnapshot<Value>, Denial>
    where
        Value: WorthQueryManagedDerivedValue,
    {
        if view.state.runtime_authority != self.runtime.authority_identity().as_u64() {
            return Err(Denial::ForeignApplication);
        }
        if view.state.binding != self.installed_schema.binding_identity() {
            return Err(Denial::ForeignInstallation);
        }
        let current = self
            .product_runtime
            .with_product_observation(&view.state.product_branch, |observation| {
                Ok((
                    observation.lifecycle_incarnation(),
                    observation.selected_commit().clone(),
                ))
            })
            .map_err(|_| Denial::ForeignBranch)?;
        if current.0 != view.state.incarnation {
            view.discard();
            return Err(Denial::ForeignBranch);
        }
        if view.state.current_commit().as_ref() != Some(&current.1) {
            view.state.rebase_cold(current.1);
            return Err(Denial::ColdReconstructionRequired);
        }
        Ok(WorthQueryManagedDerivedViewSnapshot {
            state: Arc::clone(&view.state),
            commit: current.1,
            _entry: std::marker::PhantomData,
        })
    }

    pub fn reconstruct_managed_derived_view<Query, QueryResult, Value>(
        &self,
        view: &WorthQueryManagedDerivedView<Query, Value>,
        product: &WorthQueryProductBranchLease,
        result: &WorthQueryApplicationOneShotResult<Query, QueryResult>,
        mut project: impl FnMut(&QueryResult) -> Value,
    ) -> Result<(), Denial>
    where
        Value: WorthQueryManagedDerivedValue,
    {
        self.admit_view_product(view, product)?;
        if result.rows().len() != result.observed_sources().len() {
            return Err(Denial::IncompleteDependencies);
        }
        if result.rows().len() > view.state.limits.maximum_entries() {
            return Err(Denial::EntryCapacityExceeded);
        }
        let membership = self.checked_view_dependencies(
            view,
            product,
            result.result_set_observation().source(),
        )?;
        let mut reconstructed = Vec::with_capacity(result.rows().len());
        for (source, row) in result.observed_sources().iter().zip(result.rows()) {
            let dependencies = self.checked_view_dependencies(view, product, source)?;
            reconstructed.push((
                source.managed_derived_view_key(),
                project(row),
                dependencies,
            ));
        }
        view.state
            .reconstruct(reconstructed, membership, product.selected_commit())
    }

    fn checked_view_dependencies<Query, Value>(
        &self,
        view: &WorthQueryManagedDerivedView<Query, Value>,
        product: &WorthQueryProductBranchLease,
        source: &WorthQueryObservedSource<Query>,
    ) -> Result<std::collections::BTreeSet<super::dependency::ViewDependency>, Denial>
    where
        Value: WorthQueryManagedDerivedValue,
    {
        self.checked_view_dependencies_for_query(view, product, source, &view.state.query)
    }

    fn checked_view_dependencies_for_query<ViewQuery, SourceQuery, Value>(
        &self,
        view: &WorthQueryManagedDerivedView<ViewQuery, Value>,
        product: &WorthQueryProductBranchLease,
        source: &WorthQueryObservedSource<SourceQuery>,
        expected_query: &worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity,
    ) -> Result<std::collections::BTreeSet<super::dependency::ViewDependency>, Denial>
    where
        Value: WorthQueryManagedDerivedValue,
    {
        let state = &view.state;
        if source.runtime_authority != state.runtime_authority {
            return Err(Denial::ForeignApplication);
        }
        if source.schema_binding != state.binding {
            return Err(Denial::ForeignInstallation);
        }
        if &source.query_identity != expected_query {
            return Err(Denial::ForeignQuery);
        }
        if source.branch != state.branch
            || source.selected_product_occurrence() != Some(state.incarnation)
        {
            return Err(Denial::ForeignBranch);
        }
        if source.selected_product_commit() != Some(product.selected_commit()) {
            return Err(Denial::StaleSource);
        }
        let graph = self
            .runtime
            .primary_graph()
            .ok_or(Denial::ForeignApplication)?;
        let facts = source
            .retained_checkpoint_facts(&graph.layout)
            .map_err(|_| Denial::IncompleteDependencies)?;
        source_dependencies(&facts).ok_or(Denial::IncompleteDependencies)
    }

    fn admit_view_product<Query, Value>(
        &self,
        view: &WorthQueryManagedDerivedView<Query, Value>,
        product: &WorthQueryProductBranchLease,
    ) -> Result<(), Denial>
    where
        Value: WorthQueryManagedDerivedValue,
    {
        if view.state.runtime_authority != self.runtime.authority_identity().as_u64() {
            return Err(Denial::ForeignApplication);
        }
        if view.state.binding != self.installed_schema.binding_identity() {
            return Err(Denial::ForeignInstallation);
        }
        if product.observation().lifecycle_incarnation() != view.state.incarnation
            || product.branch_identity() != &view.state.product_branch
            || product.relational_basis().identity().branch_id() != &view.state.branch
        {
            return Err(Denial::ForeignBranch);
        }
        self.admit_current_view_product(product)?;
        if view.state.current_commit().as_ref() != Some(product.selected_commit()) {
            view.state.rebase_cold(product.selected_commit().clone());
        }
        Ok(())
    }

    fn admit_current_view_product(
        &self,
        product: &WorthQueryProductBranchLease,
    ) -> Result<(), Denial> {
        let current = self
            .product_runtime
            .with_product_observation(product.branch_identity(), |observation| {
                Ok((
                    observation.lifecycle_incarnation(),
                    observation.selected_commit().clone(),
                ))
            })
            .map_err(|_| Denial::ForeignBranch)?;
        if current.0 != product.observation().lifecycle_incarnation()
            || current.1 != *product.selected_commit()
        {
            return Err(Denial::StaleSource);
        }
        Ok(())
    }
}
