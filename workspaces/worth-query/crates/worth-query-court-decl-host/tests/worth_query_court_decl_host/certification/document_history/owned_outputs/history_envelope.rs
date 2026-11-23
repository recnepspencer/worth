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
mod recording;
mod replay;
mod rollback;

type Execution<'a> = Result<
    WorthQueryHistoryEditEnvelopeOutcome<'a, Schema, History>,
    WorthQueryHistoryEditEnvelopeFailure<'a, Schema, History>,
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
fn record<'a>(
    host: &'a world::Host,
    document: world::Document<'a>,
    scope: &WorthQueryRequestScope,
    stages: Stages,
    metadata: WorthQueryHistoryStepMetadata,
) -> Execution<'a> {
    let principal = authenticate_operator(host.installed_schema(), scope);
    let branch = document
        .selection(&host.request(&principal, scope))
        .unwrap()
        .physical_endpoint()
        .product_branch();
    record_on(host, document, scope, branch, stages, metadata)
}

/// `record` for a document whose branch the caller already read.
fn record_on<'a>(
    host: &'a world::Host,
    document: world::Document<'a>,
    scope: &WorthQueryRequestScope,
    branch: WorthQueryProductBranch,
    stages: Stages,
    metadata: WorthQueryHistoryStepMetadata,
) -> Execution<'a> {
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
    envelope.execute_recorded(metadata, ExecutionAllocationPolicy::SystemAllocation)
}

fn recorded(execution: Execution<'_>) -> (world::Document<'_>, WorthQueryHistoryStep) {
    match execution {
        Ok(Outcome::Recorded { document, step, .. }) => (document, step),
        Ok(other) => panic!("the envelope must record: {other:?}"),
        Err(failure) => panic!("the envelope must record: {failure:?}"),
    }
}

fn refused(execution: Execution<'_>) -> (world::Document<'_>, Denial) {
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

fn observe(host: &world::Host, document: &world::Document<'_>) -> Observed {
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
fn close(host: &world::Host, document: world::Document<'_>) {
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
    assert!(host.runtime().branches().pending_cleanup().is_empty());
}
