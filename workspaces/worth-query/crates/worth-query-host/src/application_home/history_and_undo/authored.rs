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
use worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt;
use worth_query_publication::facade::application_entry::{
    WorthQueryApplicationRequestExt, WorthQueryCapturedMutationInput, WorthQueryClosedOperation,
    WorthQuerySourceCompletionAction, WorthQuerySourceMutationBinding,
    WorthQueryTerminalSourceEvidence,
};
use worth_runtime_world::facade::ProductUnpublishedRecoveryHandle;

use crate::source_custody::{
    WorthQueryAnyCorrectionCustodyOutcome, WorthQueryAnySourceCompletion,
    WorthQueryAnySourceCustody, WorthQueryAnySourceCustodyStep,
    WorthQueryAnyTerminalSourceEvidence, WorthQuerySourceCustodyPending,
};

use super::{
    map_undo_denial, next_restart_safe_correction_key, WorthQueryApplicationHome,
    WorthQueryCorrectionEntry, WorthQueryHomeUndoDenial, DEFAULT_CORRECTION_HISTORY_LIMITS,
};

/// Outcome of an authored-body undo or redo via [`WorthQueryAnySourceCustody`].
#[must_use = "handle the committed correction or resume pending source custody"]
pub enum WorthQueryHomeAuthoredCorrectionOutcome<Schema, Program> {
    Committed {
        closed: WorthQueryClosedOperation,
        evidence: WorthQueryAnySourceCompletion,
    },
    Declined {
        closed: WorthQueryClosedOperation,
        evidence: WorthQueryAnyTerminalSourceEvidence,
    },
    Pending {
        custody: WorthQueryAnySourceCustody<Schema, Program>,
        cause: WorthQuerySourceCustodyPending,
    },
    IdempotencyIntentDrift,
    PublicationPending,
    ProductUnpublished(ProductUnpublishedRecoveryHandle),
}

impl<Schema, Program> WorthQueryHomeAuthoredCorrectionOutcome<Schema, Program> {
    /// Returns the live commit receipt when this correction committed freshly.
    pub fn live_receipt(&self) -> Option<&WorthQueryApplicationCommitReceipt> {
        match self {
            Self::Committed { evidence, .. } => evidence.live_receipt(),
            _ => None,
        }
    }
}

impl<Schema, Program> fmt::Debug for WorthQueryHomeAuthoredCorrectionOutcome<Schema, Program> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Committed { .. } => f.write_str("Committed(..)"),
            Self::Declined { evidence, .. } => f.debug_tuple("Declined").field(evidence).finish(),
            Self::Pending { cause, .. } => f.debug_tuple("Pending").field(cause).finish(),
            Self::IdempotencyIntentDrift => f.write_str("IdempotencyIntentDrift"),
            Self::PublicationPending => f.write_str("PublicationPending"),
            Self::ProductUnpublished(_) => f.write_str("ProductUnpublished(..)"),
        }
    }
}

fn map_any_custody_outcome<Schema, Program>(
    outcome: WorthQueryAnyCorrectionCustodyOutcome<Schema, Program>,
) -> Result<WorthQueryHomeAuthoredCorrectionOutcome<Schema, Program>, WorthQueryHomeUndoDenial> {
    use WorthQueryHomeAuthoredCorrectionOutcome as Out;
    match outcome {
        WorthQueryAnyCorrectionCustodyOutcome::Custody(step) => Ok(match step {
            WorthQueryAnySourceCustodyStep::Committed { closed, evidence } => {
                Out::Committed { closed, evidence }
            }
            WorthQueryAnySourceCustodyStep::Declined { closed, evidence } => {
                Out::Declined { closed, evidence }
            }
            WorthQueryAnySourceCustodyStep::Pending { custody, cause } => {
                Out::Pending { custody, cause }
            }
        }),
        WorthQueryAnyCorrectionCustodyOutcome::PublicationRefused(denial) => {
            Err(map_undo_denial(denial))
        }
        WorthQueryAnyCorrectionCustodyOutcome::IdempotencyIntentDrift => {
            Ok(Out::IdempotencyIntentDrift)
        }
        WorthQueryAnyCorrectionCustodyOutcome::PublicationPending => Ok(Out::PublicationPending),
        WorthQueryAnyCorrectionCustodyOutcome::ProductUnpublished(handle) => {
            Ok(Out::ProductUnpublished(handle))
        }
    }
}

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
        let key = next_restart_safe_correction_key();
        self.undo_authored_with_key::<C>(entry, principal, &key)
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
        let key = next_restart_safe_correction_key();
        self.redo_authored_with_key::<C>(entry, principal, &key)
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
