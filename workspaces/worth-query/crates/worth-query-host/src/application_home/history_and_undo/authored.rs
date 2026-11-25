//! Authored-body undo and redo through `'static` [`WorthQueryAnySourceCustody`].
use std::borrow::Borrow;
use std::fmt;

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryCancellationSource, WorthQueryRequestScope,
};
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
    application_schema::ApplicationSchema,
};
use worth_query_publication::facade::application_entry::{
    WorthQueryApplicationRequestExt, WorthQueryCapturedMutationInput,
    WorthQueryOperationCorrectionPreparation, WorthQuerySourceCompletionAction,
    WorthQuerySourceMutationBinding, WorthQueryTerminalSourceEvidence,
};

use crate::session::auto_key::{
    derive_decimal_correction_key, durable_branch_seed_for_installed, MAX_AUTO_KEY_PROBES,
};
use crate::source_custody::WorthQueryAnySourceCustody;

use super::authored_outcome::{map_any_custody_outcome, WorthQueryHomeAuthoredCorrectionOutcome};
use super::{
    map_seed_denial, map_undo_denial, WorthQueryApplicationHome, WorthQueryCorrectionEntry,
    WorthQueryHomeUndoDenial, DEFAULT_CORRECTION_HISTORY_LIMITS,
};

impl<Schema, Program> WorthQueryApplicationHome<Schema, Program>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema> + 'static,
{
    /// Reverses an authored-body correction (`#[derive(WorthQueryCorrection)]`
    /// with `mode = authored_body`) using [`WorthQueryAnySourceCustody`].
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
        let principal = principal.borrow();
        let scope = WorthQueryRequestScope::new(
            None,
            WorthQueryCancellationSource::new().token(),
            principal.clock().clone(),
        );
        let seed =
            durable_branch_seed_for_installed(self.primary_graph(), self.active_branch(), &scope)
                .map_err(map_seed_denial)?;
        let request = self
            .primary_graph()
            .request(principal, &scope)
            .on_branch(self.active_branch());
        let source = request
            .corrections()
            .select_coordinates(entry.coordinates(), DEFAULT_CORRECTION_HISTORY_LIMITS)
            .map_err(map_undo_denial)?;
        for probe in 0..MAX_AUTO_KEY_PROBES {
            let key = derive_decimal_correction_key(seed, b"undo-authored", probe);
            let prepared = request
                .corrections()
                .reverse(&source)
                .idempotency(&key)
                .limits(DEFAULT_CORRECTION_HISTORY_LIMITS)
                .prepare_operation::<C, Program, C::InverseAction>(self.runtime())
                .map_err(map_undo_denial)?;
            match prepared {
                WorthQueryOperationCorrectionPreparation::Continued(_)
                | WorthQueryOperationCorrectionPreparation::IdempotencyIntentDrift => continue,
                other => {
                    return map_any_custody_outcome(WorthQueryAnySourceCustody::accept_correction(
                        other, &request,
                    ));
                }
            }
        }
        Err(WorthQueryHomeUndoDenial::AutoIdempotencyKeysExhausted)
    }

    /// Reverses an authored-body correction with an explicit caller idempotency
    /// key so retries after crash/reopen recover the exact native completion.
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
        let principal = principal.borrow();
        let scope = WorthQueryRequestScope::new(
            None,
            WorthQueryCancellationSource::new().token(),
            principal.clock().clone(),
        );
        let request = self
            .primary_graph()
            .request(principal, &scope)
            .on_branch(self.active_branch());
        let corrections = request.corrections();
        let source = corrections
            .select_coordinates(entry.coordinates(), DEFAULT_CORRECTION_HISTORY_LIMITS)
            .map_err(map_undo_denial)?;
        let prepared = corrections
            .reverse(&source)
            .idempotency(idempotency_key)
            .limits(DEFAULT_CORRECTION_HISTORY_LIMITS)
            .prepare_operation::<C, Program, C::InverseAction>(self.runtime())
            .map_err(map_undo_denial)?;
        map_any_custody_outcome(WorthQueryAnySourceCustody::accept_correction(
            prepared, &request,
        ))
    }

    /// Reapplies a previously reversed authored-body entry via
    /// [`WorthQueryAnySourceCustody`] with a fresh key.
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
        let principal = principal.borrow();
        let scope = WorthQueryRequestScope::new(
            None,
            WorthQueryCancellationSource::new().token(),
            principal.clock().clone(),
        );
        let seed =
            durable_branch_seed_for_installed(self.primary_graph(), self.active_branch(), &scope)
                .map_err(map_seed_denial)?;
        let request = self
            .primary_graph()
            .request(principal, &scope)
            .on_branch(self.active_branch());
        let alternative = self.select_redo_alternative(&request, entry)?;
        for probe in 0..MAX_AUTO_KEY_PROBES {
            let key = derive_decimal_correction_key(seed, b"redo-authored", probe);
            let prepared = request
                .corrections()
                .reapply(&alternative)
                .idempotency(&key)
                .limits(DEFAULT_CORRECTION_HISTORY_LIMITS)
                .prepare_operation::<C, Program, C::SourceAction>(self.runtime())
                .map_err(map_undo_denial)?;
            match prepared {
                WorthQueryOperationCorrectionPreparation::Continued(_)
                | WorthQueryOperationCorrectionPreparation::IdempotencyIntentDrift => continue,
                other => {
                    return map_any_custody_outcome(WorthQueryAnySourceCustody::accept_correction(
                        other, &request,
                    ));
                }
            }
        }
        Err(WorthQueryHomeUndoDenial::AutoIdempotencyKeysExhausted)
    }

    /// Reapplies a previously reversed authored-body entry with an explicit key.
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
        let principal = principal.borrow();
        let scope = WorthQueryRequestScope::new(
            None,
            WorthQueryCancellationSource::new().token(),
            principal.clock().clone(),
        );
        let request = self
            .primary_graph()
            .request(principal, &scope)
            .on_branch(self.active_branch());
        let alternative = self.select_redo_alternative(&request, entry)?;
        let prepared = request
            .corrections()
            .reapply(&alternative)
            .idempotency(idempotency_key)
            .limits(DEFAULT_CORRECTION_HISTORY_LIMITS)
            .prepare_operation::<C, Program, C::SourceAction>(self.runtime())
            .map_err(map_undo_denial)?;
        map_any_custody_outcome(WorthQueryAnySourceCustody::accept_correction(
            prepared, &request,
        ))
    }
}
