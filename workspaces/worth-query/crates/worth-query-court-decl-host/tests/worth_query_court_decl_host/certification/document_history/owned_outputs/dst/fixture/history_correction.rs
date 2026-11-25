//! A correction recorded by the document as one step
//! (`document.correction_envelope`, plan §2 and §10.1): the corrective stage
//! first, then the caller's stages, on the document's private candidate.
use super::envelope::{Executed, Receipt, Undo};
use super::program;
use super::reads::{Forest, Host};
use super::*;
use std::num::NonZeroUsize;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAdmissionClock, WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::product::WorthQueryProductBranch;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

/// Reverse `undo` of `source` on `document` under `key`, with `coupled` after
/// the corrective stage, recorded as one step.
pub fn correction_envelope<'h>(
    host: &'h Host,
    document: Forest<'h>,
    branch: WorthQueryProductBranch,
    key: String,
    source: &Receipt,
    undo: Undo,
    coupled: &[Call],
) -> Executed<'h> {
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, cancellation.token(), WorthQueryAdmissionClock::system());
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope).on_branch(branch);
    let limits = WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::MIN);
    let selected = request
        .corrections()
        .select(source, limits)
        .unwrap_or_else(|denial| panic!("{key}: the source must select: {denial:?}"));
    let reverse = request.corrections().reverse(&selected).idempotency(key.as_str()).limits(limits);
    macro_rules! run {
        ($correction:ty) => {{
            let opened = document
                .correction_envelope::<$correction, program::Program, ApplicationNoOutputGraph>(host, reverse);
            let envelope = match opened {
                Ok(envelope) => envelope,
                Err(failure) => {
                    let (document, denial) = failure.into_parts();
                    return Executed::Refused { document, denial };
                }
            };
            let mut envelope = envelope
                .ordered_authored()
                .unwrap_or_else(|(_, denial)| panic!("{key}: the correction must take authored order: {denial:?}"));
            for call in coupled {
                dispatch!(*call, intent => envelope.push_stage::<ApplicationNoOutputGraph, _>(intent))
                    .unwrap_or_else(|denial| panic!("{key}: a coupled stage must push: {call:?}: {denial:?}"));
            }
            envelope.execute_recorded(WorthQueryHistoryStepMetadata::none(), ExecutionAllocationPolicy::SystemAllocation)
        }};
    }
    let executed = match undo {
        Undo::Create(E1) => run!(UndoCreateE1),
        Undo::Create(_) => run!(UndoCreateE2),
        Undo::Retire(E1) => run!(UndoRetireE1),
        Undo::Retire(_) => run!(UndoRetireE2),
    };
    match executed {
        Ok(WorthQueryHistoryEditEnvelopeOutcome::Recorded { document, step, .. }) => {
            Executed::Recorded { document, step, receipts: Vec::new() }
        }
        Ok(WorthQueryHistoryEditEnvelopeOutcome::AlreadyRecorded { document, step }) => {
            Executed::AlreadyRecorded { document, step }
        }
        Err(failure) => {
            let (document, denial) = failure.into_parts();
            Executed::Refused { document, denial }
        }
    }
}
