//! Restart-safe Home undo/redo and owned `'static` document history facade.
use std::borrow::Borrow;
use std::num::NonZeroUsize;

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_program::ApplicationProgramDefinition, application_schema::ApplicationSchema,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryCorrectionDerivationDenial,
    WorthQueryCorrectionSourceCoordinates, WorthQueryRecoveredApplicationCommitReceipt,
};
use worth_query_publication::facade::application_entry::{
    WorthQueryApplicationCorrectionDenial, WorthQueryApplicationCorrectionOutcome,
    WorthQueryApplicationCorrectionPreparation, WorthQueryApplicationMutationOutcome,
    WorthQueryApplicationRequest, WorthQueryApplicationRequestExt,
    WorthQueryCorrectionHistoryLimits, WorthQueryCorrectionValue,
    WorthQuerySelectedCorrectionAlternative,
};
use worth_runtime_world::facade::ProductUnpublishedRecoveryHandle;

use crate::session::auto_key::{
    derive_decimal_correction_key, durable_branch_seed_for_installed, DurableSeedDenial,
    MAX_AUTO_KEY_PROBES,
};

use super::WorthQueryApplicationHome;

mod authored;
mod authored_outcome;
mod owned_history;
pub use authored_outcome::WorthQueryHomeAuthoredCorrectionOutcome;
pub use owned_history::WorthQueryHomeOwnedHistoryDenial;

/// Default bounded history traversal limits for Home undo and redo helpers.
pub const DEFAULT_CORRECTION_HISTORY_LIMITS: WorthQueryCorrectionHistoryLimits =
    WorthQueryCorrectionHistoryLimits::new(
        NonZeroUsize::new(1024),
        match NonZeroUsize::new(64) {
            Some(limit) => limit,
            None => unreachable!(),
        },
    );

const MAX_ALTERNATIVE_PAGES: usize = 64;
const ALTERNATIVE_PAGE_SIZE: usize = 32;

/// Serializable, `'static` handle identifying one committed action for later
/// undo or redo — including after closing and reopening a Directory Home.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryCorrectionEntry {
    coordinates: WorthQueryCorrectionSourceCoordinates,
}

impl WorthQueryCorrectionEntry {
    /// Captures exact correction coordinates from a live commit receipt.
    pub fn from_receipt(receipt: &WorthQueryApplicationCommitReceipt) -> Self {
        Self {
            coordinates: receipt.correction_source().coordinates(),
        }
    }

    /// Restores a correction entry from persisted coordinates.
    pub const fn from_coordinates(coordinates: WorthQueryCorrectionSourceCoordinates) -> Self {
        Self { coordinates }
    }

    /// Borrows the exact native correction coordinates.
    pub const fn coordinates(&self) -> &WorthQueryCorrectionSourceCoordinates {
        &self.coordinates
    }
}

/// Why [`WorthQueryApplicationHome::undo`], [`WorthQueryApplicationHome::undo_authored`],
/// [`WorthQueryApplicationHome::redo`], or [`WorthQueryApplicationHome::redo_authored`]
/// was refused.
#[derive(Debug)]
pub enum WorthQueryHomeUndoDenial {
    /// A conflicting edit diverged from the recorded source since it committed.
    Conflict,
    /// No matching alternative was found in bounded history traversal for redo.
    NoAlternativeAvailable,
    /// All bounded deterministic auto-idempotency probe slots were already occupied.
    AutoIdempotencyKeysExhausted,
    /// Any other exact typed correction refusal.
    Correction(WorthQueryApplicationCorrectionDenial),
}

pub(super) fn map_undo_denial(
    denial: WorthQueryApplicationCorrectionDenial,
) -> WorthQueryHomeUndoDenial {
    match denial {
        WorthQueryApplicationCorrectionDenial::Derivation(
            WorthQueryCorrectionDerivationDenial::SourceDiverged,
        ) => WorthQueryHomeUndoDenial::Conflict,
        other => WorthQueryHomeUndoDenial::Correction(other),
    }
}

pub(super) fn map_seed_denial(denial: DurableSeedDenial) -> WorthQueryHomeUndoDenial {
    match denial {
        DurableSeedDenial::Advancement(cause) => WorthQueryHomeUndoDenial::Correction(
            WorthQueryApplicationCorrectionDenial::ExecutionRequest(cause),
        ),
        DurableSeedDenial::Selection(cause) => WorthQueryHomeUndoDenial::Correction(
            WorthQueryApplicationCorrectionDenial::ProductSelection(cause),
        ),
    }
}

/// Outcome of a conventional recorded-inverse undo or redo on a Home.
#[derive(Debug)]
pub enum WorthQueryHomeCorrectionOutcome {
    Committed {
        receipt: WorthQueryApplicationCommitReceipt,
        result: WorthQueryCorrectionValue,
    },
    AlreadyCommitted(WorthQueryApplicationCommitReceipt),
    RecoveredCommitted(WorthQueryRecoveredApplicationCommitReceipt),
    DomainDenied(WorthQueryCorrectionValue),
    Other(WorthQueryApplicationCorrectionOutcome),
    PublicationPending,
    ProductUnpublished(ProductUnpublishedRecoveryHandle),
}

/// Pre-publication `Settled` outcomes prove the key was already occupied before
/// this attempt submitted; `RecoveredCommitted` after `publish()` is never retried.
fn is_pre_publication_occupied(outcome: &WorthQueryApplicationCorrectionOutcome) -> bool {
    matches!(
        outcome,
        WorthQueryApplicationMutationOutcome::AlreadyCommitted(_)
            | WorthQueryApplicationMutationOutcome::RecoveredCommitted(_)
            | WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift
    )
}

fn finish_auto_correction_preparation(
    prepared: WorthQueryApplicationCorrectionPreparation<'_>,
) -> Result<Option<WorthQueryHomeCorrectionOutcome>, WorthQueryHomeUndoDenial> {
    match prepared {
        WorthQueryApplicationCorrectionPreparation::Settled(outcome)
            if is_pre_publication_occupied(&outcome) =>
        {
            Ok(None)
        }
        WorthQueryApplicationCorrectionPreparation::Settled(outcome) => {
            Ok(Some(map_conventional_outcome(outcome)))
        }
        WorthQueryApplicationCorrectionPreparation::Prepared(prepared) => {
            let published = prepared.publish().map_err(map_undo_denial)?;
            if matches!(
                published,
                WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift
            ) {
                return Ok(None);
            }
            Ok(Some(map_conventional_outcome(published)))
        }
        WorthQueryApplicationCorrectionPreparation::PublicationPending => {
            Ok(Some(WorthQueryHomeCorrectionOutcome::PublicationPending))
        }
        WorthQueryApplicationCorrectionPreparation::ProductUnpublished(handle) => Ok(Some(
            WorthQueryHomeCorrectionOutcome::ProductUnpublished(handle),
        )),
    }
}

fn finish_explicit_correction_preparation(
    prepared: WorthQueryApplicationCorrectionPreparation<'_>,
) -> Result<WorthQueryHomeCorrectionOutcome, WorthQueryHomeUndoDenial> {
    match prepared {
        WorthQueryApplicationCorrectionPreparation::Prepared(prepared) => Ok(
            map_conventional_outcome(prepared.publish().map_err(map_undo_denial)?),
        ),
        WorthQueryApplicationCorrectionPreparation::Settled(outcome) => {
            Ok(map_conventional_outcome(outcome))
        }
        WorthQueryApplicationCorrectionPreparation::PublicationPending => {
            Ok(WorthQueryHomeCorrectionOutcome::PublicationPending)
        }
        WorthQueryApplicationCorrectionPreparation::ProductUnpublished(handle) => {
            Ok(WorthQueryHomeCorrectionOutcome::ProductUnpublished(handle))
        }
    }
}

fn map_conventional_outcome(
    outcome: WorthQueryApplicationCorrectionOutcome,
) -> WorthQueryHomeCorrectionOutcome {
    match outcome {
        WorthQueryApplicationMutationOutcome::Committed { receipt, result } => {
            WorthQueryHomeCorrectionOutcome::Committed { receipt, result }
        }
        WorthQueryApplicationMutationOutcome::AlreadyCommitted(receipt) => {
            WorthQueryHomeCorrectionOutcome::AlreadyCommitted(receipt)
        }
        WorthQueryApplicationMutationOutcome::RecoveredCommitted(receipt) => {
            WorthQueryHomeCorrectionOutcome::RecoveredCommitted(receipt)
        }
        WorthQueryApplicationMutationOutcome::DomainDenied(value) => {
            WorthQueryHomeCorrectionOutcome::DomainDenied(value)
        }
        other => WorthQueryHomeCorrectionOutcome::Other(other),
    }
}

impl<Schema, Program> WorthQueryApplicationHome<Schema, Program>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema> + 'static,
{
    /// Reverses a recorded-inverse correction entry under the Home's active
    /// branch with a deterministic, restart-safe key derived from durable state.
    pub fn undo(
        &self,
        entry: &WorthQueryCorrectionEntry,
        principal: impl Borrow<WorthQueryAuthenticatedExternalPrincipal<Schema>>,
    ) -> Result<WorthQueryHomeCorrectionOutcome, WorthQueryHomeUndoDenial> {
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
            let key = derive_decimal_correction_key(seed, b"undo", probe);
            let prepared = request
                .corrections()
                .reverse(&source)
                .idempotency(&key)
                .limits(DEFAULT_CORRECTION_HISTORY_LIMITS)
                .prepare()
                .map_err(map_undo_denial)?;
            if let Some(outcome) = finish_auto_correction_preparation(prepared)? {
                return Ok(outcome);
            }
        }
        Err(WorthQueryHomeUndoDenial::AutoIdempotencyKeysExhausted)
    }

    /// Reverses a recorded-inverse correction entry with an explicit caller
    /// idempotency key (enabling exact idempotent replay across restarts).
    pub fn undo_with_key(
        &self,
        entry: &WorthQueryCorrectionEntry,
        principal: impl Borrow<WorthQueryAuthenticatedExternalPrincipal<Schema>>,
        idempotency_key: &str,
    ) -> Result<WorthQueryHomeCorrectionOutcome, WorthQueryHomeUndoDenial> {
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
            .prepare()
            .map_err(map_undo_denial)?;
        finish_explicit_correction_preparation(prepared)
    }

    /// Reapplies a previously reversed recorded-inverse entry with a
    /// deterministic, restart-safe key derived from durable state.
    pub fn redo(
        &self,
        entry: &WorthQueryCorrectionEntry,
        principal: impl Borrow<WorthQueryAuthenticatedExternalPrincipal<Schema>>,
    ) -> Result<WorthQueryHomeCorrectionOutcome, WorthQueryHomeUndoDenial> {
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
            let key = derive_decimal_correction_key(seed, b"redo", probe);
            let prepared = request
                .corrections()
                .reapply(&alternative)
                .idempotency(&key)
                .limits(DEFAULT_CORRECTION_HISTORY_LIMITS)
                .prepare()
                .map_err(map_undo_denial)?;
            if let Some(outcome) = finish_auto_correction_preparation(prepared)? {
                return Ok(outcome);
            }
        }
        Err(WorthQueryHomeUndoDenial::AutoIdempotencyKeysExhausted)
    }

    /// Reapplies a previously reversed recorded-inverse entry with an explicit key.
    pub fn redo_with_key(
        &self,
        entry: &WorthQueryCorrectionEntry,
        principal: impl Borrow<WorthQueryAuthenticatedExternalPrincipal<Schema>>,
        idempotency_key: &str,
    ) -> Result<WorthQueryHomeCorrectionOutcome, WorthQueryHomeUndoDenial> {
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
            .prepare()
            .map_err(map_undo_denial)?;
        finish_explicit_correction_preparation(prepared)
    }

    pub(super) fn select_redo_alternative<'home>(
        &'home self,
        request: &WorthQueryApplicationRequest<'home, '_, '_, Schema>,
        entry: &WorthQueryCorrectionEntry,
    ) -> Result<WorthQuerySelectedCorrectionAlternative<'home, Schema>, WorthQueryHomeUndoDenial>
    {
        let source = request
            .corrections()
            .select_coordinates(entry.coordinates(), DEFAULT_CORRECTION_HISTORY_LIMITS)
            .map_err(map_undo_denial)?;
        let page_size = NonZeroUsize::new(ALTERNATIVE_PAGE_SIZE).expect("nonzero page size");
        let mut page = request
            .corrections()
            .alternatives(&source, DEFAULT_CORRECTION_HISTORY_LIMITS, page_size)
            .map_err(map_undo_denial)?;
        let mut pages_read = 1usize;
        while let Some(continuation) = page.continuation() {
            if pages_read >= MAX_ALTERNATIVE_PAGES {
                return Err(WorthQueryHomeUndoDenial::NoAlternativeAvailable);
            }
            let next = request
                .corrections()
                .continue_alternatives(
                    &source,
                    DEFAULT_CORRECTION_HISTORY_LIMITS,
                    page_size,
                    continuation,
                )
                .map_err(map_undo_denial)?;
            page = next;
            pages_read += 1;
        }
        let detached = page.into_page();
        request
            .corrections()
            .select_alternative_coordinates(
                &detached,
                entry.coordinates(),
                DEFAULT_CORRECTION_HISTORY_LIMITS,
            )
            .map_err(map_undo_denial)
    }
}
