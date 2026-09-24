//! One retained collection with an independently admitted public entry query.

use worth_query_declaration::facade::{
    application_query::ApplicationDerivedViewDefinition, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationQuery, WorthQueryInstalledApplicationQueryAuthorization,
};

use super::{
    Denial, WorthQueryManagedDerivedValue, WorthQueryManagedDerivedView,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::primary_graph::application_query::{
    WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationProjection,
};

mod pair;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    /// Registers one collection handle. The entry query is a separately
    /// installed public declaration, not an untracked projection callback.
    pub fn open_managed_derived_collection<
        MembershipQuery,
        MembershipParameters,
        MembershipResult,
        MembershipScope,
        EntryQuery,
        EntryParameters,
        EntryResult,
        EntryScope,
        Value,
    >(
        &self,
        definition: &ApplicationDerivedViewDefinition<
            Schema,
            MembershipQuery,
            MembershipParameters,
            MembershipResult,
            MembershipScope,
        >,
        membership_query: &WorthQueryInstalledApplicationQuery<
            Schema,
            MembershipQuery,
            MembershipParameters,
            MembershipResult,
            MembershipScope,
        >,
        entry_query: &WorthQueryInstalledApplicationQuery<
            Schema,
            EntryQuery,
            EntryParameters,
            EntryResult,
            EntryScope,
        >,
        product: &WorthQueryProductBranchLease,
    ) -> Result<WorthQueryManagedDerivedView<MembershipQuery, Value>, Denial>
    where
        Value: WorthQueryManagedDerivedValue,
    {
        self.installed_schema
            .validate_installed_query(entry_query)
            .map_err(|_| Denial::ForeignInstallation)?;
        if !matches!(
            entry_query.authorization(),
            WorthQueryInstalledApplicationQueryAuthorization::Public
        ) {
            return Err(Denial::AuthorizationRequired);
        }
        self.open_managed_derived_view_with_entry_query(
            definition,
            membership_query,
            Some(entry_query.identity().clone()),
            None,
            product,
        )
    }

    /// Executes the declared scoped Query under its fresh admitted authority;
    /// both row and scoped result-set facts become entry dependencies.
    pub fn refresh_managed_derived_collection_entry<
        'a,
        MembershipQuery,
        EntryQuery,
        EntryParameters,
        EntryResult,
        Principal,
        PrincipalIdentity,
        EntryScope,
        Value,
    >(
        &self,
        view: &WorthQueryManagedDerivedView<MembershipQuery, Value>,
        product: &WorthQueryProductBranchLease,
        key: &super::super::WorthQueryManagedDerivedViewKey,
        plan: WorthQueryAdmittedApplicationQueryPlan<
            'a,
            Schema,
            EntryQuery,
            EntryParameters,
            EntryResult,
            Principal,
            PrincipalIdentity,
            EntryScope,
        >,
        project: impl FnOnce(&EntryResult) -> Value,
    ) -> Result<std::sync::Arc<Value>, Denial>
    where
        EntryResult: WorthQueryApplicationProjection<Schema, EntryQuery>,
        Value: WorthQueryManagedDerivedValue,
    {
        self.admit_view_product(view, product)?;
        view.state
            .requires_entry_refresh(key, product.selected_commit())?;
        if view.state.secondary_entry_query.is_some() {
            return Err(Denial::ForeignQuery);
        }
        let expected = view
            .state
            .entry_query
            .as_ref()
            .ok_or(Denial::ForeignQuery)?;
        if plan.query_identity() != expected || plan.scope.entity_id() != key.root() {
            return Err(Denial::ForeignQuery);
        }
        let result = self
            .execute_application_query_one_shot(plan)
            .map_err(|_| Denial::QueryExecutionDenied)?;
        self.admit_view_product(view, product)?;
        let ([row], [source]) = (result.rows(), result.observed_sources()) else {
            return Err(Denial::IncompleteDependencies);
        };
        if source.source_root() != key.root() {
            return Err(Denial::IncompleteDependencies);
        }
        let mut dependencies =
            self.checked_view_dependencies_for_query(view, product, source, expected)?;
        dependencies.extend(self.checked_view_dependencies_for_query(
            view,
            product,
            result.result_set_observation().source(),
            expected,
        )?);
        view.state
            .refresh_entry(key, project(row), dependencies, product.selected_commit())
    }
}
