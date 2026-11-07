//! Finite host, original source operations and public navigation owners.
use crate::certification::document_history::recording::{begin, finish, ready};
use crate::certification::document_history::{App, Document};
use std::num::NonZeroUsize;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationRequestExt, WorthQueryHistoryDescriptionContext,
        WorthQueryHistoryNavigationAdvance as Advance, WorthQueryHistoryNavigationCompletion,
        WorthQueryHistoryPageLimits, WorthQueryHistoryReleaseOutcome as Release,
        WorthQueryHistoryStep,
    },
    declaration::application_history::ApplicationHistoryRecentSteps,
};
use worth_query_testkit_document_retention::facade::{
    document_retention_model::{
        document_history::{navigation::Navigation, retention_policy::Window},
        host::{
            primary_graph, publish_on_first_program_with_limits, runtime,
            WorthQueryApplicationLimits,
        },
        operator_identity::{authenticate_operator, request_scope},
        retention_entry::DOCUMENT_IDENTITY,
    },
    document_world_resources::{world_resources_with_capacity, WorldLifecycleCapacity},
};

pub(super) fn install(nodes: usize) -> App {
    let nodes = u64::try_from(nodes).unwrap();
    let capacity = WorldLifecycleCapacity {
        live_product_branches: nodes + 32,
        // Retained original sources plus bounded receiving-phase observations.
        active_observations: nodes * 4 + 256,
        // At most one owned Relational and one owned Signal component per branch.
        owner_created_component_custody_records: 2 * (nodes + 32),
    };
    eprintln!(
        "history capacity={capacity:?}; retained_commits={}; metadata_bytes={}; exact_pins={}",
        nodes * 4 + 128,
        128 * 1024 * 1024,
        nodes * 4 + 256
    );
    publish_on_first_program_with_limits(WorthQueryApplicationLimits::new(
        world_resources_with_capacity(
            nodes * 4 + 128,
            128 * 1024 * 1024,
            nodes * 4 + 256,
            capacity,
            worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
        ),
        runtime::WorthQueryApplicationQueryResourceProfile::new(NonZeroUsize::new(128).unwrap()),
        primary_graph::SignalConditionalEvaluationBudget::development(),
    ))
}

pub(super) fn prefix() -> WorthQueryHistoryPageLimits {
    WorthQueryHistoryPageLimits {
        maximum_entries: NonZeroUsize::MIN,
        maximum_examined_entries: NonZeroUsize::MIN,
        maximum_owned_bytes: NonZeroUsize::new(16 * 1024).unwrap(),
    }
}

pub(super) fn window(host: &App, document: &mut Document<'_>, nodes: usize) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut change = document.set_history_window(ApplicationHistoryRecentSteps::new(
        NonZeroUsize::new(nodes).unwrap(),
    ));
    let status = change.advance::<Window>(&request).unwrap();
    assert!(status.pending_window.is_none());
    assert_eq!(status.recent_steps.get(), nodes);
}

pub(super) fn author<'a>(
    host: &App,
    document: Document<'a>,
    days: u64,
    key: u64,
) -> (Document<'a>, WorthQueryHistoryStep) {
    finish(
        host,
        ready(host, begin(host, document, DOCUMENT_IDENTITY, days, key)),
        days,
    )
}

pub(super) fn select(
    host: &App,
    document: &mut Document<'_>,
    context: &WorthQueryHistoryDescriptionContext,
    target: &WorthQueryHistoryStep,
    key: u64,
) -> WorthQueryHistoryNavigationCompletion {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let expected = document.selection(&request).unwrap();
    let mut owner = document
        .commands(context)
        .unwrap()
        .navigate::<Navigation>()
        .checkout(target.id())
        .expected_selection(expected)
        .idempotency(u128::from(key))
        .prepare(&request)
        .unwrap();
    for _ in 0..16 {
        owner = match owner.advance(&request) {
            Advance::Pending(owner) => owner,
            Advance::Selected(completion) => return completion,
            Advance::Refused { cause, .. } => panic!("lawful scale checkout: {cause:?}"),
        };
    }
    panic!("finite scale checkout")
}

pub(super) fn maintain(
    host: &App,
    document: &mut Document<'_>,
    context: &WorthQueryHistoryDescriptionContext,
) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    for _ in 0..33 {
        match document
            .commands(context)
            .unwrap()
            .maintain(&request)
            .unwrap()
        {
            Release::NoPendingRelease => return,
            Release::Closed => {}
            other => panic!("unheld original cleanup must finish: {other:?}"),
        }
    }
    panic!("the admitted queue has at most32 owners")
}
