//! Two independently authorized public entry reads retained as one entry.

use worth_query_declaration::facade::{
    application_query::ApplicationDerivedViewDefinition, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationQuery, WorthQueryInstalledApplicationQueryAuthorization,
};

use super::{
    Denial, WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationProjection,
    WorthQueryManagedDerivedValue, WorthQueryManagedDerivedView,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedViewKey;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationOneShotResult;

mod read;
mod reconcile;
mod reconstruct;
mod reconstruct_lazy;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn open_managed_derived_collection_pair<
        MembershipQuery,
        MembershipParameters,
        MembershipResult,
        MembershipScope,
        FirstQuery,
        FirstParameters,
        FirstResult,
        FirstScope,
        SecondQuery,
        SecondParameters,
        SecondResult,
        SecondScope,
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
        first_query: &WorthQueryInstalledApplicationQuery<
            Schema,
            FirstQuery,
            FirstParameters,
            FirstResult,
            FirstScope,
        >,
        second_query: &WorthQueryInstalledApplicationQuery<
            Schema,
            SecondQuery,
            SecondParameters,
            SecondResult,
            SecondScope,
        >,
        product: &WorthQueryProductBranchLease,
    ) -> Result<WorthQueryManagedDerivedView<MembershipQuery, Value>, Denial>
    where
        Value: WorthQueryManagedDerivedValue,
    {
        for valid in [
            self.installed_schema
                .validate_installed_query(first_query)
                .is_ok(),
            self.installed_schema
                .validate_installed_query(second_query)
                .is_ok(),
        ] {
            if !valid {
                return Err(Denial::ForeignInstallation);
            }
        }
        if !matches!(
            first_query.authorization(),
            WorthQueryInstalledApplicationQueryAuthorization::Public
        ) || !matches!(
            second_query.authorization(),
            WorthQueryInstalledApplicationQueryAuthorization::Public
        ) {
            return Err(Denial::AuthorizationRequired);
        }
        self.open_managed_derived_view_with_entry_query(
            definition,
            membership_query,
            Some(first_query.identity().clone()),
            Some(second_query.identity().clone()),
            product,
        )
    }

    /// The second plan is admitted from the first observed row. Both rows must
    /// agree on their declared link key. All four source sets
    /// (two rows, two result sets) are
    /// retained before the projected value can become current.
    pub fn refresh_managed_derived_collection_pair_entry<
        'a,
        MembershipQuery,
        FirstQuery,
        FirstParameters,
        FirstResult,
        FirstPrincipal,
        FirstPrincipalIdentity,
        FirstScope,
        SecondQuery,
        SecondResult,
        Value,
        LinkKey,
    >(
        &self,
        view: &WorthQueryManagedDerivedView<MembershipQuery, Value>,
        product: &WorthQueryProductBranchLease,
        key: &WorthQueryManagedDerivedViewKey,
        first: WorthQueryAdmittedApplicationQueryPlan<
            'a,
            Schema,
            FirstQuery,
            FirstParameters,
            FirstResult,
            FirstPrincipal,
            FirstPrincipalIdentity,
            FirstScope,
        >,
        second_for: impl FnOnce(
            &FirstResult,
        ) -> Result<
            WorthQueryApplicationOneShotResult<SecondQuery, SecondResult>,
            Denial,
        >,
        first_link: impl FnOnce(&FirstResult) -> LinkKey,
        second_link: impl FnOnce(&SecondResult) -> LinkKey,
        project: impl FnOnce(&FirstResult, &SecondResult) -> Value,
    ) -> Result<std::sync::Arc<Value>, Denial>
    where
        FirstQuery: 'a,
        FirstParameters: 'a,
        FirstResult: WorthQueryApplicationProjection<Schema, FirstQuery> + 'a,
        FirstPrincipal: 'a,
        FirstPrincipalIdentity: 'a,
        FirstScope: 'a,
        SecondResult: WorthQueryApplicationProjection<Schema, SecondQuery> + 'a,
        Value: WorthQueryManagedDerivedValue,
        LinkKey: Eq,
    {
        view.state
            .requires_entry_refresh(key, product.selected_commit())?;
        let (issued_key, value, dependencies) = self.read_managed_entry_pair(
            view,
            product,
            key.root(),
            first,
            second_for,
            first_link,
            second_link,
            project,
        )?;
        if &issued_key != key {
            return Err(Denial::IncompleteDependencies);
        }
        view.state
            .refresh_entry(key, value, dependencies, product.selected_commit())
    }
}
