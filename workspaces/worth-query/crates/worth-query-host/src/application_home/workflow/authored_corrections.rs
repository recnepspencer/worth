//! Authored-body undo and redo of the paired Home, on its workflow Home.
//!
//! Each method is the same call on [`WorthQueryApplicationHome`] with the same
//! signature and bounds; custody and refusal are those of the plain Home.
use super::*;
use std::borrow::Borrow;
use std::fmt;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_declaration::facade::{
    application_aftermath::{
        ApplicationAuthoredBodyCorrection, ApplicationAuthoredBodyReapplication,
        ApplicationCorrectionBinding, ApplicationInverseBinding,
    },
    application_operation::{
        ApplicationConventionalAdmission, ApplicationMutationAdmissionBinding,
        ApplicationMutationBinding, ApplicationMutationInputScope,
        ApplicationMutationScopeResolution,
    },
    application_program::ApplicationProgramDefinition,
};
use worth_query_publication::facade::application_entry::{
    WorthQueryCapturedMutationInput, WorthQuerySourceCompletionAction,
    WorthQuerySourceMutationBinding, WorthQueryTerminalSourceEvidence,
};

use crate::application_home::{
    WorthQueryCorrectionEntry, WorthQueryHomeAuthoredCorrectionOutcome, WorthQueryHomeUndoDenial,
};

impl<Schema, Spec, Program> WorthQueryWorkflowApplicationHome<Schema, Spec, Program>
where
    Schema: ApplicationSchema + 'static,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    Program: ApplicationProgramDefinition<Schema> + 'static,
{
    /// [`WorthQueryApplicationHome::undo_authored`].
    pub fn undo_authored<C>(
        &self,
        entry: &WorthQueryCorrectionEntry,
        principal: impl Borrow<WorthQueryAuthenticatedExternalPrincipal<Schema>>,
    ) -> Result<WorthQueryHomeAuthoredCorrectionOutcome<Schema, Program>, WorthQueryHomeUndoDenial>
    where
        C: ApplicationAuthoredBodyCorrection<Schema, Program>,
        ApplicationInverseBinding<Schema, C>: WorthQuerySourceMutationBinding<Schema>
            + ApplicationMutationAdmissionBinding<
                Schema,
                Admission = ApplicationConventionalAdmission,
            >,
        <ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::Input:
            Clone
                + Send
                + Sync
                + ApplicationMutationInputScope<Schema, ApplicationInverseBinding<Schema, C>>,
        <ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::IdempotencyKey:
            Clone,
        <ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::ScopeBinding:
            ApplicationMutationScopeResolution<
                Schema,
                <ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
            >,
        C::InverseAction: WorthQuerySourceCompletionAction<
                Schema,
                ApplicationInverseBinding<Schema, C>,
            > + 'static,
        WorthQueryTerminalSourceEvidence<
            Schema,
            WorthQueryCapturedMutationInput<Schema, ApplicationInverseBinding<Schema, C>>,
        >: fmt::Debug,
{
        self.home.undo_authored::<C>(entry, principal)
    }

    /// [`WorthQueryApplicationHome::undo_authored_with_key`].
    pub fn undo_authored_with_key<C>(
        &self,
        entry: &WorthQueryCorrectionEntry,
        principal: impl Borrow<WorthQueryAuthenticatedExternalPrincipal<Schema>>,
        idempotency_key: &str,
    ) -> Result<WorthQueryHomeAuthoredCorrectionOutcome<Schema, Program>, WorthQueryHomeUndoDenial>
    where
        C: ApplicationAuthoredBodyCorrection<Schema, Program>,
        ApplicationInverseBinding<Schema, C>: WorthQuerySourceMutationBinding<Schema>
            + ApplicationMutationAdmissionBinding<
                Schema,
                Admission = ApplicationConventionalAdmission,
            >,
        <ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::Input:
            Clone
                + Send
                + Sync
                + ApplicationMutationInputScope<Schema, ApplicationInverseBinding<Schema, C>>,
        <ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::IdempotencyKey:
            Clone,
        <ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::ScopeBinding:
            ApplicationMutationScopeResolution<
                Schema,
                <ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
            >,
        C::InverseAction: WorthQuerySourceCompletionAction<
                Schema,
                ApplicationInverseBinding<Schema, C>,
            > + 'static,
        WorthQueryTerminalSourceEvidence<
            Schema,
            WorthQueryCapturedMutationInput<Schema, ApplicationInverseBinding<Schema, C>>,
        >: fmt::Debug,
{
        self.home
            .undo_authored_with_key::<C>(entry, principal, idempotency_key)
    }

    /// [`WorthQueryApplicationHome::redo_authored`].
    pub fn redo_authored<C>(
        &self,
        entry: &WorthQueryCorrectionEntry,
        principal: impl Borrow<WorthQueryAuthenticatedExternalPrincipal<Schema>>,
    ) -> Result<WorthQueryHomeAuthoredCorrectionOutcome<Schema, Program>, WorthQueryHomeUndoDenial>
    where
        C: ApplicationAuthoredBodyReapplication<Schema, Program>,
        <C as ApplicationCorrectionBinding<Schema>>::Source: WorthQuerySourceMutationBinding<Schema>
            + ApplicationMutationAdmissionBinding<
                Schema,
                Admission = ApplicationConventionalAdmission,
            >,
        <<C as ApplicationCorrectionBinding<Schema>>::Source as ApplicationMutationBinding<
            Schema,
        >>::Input: Clone
            + Send
            + Sync
            + ApplicationMutationInputScope<
                Schema,
                <C as ApplicationCorrectionBinding<Schema>>::Source,
            >,
        <<C as ApplicationCorrectionBinding<Schema>>::Source as ApplicationMutationBinding<
            Schema,
        >>::IdempotencyKey: Clone,
        <<C as ApplicationCorrectionBinding<Schema>>::Source as ApplicationMutationBinding<
            Schema,
        >>::ScopeBinding: ApplicationMutationScopeResolution<
            Schema,
            <<C as ApplicationCorrectionBinding<Schema>>::Source as ApplicationMutationBinding<
                Schema,
            >>::PrincipalIdentity,
        >,
        C::SourceAction: WorthQuerySourceCompletionAction<
                Schema,
                <C as ApplicationCorrectionBinding<Schema>>::Source,
            > + 'static,
        WorthQueryTerminalSourceEvidence<
            Schema,
            WorthQueryCapturedMutationInput<
                Schema,
                <C as ApplicationCorrectionBinding<Schema>>::Source,
            >,
        >: fmt::Debug,
    {
        self.home.redo_authored::<C>(entry, principal)
    }

    /// [`WorthQueryApplicationHome::redo_authored_with_key`].
    pub fn redo_authored_with_key<C>(
        &self,
        entry: &WorthQueryCorrectionEntry,
        principal: impl Borrow<WorthQueryAuthenticatedExternalPrincipal<Schema>>,
        idempotency_key: &str,
    ) -> Result<WorthQueryHomeAuthoredCorrectionOutcome<Schema, Program>, WorthQueryHomeUndoDenial>
    where
        C: ApplicationAuthoredBodyReapplication<Schema, Program>,
        <C as ApplicationCorrectionBinding<Schema>>::Source: WorthQuerySourceMutationBinding<Schema>
            + ApplicationMutationAdmissionBinding<
                Schema,
                Admission = ApplicationConventionalAdmission,
            >,
        <<C as ApplicationCorrectionBinding<Schema>>::Source as ApplicationMutationBinding<
            Schema,
        >>::Input: Clone
            + Send
            + Sync
            + ApplicationMutationInputScope<
                Schema,
                <C as ApplicationCorrectionBinding<Schema>>::Source,
            >,
        <<C as ApplicationCorrectionBinding<Schema>>::Source as ApplicationMutationBinding<
            Schema,
        >>::IdempotencyKey: Clone,
        <<C as ApplicationCorrectionBinding<Schema>>::Source as ApplicationMutationBinding<
            Schema,
        >>::ScopeBinding: ApplicationMutationScopeResolution<
            Schema,
            <<C as ApplicationCorrectionBinding<Schema>>::Source as ApplicationMutationBinding<
                Schema,
            >>::PrincipalIdentity,
        >,
        C::SourceAction: WorthQuerySourceCompletionAction<
                Schema,
                <C as ApplicationCorrectionBinding<Schema>>::Source,
            > + 'static,
        WorthQueryTerminalSourceEvidence<
            Schema,
            WorthQueryCapturedMutationInput<
                Schema,
                <C as ApplicationCorrectionBinding<Schema>>::Source,
            >,
        >: fmt::Debug,
    {
        self.home
            .redo_authored_with_key::<C>(entry, principal, idempotency_key)
    }
}
