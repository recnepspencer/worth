//! A document that keeps its runtime alive is stored beside that runtime.
use super::*;
use worth_query_host::facade::application_entry::WorthQueryOwnedApplicationHistoryDocument;
use worth_query_host::facade::product::WorthQueryOwnedProductHead;

type Owned = WorthQueryOwnedApplicationHistoryDocument<
    Schema,
    History,
    OrdinaryHistoryPayload,
    WorthQueryHistoryWithOutputTrees<Schema>,
>;

/// What an application keeps for one open document. Neither field borrows the other.
struct Session {
    host: world::Host,
    document: Option<Owned>,
}

const _: fn() = || {
    fn shared_between_threads<T: Send + Sync + 'static>() {}
    fn borrows_nothing<T: 'static>() {}
    shared_between_threads::<WorthQueryOwnedApplicationHistoryDocument<Schema, History>>();
    // Bound output trees keep type-erased producers that are not shareable
    // between threads, as they are for the borrowed document.
    borrows_nothing::<Owned>();
    borrows_nothing::<Session>();
};

/// A head forked from the current world, not yet attached to any history.
fn fork_head<'a>(
    host: &'a world::Host,
    phase: &WorthQueryAdvancementPhase<'_>,
) -> WorthQueryOwnedProductHead<'a, Schema> {
    let selected = host.on_branch(host.current_world()).select(phase).unwrap();
    host.branches()
        .fork_expected(selected)
        .unwrap()
        .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
        .create(phase)
        .unwrap()
        .into_owned_head()
}

fn open() -> Session {
    let host = world::install();
    let scope = request_scope();
    let document = {
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        first::settle(&host, &request, "a");
        second::settle(&host, &request, "a");
        let document = host
            .with_host_advancement(|phase| {
                let head = fork_head(&host, &phase);
                request
                    .on_branch(head.branch())
                    .attach_owned_history::<History, _>(
                        &host,
                        &phase,
                        head,
                        query::model::Read("a".into()),
                    )
                    .unwrap()
            })
            .unwrap();
        courts::configure_all_trees(&host, document)
    };
    Session {
        host,
        document: Some(document),
    }
}

#[test]
fn an_owned_document_lives_beside_its_runtime() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let mut session = open();
    let document = session.document.take().unwrap();
    let (document, a) = record_step(&session.host, document, "owned-a", 2, 6);
    let at_a = snapshot(&session.host, &document);
    let (document, b) = record_step(&session.host, document, "owned-b", 3, 7);
    let at_b = snapshot(&session.host, &document);
    assert_eq!(b.parent(), Some(a.id()));
    assert_eq!((at_b.outputs.0.value, at_b.outputs.1.value), (30, 1003));
    navigate(&session.host, &document, Move::Undo, 0x57e0_0021);
    assert_eq!(snapshot(&session.host, &document), at_a);
    navigate(&session.host, &document, Move::RedoTo(b.id()), 0x57e0_0022);
    assert_eq!(snapshot(&session.host, &document), at_b);
    session.document = Some(document);
    // The open document keeps the runtime alive, so the runtime cannot shut down under it.
    assert_eq!(
        session
            .host
            .close_conditional_runtime()
            .unwrap_err()
            .kind(),
        WorthQueryConditionalRuntimeInstallationDenialKind::RuntimeKeepaliveRetained
    );
    let document = session.document.take().unwrap();
    assert_eq!(snapshot(&session.host, &document), at_b);
    let _receipt: WorthQueryDocumentHistoryCloseReceipt = session
        .host
        .with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
    assert!(session.host.runtime().branches().pending_cleanup().is_empty());
    session.host.close_conditional_runtime().unwrap();
}

/// The borrowed head a refusal returns still attaches on its own runtime.
fn attaches_borrowed(host: &world::Host, head: WorthQueryOwnedProductHead<'_, Schema>) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let document = host
        .with_host_advancement(|phase| {
            host.request(&principal, &scope)
                .on_branch(head.branch())
                .attach_history::<History>(&phase, head, query::model::Read("a".into()))
                .unwrap()
        })
        .unwrap();
    close(host, document);
}

#[test]
fn a_head_of_another_runtime_is_returned_unchanged() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let other = world::install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let head = other
        .with_host_advancement(|phase| fork_head(&other, &phase))
        .unwrap();
    let branch = head.branch();
    let failure = host
        .with_host_advancement(|phase| {
            host.request(&principal, &scope)
                .on_branch(branch)
                .attach_owned_history::<History, _>(
                    &host,
                    &phase,
                    head,
                    query::model::Read("a".into()),
                )
                .unwrap_err()
        })
        .unwrap();
    let (head, denial) = failure.into_parts();
    assert!(
        matches!(denial, WorthQueryApplicationHistoryDenial::ForeignRuntime),
        "{denial:?}"
    );
    assert_eq!(head.branch(), branch);
    attaches_borrowed(&other, head);
}

#[test]
fn a_request_on_another_branch_returns_the_head_unchanged() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let head = host
        .with_host_advancement(|phase| fork_head(&host, &phase))
        .unwrap();
    let branch = head.branch();
    let failure = host
        .with_host_advancement(|phase| {
            host.request(&principal, &scope)
                .on_branch(host.current_world())
                .attach_owned_history::<History, _>(
                    &host,
                    &phase,
                    head,
                    query::model::Read("a".into()),
                )
                .unwrap_err()
        })
        .unwrap();
    let (head, denial) = failure.into_parts();
    assert!(
        matches!(denial, WorthQueryApplicationHistoryDenial::RequestTargetMismatch),
        "{denial:?}"
    );
    assert_eq!(head.branch(), branch);
    attaches_borrowed(&host, head);
    // No keepalive was left behind by the refusal.
    let mut host = host;
    host.close_conditional_runtime().unwrap();
}
