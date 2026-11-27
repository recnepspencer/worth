//! The recording driver's admission half (plan r3 §3.3; §1.2 rows 1, 2 and
//! 8). Every kind binds `DocumentRequestContext` and reads its metadata and
//! reserves through it; an edit and a group open their phase through it, and
//! an envelope admits inside its executor's phase. A resumable kind forks its
//! private candidate through `fork_candidate`. The handoff to the sealing half
//! is the kind's reserved record and its candidate, as the document history
//! issued them.
use super::edit::{ForkRecovery, WorthQueryHistoryEditDenial};
use super::*;
use worth_query_execution::{
    facade::product::WorthQueryProductBranchCreateError,
    publication_boundary::document_history::WorthQueryDocumentHistoryCandidateDenial,
};

/// How a resumable kind reports a fork refusal: a history refusal of the
/// document, or a member refusal (a parked creation, a refused candidate).
pub(super) trait ForkDenial {
    fn history(denial: WorthQueryApplicationHistoryDenial) -> Self;
    fn member(denial: WorthQueryHistoryEditDenial) -> Self;
}

impl ForkDenial for WorthQueryHistoryEditDenial {
    fn history(denial: WorthQueryApplicationHistoryDenial) -> Self {
        Self::History(denial)
    }

    fn member(denial: WorthQueryHistoryEditDenial) -> Self {
        denial
    }
}

impl ForkDenial for WorthQueryHistoryGroupDenial {
    fn history(denial: WorthQueryApplicationHistoryDenial) -> Self {
        Self::History(denial)
    }

    fn member(denial: WorthQueryHistoryEditDenial) -> Self {
        Self::Member(denial)
    }
}

/// Fork a resumable kind's private candidate in a phase of its own: release
/// a parked creation, then read the document, then `fork`. An envelope forks
/// inside its one phase and parks nothing (§1.2, lifetimes).
pub(super) fn fork_candidate<S, H, Payload, Outputs, T, E>(
    document: &WorthQueryApplicationHistoryDocument<'_, S, H, Payload, Outputs>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    recovery: &mut Option<ForkRecovery>,
    fork: impl FnOnce(&WorthQueryAdvancementPhase<'_>, &mut Option<ForkRecovery>) -> Result<T, E>,
) -> Result<T, E>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    E: ForkDenial,
{
    ForkRecovery::release_in_request(recovery, request).map_err(E::member)?;
    document
        .owner
        .application()
        .with_application_advancement(request.scope, |phase| {
            document
                .read_in_advancement(&phase, request)
                .map_err(E::history)?;
            fork(&phase, recovery)
        })
        .map_err(|cause| E::history(WorthQueryApplicationHistoryDenial::ExecutionRequest(cause)))?
}

impl ForkRecovery {
    /// A refused fork. A product not yet published parks its creation for
    /// the next request to release.
    pub(super) fn creation_refused(
        recovery: &mut Option<Self>,
        cause: WorthQueryDocumentHistoryCandidateDenial,
    ) -> WorthQueryHistoryEditDenial {
        match cause {
            WorthQueryDocumentHistoryCandidateDenial::Creation(
                WorthQueryProductBranchCreateError::ProductUnpublished(owner),
            ) => {
                *recovery = Some(Self::Creation(owner));
                WorthQueryHistoryEditDenial::ForkRecovery
            }
            cause => WorthQueryHistoryEditDenial::Candidate(cause),
        }
    }
}
