//! One refusal end for every recorded kind (plan r3 §3.3 step 5): one close
//! of the private candidate, then a park or a kept retry, then the cancel.
use super::super::{
    head_custody::HistoryHeadCustody, journal::ReservedHistoryRecord, selection::HistorySelection,
    WorthQueryDocumentHistory, WorthQueryDocumentHistoryDenial as Denial,
    WorthQueryOwnedProductHead,
};
use super::RecordingKind;
use crate::{
    basis::WorthQueryProductBranch,
    domain_computation::primary_graph::{
        WorthQueryAdvancementPhase, WorthQueryApplicationProductBranchCleanupFailure,
        WorthQueryApplicationProductBranchCloseDenial, WorthQueryExpectedProductBranch,
        WorthQueryOwnedProductHeadCloseFailure,
    },
};
use worth_query_declaration::facade::{
    application_history::ApplicationHistoryBinding, application_schema::ApplicationSchema,
};

/// What a refused recording does with a candidate that did not close.
pub(in super::super) enum CandidateDisposition {
    /// The record and the candidate go back to the caller, which retries.
    KeepForRetry,
    /// The candidate moves into the record's prepaid head slot, where retention
    /// maintenance and final disposal close it; the record is released.
    Park,
}

/// A refused recording's private candidate, held until it closes.
#[must_use = "close the refused candidate and refuse its record through the document history"]
pub struct WorthQueryRefusedCandidate<'runtime, S>(Candidate<'runtime, S>);

enum Candidate<'runtime, S> {
    Live {
        head: WorthQueryOwnedProductHead<'runtime, S>,
        denial: Option<WorthQueryApplicationProductBranchCloseDenial>,
    },
    Retired {
        branch: WorthQueryProductBranch,
        failure: WorthQueryApplicationProductBranchCleanupFailure,
    },
    Closed,
}

impl<'runtime, S: ApplicationSchema> WorthQueryRefusedCandidate<'runtime, S> {
    pub fn new(candidate: WorthQueryExpectedProductBranch<'runtime, S>) -> Self {
        Self(Candidate::Live {
            head: candidate.into_owned_head(),
            denial: None,
        })
    }

    /// A live head closes only under an advancement phase. Retired cleanup
    /// retries under its own request, so it needs none.
    pub fn needs_phase(&self) -> bool {
        matches!(self.0, Candidate::Live { .. })
    }

    pub fn is_closed(&self) -> bool {
        matches!(self.0, Candidate::Closed)
    }

    /// One close attempt. Without a phase a live head stays live.
    pub fn close(self, phase: Option<&WorthQueryAdvancementPhase<'_>>) -> Self {
        Self(match (self.0, phase) {
            (Candidate::Live { head, .. }, Some(phase)) => match head.close(phase) {
                Ok(_) => Candidate::Closed,
                Err(WorthQueryOwnedProductHeadCloseFailure::Retained { head, denial }) => {
                    Candidate::Live {
                        head,
                        denial: Some(denial),
                    }
                }
                Err(WorthQueryOwnedProductHeadCloseFailure::Retired { branch, failure }) => {
                    Candidate::Retired { branch, failure }
                }
            },
            (Candidate::Retired { branch, failure }, _) => match failure.into_cleanup().retry() {
                Ok(_) => Candidate::Closed,
                Err(failure) => Candidate::Retired { branch, failure },
            },
            (unchanged, _) => unchanged,
        })
    }

    pub fn close_denial(&self) -> Option<&WorthQueryApplicationProductBranchCloseDenial> {
        match &self.0 {
            Candidate::Live { denial, .. } => denial.as_ref(),
            _ => None,
        }
    }

    pub fn cleanup_failure(&self) -> Option<&WorthQueryApplicationProductBranchCleanupFailure> {
        match &self.0 {
            Candidate::Retired { failure, .. } => Some(failure),
            _ => None,
        }
    }

    fn into_custody(self) -> HistoryHeadCustody {
        match self.0 {
            Candidate::Live { head, denial: None } => {
                HistoryHeadCustody::Live(head.into_detached())
            }
            Candidate::Live {
                head,
                denial: Some(denial),
            } => HistoryHeadCustody::Refused {
                head: head.into_detached(),
                denial,
            },
            Candidate::Retired { branch, failure } => {
                HistoryHeadCustody::Retired { branch, failure }
            }
            Candidate::Closed => unreachable!("a closed candidate is never parked"),
        }
    }
}

/// The reserved record a refusal releases, and the check that it is still
/// this document's.
pub(in super::super) trait RefusedRecord {
    fn check(&self, selection: &HistorySelection) -> Result<(), Denial>;
    fn into_record(self) -> ReservedHistoryRecord;
}

/// A refusal that kept its record: the record is not this document's (the
/// denial), or the candidate did not close and the kind keeps it for a retry
/// (no denial).
pub type WorthQueryKeptRefusal<'runtime, S, R> = (
    R,
    Option<WorthQueryRefusedCandidate<'runtime, S>>,
    Option<Denial>,
);

impl<'runtime, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryDocumentHistory<'runtime, S, H>
{
    /// Release `reserved` after its candidate's close attempt, by the kind's
    /// disposition. A closed or absent candidate cancels the record.
    pub(in super::super) fn refuse<R: RefusedRecord>(
        &self,
        kind: RecordingKind,
        reserved: R,
        candidate: Option<WorthQueryRefusedCandidate<'runtime, S>>,
    ) -> Result<(), WorthQueryKeptRefusal<'runtime, S, R>> {
        let unclosed = candidate.filter(|candidate| !candidate.is_closed());
        let mut selected = self.selection.lock();
        if let Err(denial) = reserved.check(&selected) {
            return Err((reserved, unclosed, Some(denial)));
        }
        match (unclosed, kind.on_refusal()) {
            (None, _) => selected.journal.cancel(reserved.into_record()),
            (Some(candidate), CandidateDisposition::Park) => selected
                .journal
                .cancel_parking(reserved.into_record(), candidate.into_custody()),
            (Some(candidate), CandidateDisposition::KeepForRetry) => {
                return Err((reserved, Some(candidate), None))
            }
        }
        Ok(())
    }
}
