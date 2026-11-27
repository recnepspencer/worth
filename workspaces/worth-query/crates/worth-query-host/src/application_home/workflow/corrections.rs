//! Durable correction history and recorded-inverse undo/redo of the paired
//! Home, on its workflow Home.
//!
//! Each method is the same call on [`WorthQueryApplicationHome`] with the
//! same signature. Every call admits the principal afresh on the Home's active
//! branch; a workflow transition is corrected only where its own recorded
//! correction binding and disclosure admit it, exactly as on the plain Home.
use super::*;
use std::borrow::Borrow;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_program::ApplicationProgramDefinition;
use worth_query_publication::facade::application_entry::WorthQueryApplicationCorrectionHistoryDenial;

use crate::application_home::{
    WorthQueryCorrectionEntry, WorthQueryCorrectionHistory, WorthQueryCorrectionHistoryLimit,
    WorthQueryHomeCorrectionOutcome, WorthQueryHomeUndoDenial,
};

impl<Schema, Spec, Program> WorthQueryWorkflowApplicationHome<Schema, Spec, Program>
where
    Schema: ApplicationSchema + 'static,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    Program: ApplicationProgramDefinition<Schema> + 'static,
{
    /// [`WorthQueryApplicationHome::correction_history`].
    pub fn correction_history(
        &self,
        principal: impl Borrow<WorthQueryAuthenticatedExternalPrincipal<Schema>>,
        limit: WorthQueryCorrectionHistoryLimit,
    ) -> Result<WorthQueryCorrectionHistory<Schema>, WorthQueryApplicationCorrectionHistoryDenial>
    {
        self.home.correction_history(principal, limit)
    }

    /// [`WorthQueryApplicationHome::correction_history_with_scope`].
    pub fn correction_history_with_scope(
        &self,
        principal: impl Borrow<WorthQueryAuthenticatedExternalPrincipal<Schema>>,
        limit: WorthQueryCorrectionHistoryLimit,
        scope: &WorthQueryRequestScope,
    ) -> Result<WorthQueryCorrectionHistory<Schema>, WorthQueryApplicationCorrectionHistoryDenial>
    {
        self.home
            .correction_history_with_scope(principal, limit, scope)
    }

    /// [`WorthQueryApplicationHome::undo`].
    pub fn undo(
        &self,
        entry: &WorthQueryCorrectionEntry,
        principal: impl Borrow<WorthQueryAuthenticatedExternalPrincipal<Schema>>,
    ) -> Result<WorthQueryHomeCorrectionOutcome, WorthQueryHomeUndoDenial> {
        self.home.undo(entry, principal)
    }

    /// [`WorthQueryApplicationHome::undo_with_key`].
    pub fn undo_with_key(
        &self,
        entry: &WorthQueryCorrectionEntry,
        principal: impl Borrow<WorthQueryAuthenticatedExternalPrincipal<Schema>>,
        idempotency_key: &str,
    ) -> Result<WorthQueryHomeCorrectionOutcome, WorthQueryHomeUndoDenial> {
        self.home.undo_with_key(entry, principal, idempotency_key)
    }

    /// [`WorthQueryApplicationHome::redo`].
    pub fn redo(
        &self,
        entry: &WorthQueryCorrectionEntry,
        principal: impl Borrow<WorthQueryAuthenticatedExternalPrincipal<Schema>>,
    ) -> Result<WorthQueryHomeCorrectionOutcome, WorthQueryHomeUndoDenial> {
        self.home.redo(entry, principal)
    }

    /// [`WorthQueryApplicationHome::redo_with_key`].
    pub fn redo_with_key(
        &self,
        entry: &WorthQueryCorrectionEntry,
        principal: impl Borrow<WorthQueryAuthenticatedExternalPrincipal<Schema>>,
        idempotency_key: &str,
    ) -> Result<WorthQueryHomeCorrectionOutcome, WorthQueryHomeUndoDenial> {
        self.home.redo_with_key(entry, principal, idempotency_key)
    }
}
