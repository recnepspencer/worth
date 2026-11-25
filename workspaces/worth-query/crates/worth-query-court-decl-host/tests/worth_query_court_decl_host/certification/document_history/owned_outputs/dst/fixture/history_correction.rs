//! A correction recorded by the document as one step
//! (`document.correction_envelope`, plan §2 and §10.1): the corrective stage
//! first, then the caller's stages, on the document's private candidate.
use super::envelope::{Executed, Receipt, Token, Undo};
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
/// the corrective stage, recorded as one step. A recorded step's receipts are
/// the corrective stage's (the envelope's primary member, under `undo`'s
/// entity) first, then each coupled stage's, so later corrections can name
/// any of them.
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
    let selected = match request.corrections().select(source, limits) {
        Ok(selected) => selected,
        Err(denial) => return Executed::Unselectable { document, denial: format!("{denial:?}") },
    };
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
            let mut tokens: Vec<(usize, Token)> = Vec::new();
            for call in coupled {
                let token = dispatch!(*call, intent => envelope.push_stage::<ApplicationNoOutputGraph, _>(intent))
                    .unwrap_or_else(|denial| panic!("{key}: a coupled stage must push: {call:?}: {denial:?}"));
                tokens.push((call.entity(), token));
            }
            (envelope.execute_recorded(WorthQueryHistoryStepMetadata::none(), ExecutionAllocationPolicy::SystemAllocation), tokens)
        }};
    }
    let (executed, tokens) = match undo {
        Undo::Create(E1) => run!(UndoCreateE1),
        Undo::Create(_) => run!(UndoCreateE2),
        Undo::Retire(E1) => run!(UndoRetireE1),
        Undo::Retire(_) => run!(UndoRetireE2),
    };
    let corrected = match undo {
        Undo::Create(entity) | Undo::Retire(entity) => entity,
    };
    match executed {
        Ok(WorthQueryHistoryEditEnvelopeOutcome::Recorded { document, step, receipt }) => {
            let mut receipts = vec![(corrected, receipt.receipt().clone())];
            for (entity, token) in &tokens {
                let stage = receipt
                    .stage_receipt(token)
                    .unwrap_or_else(|denial| panic!("{key}: a recorded stage has a receipt: {denial:?}"));
                receipts.push((*entity, stage.clone()));
            }
            Executed::Recorded { document, step, receipts }
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

/// A scope-record history document on a fresh fork of the host's world.
pub type ScopeDocument<'h> = WorthQueryApplicationHistoryDocument<'h, Schema, ScopeHistory>;

/// Fork a fresh owned head and attach the scope-record history to it.
pub fn attach_scope_history(host: &Host) -> ScopeDocument<'_> {
    let scope = WorthQueryRequestScope::new(None, WorthQueryCancellationSource::new().token(), WorthQueryAdmissionClock::system());
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.with_host_advancement(|phase| {
        let selected = host.on_branch(host.current_world()).select(&phase).unwrap();
        let head = host
            .branches()
            .fork_expected(selected)
            .unwrap()
            .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
            .create(&phase)
            .unwrap()
            .into_owned_head();
        host.request(&principal, &scope)
            .on_branch(head.branch())
            .attach_history::<ScopeHistory>(&phase, head, query::model::Read("a".into()))
            .unwrap_or_else(|failure| panic!("the scope history must attach: {:?}", failure.denial()))
    })
    .unwrap()
}

/// Open a correction envelope of `source` (selected on `branch`) on a
/// scope-record history document: the refusal's text, with the document
/// returned, or `Ok` if it opened (the envelope is cancelled).
pub fn open_on_scope_history<'h>(
    host: &'h Host,
    document: ScopeDocument<'h>,
    branch: WorthQueryProductBranch,
    key: String,
    source: &Receipt,
) -> (ScopeDocument<'h>, Result<(), String>) {
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
    match document.correction_envelope::<UndoCreateE1, program::Program, ApplicationNoOutputGraph>(host, reverse) {
        Ok(envelope) => (envelope.cancel(), Ok(())),
        Err(failure) => {
            let (document, denial) = failure.into_parts();
            (document, Err(format!("{denial:?}")))
        }
    }
}
