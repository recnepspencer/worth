//! Owned `'static` document history attachment and restart-safe Home undo/redo.
use std::borrow::Borrow;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_history::ApplicationHistoryBinding,
    application_program::ApplicationProgramDefinition,
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeBinding,
        ApplicationQueryScopeResolution,
    },
    application_schema::{ApplicationSchema, ApplicationStructuredValueBinding},
};
use worth_query_execution::facade::{
    application_contribution::WorthQueryAdvancementDenial,
    primary_graph::{
        WorthQueryApplicationCommitReceipt, WorthQueryApplicationProjection,
        WorthQueryCorrectionDerivationDenial, WorthQueryCorrectionSourceCoordinates,
        WorthQueryRecoveredApplicationCommitReceipt,
    },
};
use worth_query_publication::facade::application_entry::{
    WorthQueryApplicationCorrectionDenial, WorthQueryApplicationCorrectionOutcome,
    WorthQueryApplicationCorrectionPreparation, WorthQueryApplicationHistoryDenial,
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequest,
    WorthQueryApplicationRequestExt, WorthQueryCorrectionHistoryLimits, WorthQueryCorrectionValue,
    WorthQueryOwnedApplicationHistoryDocument, WorthQuerySelectedCorrectionAlternative,
};
use worth_runtime_world::facade::ProductUnpublishedRecoveryHandle;

use super::WorthQueryApplicationHome;

mod authored;
pub use authored::WorthQueryHomeAuthoredCorrectionOutcome;

type HistoryAccessBinding<S, H> =
    <<H as ApplicationHistoryBinding<S>>::Access as ApplicationQueryIntent<S>>::Binding;
type HistoryQuery<S, H> = <HistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query;
type HistoryScope<S, H> = <<HistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope;
type HistoryPrincipal<S, H> =
    <HistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::PrincipalIdentity;
type HistoryRows<S, H> = <<HistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

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

/// Why attaching an owned `'static` history document was refused.
#[derive(Debug)]
pub enum WorthQueryHomeOwnedHistoryDenial {
    Advancement(WorthQueryAdvancementDenial),
    Fork(super::preview::WorthQueryPreviewForkDenial),
    Attachment(WorthQueryApplicationHistoryDenial),
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

pub(super) fn next_restart_safe_correction_key() -> String {
    static SEQUENCE: AtomicU64 = AtomicU64::new(1);
    let micros = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_micros() as u64);
    let seq = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let combined = ((micros & 0x0000_7fff_ffff_ffff) << 16) | (seq & 0xffff);
    combined.to_string()
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
    /// Forks a dedicated head from the Home's active branch and attaches an
    /// owned `'static` [`WorthQueryOwnedApplicationHistoryDocument`].
    ///
    /// Because the returned document is `'static`, applications can store it
    /// directly in desktop UI state alongside the Home without lifetime borrows.
    /// Close the document before closing the Home.
    pub fn owned_history<H>(
        &self,
        principal: impl Borrow<WorthQueryAuthenticatedExternalPrincipal<Schema>>,
        access: H::Access,
    ) -> Result<
        WorthQueryOwnedApplicationHistoryDocument<Schema, H>,
        WorthQueryHomeOwnedHistoryDenial,
    >
    where
        H: ApplicationHistoryBinding<Schema>,
        <HistoryAccessBinding<Schema, H> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<Schema, HistoryPrincipal<Schema, H>>,
        HistoryRows<Schema, H>: WorthQueryApplicationProjection<Schema, HistoryQuery<Schema, H>>,
        HistoryScope<Schema, H>: 'static,
    {
        let principal = principal.borrow();
        let expected = super::preview::fork_expected_from_active(self, principal.clock())
            .map_err(WorthQueryHomeOwnedHistoryDenial::Fork)?;
        let head = expected.into_owned_head();
        let scope = WorthQueryRequestScope::new(
            None,
            WorthQueryCancellationSource::new().token(),
            principal.clock().clone(),
        );
        let graph = self.primary_graph();
        let mut head_slot = Some(head);
        let attached = graph.with_application_advancement(&scope, |phase| {
            let head = head_slot.take().expect("advancement runs once");
            let request = graph.request(principal, &scope).on_branch(head.branch());
            match request.attach_owned_history::<H, Program>(self.runtime(), &phase, head, access) {
                Ok(document) => Ok(document),
                Err(failure) => {
                    let (head, denial) = failure.into_parts();
                    let _ = head.close(&phase);
                    Err(WorthQueryHomeOwnedHistoryDenial::Attachment(denial))
                }
            }
        });
        match attached {
            Ok(result) => result,
            Err(denial) => {
                if let Some(head) = head_slot.take() {
                    let _ = graph.with_application_advancement(&scope, |phase| {
                        let _ = head.close(&phase);
                    });
                }
                Err(WorthQueryHomeOwnedHistoryDenial::Advancement(denial))
            }
        }
    }

    /// Reverses a recorded-inverse correction entry under the Home's active
    /// branch with a fresh restart-safe idempotency key.
    pub fn undo(
        &self,
        entry: &WorthQueryCorrectionEntry,
        principal: impl Borrow<WorthQueryAuthenticatedExternalPrincipal<Schema>>,
    ) -> Result<WorthQueryHomeCorrectionOutcome, WorthQueryHomeUndoDenial> {
        let key = next_restart_safe_correction_key();
        self.undo_with_key(entry, principal, &key)
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

    /// Reapplies a previously reversed recorded-inverse entry with a fresh key.
    pub fn redo(
        &self,
        entry: &WorthQueryCorrectionEntry,
        principal: impl Borrow<WorthQueryAuthenticatedExternalPrincipal<Schema>>,
    ) -> Result<WorthQueryHomeCorrectionOutcome, WorthQueryHomeUndoDenial> {
        let key = next_restart_safe_correction_key();
        self.redo_with_key(entry, principal, &key)
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
