//! The recording driver (plan r3 §3.3; §1.2 rows 1, 2 and 8).
//!
//! Admission: every kind binds `DocumentRequestContext` and reads its metadata
//! and reserves through it; an edit and a group open their phase through it,
//! and an envelope admits inside its executor's phase. A resumable kind forks
//! its private candidate through `fork_candidate`.
//!
//! Sealing: in one phase, `seal` records the kind's output forest on the
//! candidate, adopts the record, and checks its delivery. Each kind keeps its
//! own phase loop: an edit seals one phase per request, a group in the phase
//! that completes it, an envelope over as many phases as its producers need
//! within its one request.
use super::edit::{ForkRecovery, WorthQueryHistoryEditDenial};
use super::outputs::{OutputDriver, RecordingForest};
use super::*;
use worth_query_execution::{
    facade::product::{WorthQueryProductBranch, WorthQueryProductBranchCreateError},
    publication_boundary::document_history::{
        WorthQueryCandidateDisposition, WorthQueryCompletedRecordingOutputs,
        WorthQueryDocumentHistoryCandidateDenial, WorthQueryRecordingKind,
    },
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

/// Seal one kind's record in `phase`, in order: `record` advances the bound
/// output trees on the candidate (false while a producer is still working:
/// the phase ends without adoption); `adopt` adopts the record, from the
/// completed forest when trees are bound, and a refused adoption gives the
/// record and the candidate back to `state`; `deliver` runs the kind's
/// handoff pause and delivery check after adoption. `Ok(None)` means the
/// forest is still recording. An adopted record drops its forest. A refusal
/// before adoption keeps the forest when the kind keeps its candidate for a
/// retry; when the kind parks its candidate, every tree is released first,
/// because the candidate closes next. A forest that cannot release is kept
/// for a later release.
pub(super) fn seal<S, H, Outputs, State, T, E>(
    outputs: &Outputs,
    phase: &WorthQueryAdvancementPhase<'_>,
    candidate: WorthQueryProductBranch,
    kind: WorthQueryRecordingKind,
    state: &mut State,
    record: impl FnOnce(&mut State, &mut RecordingForest<S>) -> Result<bool, E>,
    adopt: impl FnOnce(&mut State, Option<&WorthQueryCompletedRecordingOutputs>) -> Result<T, E>,
    deliver: impl FnOnce(&mut State) -> Result<(), E>,
) -> Result<Result<Option<T>, E>, WorthQueryHistoryOutputDenial>
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    Outputs: OutputDriver<S, H>,
{
    outputs.with_recording(phase, candidate, |mut recording| {
        let mut adopted = false;
        let sealed = (|| {
            if let Some(forest) = recording.as_deref_mut()
                && !record(state, forest)?
            {
                return Ok(None);
            }
            let step = adopt(
                state,
                recording.as_deref().and_then(RecordingForest::completed),
            )?;
            adopted = true;
            deliver(state)?;
            Ok(Some(step))
        })();
        match sealed {
            Err(denial)
                if !adopted && kind.on_refusal() == WorthQueryCandidateDisposition::Park =>
            {
                let released =
                    recording.is_none_or(|forest| release(forest, phase, candidate).is_ok());
                (Err(denial), released)
            }
            sealed => (sealed, adopted),
        }
    })
}

/// Close every tree the forest started, on the candidate it was bound to.
fn release<S: ApplicationSchema + 'static>(
    forest: &mut RecordingForest<S>,
    phase: &WorthQueryAdvancementPhase<'_>,
    current: WorthQueryProductBranch,
) -> Result<(), WorthQueryHistoryOutputDenial> {
    let branch = forest.origin().map_or(current, |origin| origin.candidate());
    forest.close(phase, branch)
}
