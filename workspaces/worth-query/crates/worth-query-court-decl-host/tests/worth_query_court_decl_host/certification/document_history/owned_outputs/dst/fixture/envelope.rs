//! The calls that run stages: an edit envelope on the document, stopped at a
//! request consult when asked; the same calls written on the document branch
//! outside the document; and a correction through today's route
//! (`request.corrections()`), which also writes outside the document.
use super::program::Shape;
use super::reads::{Forest, Host};
use super::*;
use std::num::NonZeroUsize;
use worth_execution::with_request_consult_observer_for_test;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAdmissionClock, WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::product::WorthQueryProductBranch;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

/// Every DST binding has the same result and denial, so one token type.
pub type Token = WorthQueryEditStageToken<
    <set_e1::Binding as ApplicationMutationBinding<Schema>>::Result,
    <set_e1::Binding as ApplicationMutationBinding<Schema>>::Denial,
>;

/// What one stage committed, kept for a later correction of it.
pub type Receipt = WorthQueryApplicationCommitReceipt;

/// One stage an envelope pushes: a DST call, or a stage built to meet an
/// environment refusal on the owned world's model bindings.
#[derive(Clone, Copy, Debug)]
pub enum Push {
    Dst(Call),
    /// `Edit model` on the document's model: enrolled and owned.
    OwnedModel,
    /// `Edit model` on model `b`, which the document does not own.
    UnownedModel,
    /// A model binding that is installed but not enrolled in the history.
    UndeclaredModel,
}

/// The end of one envelope on the document.
pub enum Executed<'h> {
    /// The step, and the commit receipt of each entity's stage.
    Recorded {
        document: Forest<'h>,
        step: WorthQueryHistoryStep,
        receipts: Vec<(usize, Receipt)>,
    },
    AlreadyRecorded {
        document: Forest<'h>,
        step: WorthQueryHistoryStep,
    },
    /// A correction whose source refused to select, before any envelope
    /// opened: the document is untouched.
    Unselectable {
        document: Forest<'h>,
        denial: String,
    },
    Refused {
        document: Forest<'h>,
        denial: WorthQueryHistoryEditEnvelopeDenial,
    },
}

/// Push `pushes` in order on the document's envelope under `key`, then
/// execute it with `metadata`. With `stop`, the request stops at that
/// consult of the execution.
pub fn envelope<'h>(
    host: &'h Host,
    document: Forest<'h>,
    branch: WorthQueryProductBranch,
    key: String,
    pushes: &[Push],
    metadata: Vec<u8>,
    stop: Option<u32>,
) -> Executed<'h> {
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, cancellation.token(), WorthQueryAdmissionClock::system());
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope).on_branch(branch);
    let envelope = match document.edit_envelope(host, &request, key) {
        Ok(envelope) => envelope,
        Err(failure) => {
            let (document, denial) = failure.into_parts();
            return Executed::Refused { document, denial };
        }
    };
    // The model's stages run in their written order. Declared order refuses
    // two creations of one entity type as a cycle (traced, seed 0x7b020001
    // k3: create-e1 and create-e2 both read and create `Member`).
    let mut envelope = envelope
        .ordered_authored()
        .unwrap_or_else(|(_, denial)| panic!("the envelope must take authored order: {denial:?}"));
    let mut tokens: Vec<(usize, Token)> = Vec::new();
    for push in pushes {
        let pushed = match *push {
            Push::Dst(call) => dispatch!(call, intent => envelope.push_stage::<ApplicationNoOutputGraph, _>(intent))
                .map(|token| tokens.push((call.entity(), token))),
            Push::OwnedModel | Push::UnownedModel => {
                let key = if matches!(push, Push::OwnedModel) { "a" } else { "b" };
                let intent = edit::Intent { key: key.into(), value: 1 };
                envelope.push_stage::<ApplicationNoOutputGraph, _>(intent).map(drop)
            }
            Push::UndeclaredModel => {
                let intent = synchronous_edit::envelope_fixture::model::Intent {
                    key: "a".into(),
                    value: 1,
                    key_derived_target: false,
                };
                envelope.push_stage::<ApplicationNoOutputGraph, _>(intent).map(drop)
            }
        };
        pushed.unwrap_or_else(|denial| panic!("a stage must push: {push:?}: {denial:?}"));
    }
    let metadata = WorthQueryHistoryStepMetadata::from_bytes(metadata);
    let execute = move || envelope.execute_recorded(metadata, ExecutionAllocationPolicy::SystemAllocation);
    let execution = match stop {
        None => execute(),
        Some(stop) => {
            let stop = stop as usize;
            with_request_consult_observer_for_test(
                move |consult| {
                    if consult == stop {
                        cancellation.cancel();
                    }
                },
                execute,
            )
            .0
        }
    };
    match execution {
        Ok(WorthQueryHistoryEditEnvelopeOutcome::Recorded { document, step, receipt }) => {
            let receipts = tokens
                .iter()
                .map(|(entity, token)| {
                    let stage = receipt
                        .stage_receipt(token)
                        .unwrap_or_else(|denial| panic!("a recorded stage has a receipt: {denial:?}"));
                    (*entity, stage.clone())
                })
                .collect();
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

/// Run `calls` on `branch` outside the document through the ordinary
/// runtime envelope, then settle the program's demand there, so the branch's
/// outputs follow its rows as the model's `OutOfBandMutation` says.
pub fn write_outside(host: &Host, shape: Shape, branch: WorthQueryProductBranch, key: String, calls: &[Call]) {
    super::reads::with_branch_request(host, branch, |request| {
        let mut envelope = request
            .edit_envelope()
            .idempotency(key.clone())
            .runtime_stages(host)
            .ordered_authored()
            .unwrap_or_else(|denial| panic!("{key}: the write outside must open: {denial:?}"));
        for call in calls {
            dispatch!(*call, intent => envelope.push_stage::<ApplicationNoOutputGraph, _>(intent))
                .unwrap_or_else(|denial| panic!("{key}: a stage must push: {call:?}: {denial:?}"));
        }
        let outcome = envelope
            .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap_or_else(|denial| panic!("{key}: the write outside must run: {denial:?}"));
        assert!(matches!(outcome, WorthQueryEditEnvelopeOutcome::Committed(_)), "{key}: {outcome:?}");
        settle(host, request, shape);
    });
}

/// Reverse one recorded stage through today's correction route on the
/// document branch: it commits there, outside the document, or its inverse
/// binding refuses and nothing commits (`false`). Traced (seed 0x7b020000,
/// branch-after-delete, k2): the route does not co-retire, so `despawn-e1`
/// refuses `OwnedLive` while `e2` is live.
pub fn correct_outside(
    host: &Host,
    shape: Shape,
    branch: WorthQueryProductBranch,
    key: String,
    source: &Receipt,
    undo: Undo,
) -> bool {
    super::reads::with_branch_request(host, branch, |request| {
        let limits = WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::MIN);
        let selected = request
            .corrections()
            .select(source, limits)
            .unwrap_or_else(|denial| panic!("{key}: the source must select: {denial:?}"));
        let reverse = request.corrections().reverse(&selected).idempotency(key.as_str()).limits(limits);
        let envelope = match undo {
            Undo::Create(E1) => reverse.edit_envelope::<UndoCreateE1, program::Program, ApplicationNoOutputGraph>(host),
            Undo::Create(_) => reverse.edit_envelope::<UndoCreateE2, program::Program, ApplicationNoOutputGraph>(host),
            Undo::Retire(E1) => reverse.edit_envelope::<UndoRetireE1, program::Program, ApplicationNoOutputGraph>(host),
            Undo::Retire(_) => reverse.edit_envelope::<UndoRetireE2, program::Program, ApplicationNoOutputGraph>(host),
        };
        let outcome = envelope
            .unwrap_or_else(|denial| panic!("{key}: the correction must prepare: {denial:?}"))
            .ordered_authored()
            .unwrap_or_else(|denial| panic!("{key}: the correction must order: {denial:?}"))
            .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap_or_else(|denial| panic!("{key}: the correction must run: {denial:?}"));
        match outcome {
            WorthQueryEditEnvelopeOutcome::Committed(_) => {
                settle(host, request, shape);
                true
            }
            WorthQueryEditEnvelopeOutcome::DomainDenied(_) => false,
            other => panic!("{key}: {other:?}"),
        }
    })
}

/// Which correction reverses a source stage.
#[derive(Clone, Copy, Debug)]
pub enum Undo {
    Create(usize),
    Retire(usize),
}

/// One settle loop per tree type: the demanded output types differ.
macro_rules! settle_tree {
    ($host:ident, $request:ident, $program:ty, $tree:ty) => {{
        let mut outputs = $request
            .demand(upstream::Demand("a".into()))
            .retain_program_outputs::<$program, $tree>($host)
            .unwrap_or_else(|cause| panic!("the tree must demand: {:?}", cause.denial()));
        for _ in 0..512 {
            outputs = match outputs.advance($request) {
                WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
                WorthQueryProgramOutputOperationAdvance::Settled(done) => {
                    done.deliver_current($request).unwrap();
                    done.close($request).unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
                    return;
                }
                WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => panic!("{denial:?}"),
                WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => panic!("{:?}", terminal.denial()),
            };
        }
        panic!("the tree must settle within 512 advances")
    }};
}

/// Demand the `shape` tree on the request's branch until it settles.
pub fn settle(host: &Host, request: &family::Request<'_, '_, '_>, shape: Shape) {
    match shape {
        Shape::Members => settle_tree!(host, request, program::Program, program::Tree),
        Shape::WithConsumer => settle_tree!(host, request, program::Program, program::ConsumerTree),
    }
}
