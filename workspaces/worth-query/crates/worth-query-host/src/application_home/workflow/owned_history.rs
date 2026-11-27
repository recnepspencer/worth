//! Owned `'static` history documents of the paired Home, on its workflow Home.
//!
//! Each method is the same call on [`WorthQueryApplicationHome`] with the same
//! signature: the document forks a dedicated head from the active branch and
//! keeps the plain Home's head custody on every refusal.
use super::*;
use std::borrow::Borrow;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_history::ApplicationHistoryBinding,
    application_program::ApplicationProgramDefinition,
    application_query::{ApplicationQueryBinding, ApplicationQueryScopeResolution},
};
use worth_query_execution::facade::{
    primary_graph::WorthQueryApplicationProjection, product::WorthQueryOwnedProductHead,
};
use worth_query_publication::facade::application_entry::WorthQueryOwnedApplicationHistoryDocument;

use crate::application_home::history_and_undo::{
    HistoryAccessBinding, HistoryPrincipal, HistoryQuery, HistoryRows, HistoryScope,
};
use crate::application_home::WorthQueryHomeOwnedHistoryDenial;

impl<Schema, Spec, Program> WorthQueryWorkflowApplicationHome<Schema, Spec, Program>
where
    Schema: ApplicationSchema + 'static,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    Program: ApplicationProgramDefinition<Schema> + 'static,
{
    /// [`WorthQueryApplicationHome::owned_history`].
    pub fn owned_history<'home, H>(
        &'home self,
        principal: impl Borrow<WorthQueryAuthenticatedExternalPrincipal<Schema>>,
        access: H::Access,
    ) -> Result<
        WorthQueryOwnedApplicationHistoryDocument<Schema, H>,
        WorthQueryHomeOwnedHistoryDenial<'home, Schema>,
    >
    where
        H: ApplicationHistoryBinding<Schema>,
        <HistoryAccessBinding<Schema, H> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<Schema, HistoryPrincipal<Schema, H>>,
        HistoryRows<Schema, H>: WorthQueryApplicationProjection<Schema, HistoryQuery<Schema, H>>,
        HistoryScope<Schema, H>: 'static,
    {
        self.home.owned_history::<H>(principal, access)
    }

    /// [`WorthQueryApplicationHome::owned_history_with_scope`].
    pub fn owned_history_with_scope<'home, H>(
        &'home self,
        principal: impl Borrow<WorthQueryAuthenticatedExternalPrincipal<Schema>>,
        access: H::Access,
        scope: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryOwnedApplicationHistoryDocument<Schema, H>,
        WorthQueryHomeOwnedHistoryDenial<'home, Schema>,
    >
    where
        H: ApplicationHistoryBinding<Schema>,
        <HistoryAccessBinding<Schema, H> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<Schema, HistoryPrincipal<Schema, H>>,
        HistoryRows<Schema, H>: WorthQueryApplicationProjection<Schema, HistoryQuery<Schema, H>>,
        HistoryScope<Schema, H>: 'static,
    {
        self.home
            .owned_history_with_scope::<H>(principal, access, scope)
    }

    /// [`WorthQueryApplicationHome::attach_owned_history_head`].
    pub fn attach_owned_history_head<'home, H>(
        &'home self,
        head: WorthQueryOwnedProductHead<'home, Schema>,
        principal: impl Borrow<WorthQueryAuthenticatedExternalPrincipal<Schema>>,
        access: H::Access,
        scope: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryOwnedApplicationHistoryDocument<Schema, H>,
        WorthQueryHomeOwnedHistoryDenial<'home, Schema>,
    >
    where
        H: ApplicationHistoryBinding<Schema>,
        <HistoryAccessBinding<Schema, H> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<Schema, HistoryPrincipal<Schema, H>>,
        HistoryRows<Schema, H>: WorthQueryApplicationProjection<Schema, HistoryQuery<Schema, H>>,
        HistoryScope<Schema, H>: 'static,
    {
        self.home
            .attach_owned_history_head::<H>(head, principal, access, scope)
    }
}
