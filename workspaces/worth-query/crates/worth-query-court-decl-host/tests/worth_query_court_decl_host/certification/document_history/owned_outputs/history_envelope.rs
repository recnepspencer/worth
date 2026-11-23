//! A multi-stage edit envelope recorded as one step of the owned-model history.
use super::*;
use std::num::NonZeroUsize;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_query_host::facade::application_entry::WorthQueryApplicationHistoryDenial;
use worth_query_host::facade::product::WorthQueryProductBranch;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use WorthQueryHistoryEditEnvelopeDenial as Denial;
use WorthQueryHistoryEditEnvelopeOutcome as Outcome;

mod metadata;
pub(crate) mod output_forest;
mod recording;
mod replay;
mod rollback;

/// The model history document under either outputs mode.
type Document<'a, O> =
    WorthQueryApplicationHistoryDocument<'a, Schema, History, OrdinaryHistoryPayload, O>;
type Execution<'a, O = WorthQueryHistoryNoOutputs> = Result<
    WorthQueryHistoryEditEnvelopeOutcome<'a, Schema, History, OrdinaryHistoryPayload, O>,
    WorthQueryHistoryEditEnvelopeFailure<'a, Schema, History, OrdinaryHistoryPayload, O>,
>;

/// The model edit (stage 0) and the member edit (stage 1) under one caller key.
#[derive(Clone, Copy)]
struct Stages {
    key: &'static str,
    model: u64,
    member: u64,
}

fn limits() -> WorthQueryHistoryPageLimits {
    WorthQueryHistoryPageLimits {
        maximum_entries: NonZeroUsize::new(8).unwrap(),
        maximum_examined_entries: NonZeroUsize::new(8).unwrap(),
        maximum_owned_bytes: NonZeroUsize::new(16 * 1024).unwrap(),
    }
}

fn metadata(bytes: &[u8]) -> WorthQueryHistoryStepMetadata {
    WorthQueryHistoryStepMetadata::from_bytes(bytes.to_vec())
}

/// Author both stages on the document and execute them as one recorded step.
fn record<'d, O: WorthQueryHistoryOutputsMode<Schema, History>>(
    host: &world::Host,
    document: Document<'d, O>,
    scope: &WorthQueryRequestScope,
    stages: Stages,
    metadata: WorthQueryHistoryStepMetadata,
) -> Execution<'d, O> {
    let principal = authenticate_operator(host.installed_schema(), scope);
    let branch = document
        .selection(&host.request(&principal, scope))
        .unwrap()
        .physical_endpoint()
        .product_branch();
    record_on(host, document, scope, branch, stages, metadata)
}

/// `record` for a document whose branch the caller already read.
fn record_on<'d, O: WorthQueryHistoryOutputsMode<Schema, History>>(
    host: &world::Host,
    document: Document<'d, O>,
    scope: &WorthQueryRequestScope,
    branch: WorthQueryProductBranch,
    stages: Stages,
    metadata: WorthQueryHistoryStepMetadata,
) -> Execution<'d, O> {
    record_within(host, document, scope, branch, stages, metadata, |execute| {
        execute()
    })
}

/// `record_on` whose execution, after both stages are authored, runs inside `within`.
fn record_within<'d, O: WorthQueryHistoryOutputsMode<Schema, History>>(
    host: &world::Host,
    document: Document<'d, O>,
    scope: &WorthQueryRequestScope,
    branch: WorthQueryProductBranch,
    stages: Stages,
    metadata: WorthQueryHistoryStepMetadata,
    within: impl FnOnce(Box<dyn FnOnce() -> Execution<'d, O> + '_>) -> Execution<'d, O>,
) -> Execution<'d, O> {
    let principal = authenticate_operator(host.installed_schema(), scope);
    let request = host.request(&principal, scope).on_branch(branch);
    let mut envelope = document.edit_envelope(host, &request, stages.key.to_owned())?;
    envelope
        .push_stage::<ApplicationNoOutputGraph, _>(edit::Intent {
            key: "a".into(),
            value: stages.model,
        })
        .unwrap();
    envelope
        .push_stage::<ApplicationNoOutputGraph, _>(member_edit::Intent {
            key: "m".into(),
            value: stages.member,
        })
        .unwrap();
    within(Box::new(move || {
        envelope.execute_recorded(metadata, ExecutionAllocationPolicy::SystemAllocation)
    }))
}

fn recorded<O>(execution: Execution<'_, O>) -> (Document<'_, O>, WorthQueryHistoryStep) {
    match execution {
        Ok(Outcome::Recorded { document, step, .. }) => (document, step),
        Ok(other) => panic!("the envelope must record: {other:?}"),
        Err(failure) => panic!("the envelope must record: {failure:?}"),
    }
}

fn refused<O>(execution: Execution<'_, O>) -> (Document<'_, O>, Denial) {
    match execution {
        Err(failure) => failure.into_parts(),
        Ok(other) => panic!("the envelope must refuse: {other:?}"),
    }
}

/// What a fresh request reads from the document and its own branch.
#[derive(Debug, PartialEq)]
struct Observed {
    current: WorthQueryHistoryNodeId,
    nodes: Vec<WorthQueryHistoryNodeId>,
    branch: WorthQueryProductBranch,
    model: u64,
    member: u64,
}

fn observe<O: WorthQueryHistoryOutputsMode<Schema, History>>(
    host: &world::Host,
    document: &Document<'_, O>,
) -> Observed {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let branch = document
        .selection(&request)
        .unwrap()
        .physical_endpoint()
        .product_branch();
    let member = host
        .request(&principal, &scope)
        .on_branch(branch)
        .query(query::member_public::Read("m".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .value;
    Observed {
        current: document.current_step(&request).unwrap().id(),
        nodes: document
            .history_page(&request, limits(), None)
            .unwrap()
            .rows()
            .iter()
            .map(|row| row.step().id())
            .collect(),
        branch,
        model: world::model(host, branch).value,
        member,
    }
}

/// Closing disposes every head the document still holds, including a
/// candidate an abandoned envelope left for it.
fn close<O: WorthQueryHistoryOutputsMode<Schema, History>>(
    host: &world::Host,
    document: Document<'_, O>,
) {
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
    assert!(host.runtime().branches().pending_cleanup().is_empty());
}
