//! What a fresh request reads: the document branch's members and output
//! rows, and the document's selection, `history_page` and `redo_choices`.
use super::*;
use std::num::NonZeroUsize;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAdmissionClock, WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::product::WorthQueryProductBranch;

pub type Host = WorthQueryProgramApplicationRuntime<Schema, program::Program>;
/// The model history with outputs `O`.
pub type Doc<'h, O> = WorthQueryApplicationHistoryDocument<'h, Schema, History, OrdinaryHistoryPayload, O>;
/// The DST's document: the model history with its output tree bound.
pub type Forest<'h> = Doc<'h, WorthQueryHistoryWithOutputTrees<Schema>>;

/// One DST call's deadline. A call settles in well under a second, so a
/// call that runs this long is a hang: it refuses `DeadlineExceeded` here
/// instead of after the shared scope's 600 s (traced, seed 0x7b020039).
const CALL_DEADLINE: std::time::Duration = std::time::Duration::from_secs(30);

/// A fresh request scope that ends at [`CALL_DEADLINE`].
fn call_scope() -> WorthQueryRequestScope {
    let clock = WorthQueryAdmissionClock::system();
    WorthQueryRequestScope::new(Some(clock.now() + CALL_DEADLINE), WorthQueryCancellationSource::new().token(), clock)
}

/// Run `body` under a fresh request on the operator's principal.
pub fn with_request<T>(host: &Host, body: impl FnOnce(&family::Request<'_, '_, '_>) -> T) -> T {
    let scope = call_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    body(&host.request(&principal, &scope))
}

/// Run `body` under a fresh request on `branch`.
pub fn with_branch_request<T>(
    host: &Host,
    branch: WorthQueryProductBranch,
    body: impl FnOnce(&family::Request<'_, '_, '_>) -> T,
) -> T {
    let scope = call_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    body(&host.request(&principal, &scope).on_branch(branch))
}

/// A live entity: its full-generation ID and value.
#[derive(Clone, Copy, Debug)]
pub struct Live {
    pub id: worth_relational::facade::identity::EntityId,
    pub value: u64,
}

/// The members `e1` and `e2` of model `a` on `branch`. A member that is not
/// in the model's member set is not live: its ownership edge is part of it.
pub fn read_entities(host: &Host, branch: WorthQueryProductBranch) -> [Option<Live>; 2] {
    try_read_entities(host, branch).unwrap_or_else(|denial| panic!("the member set must read: {denial:?}"))
}

/// [`read_entities`], with the read's refusal.
pub fn try_read_entities(
    host: &Host,
    branch: WorthQueryProductBranch,
) -> Result<[Option<Live>; 2], WorthQueryApplicationRequestQueryDenial> {
    let members = with_branch_request(host, branch, |request| {
        request
            .query(query::members::Read("a".into()))
            .execute()
            .map(|rows| rows.rows()[0].members.clone())
    })?;
    let live = NAMES.map(|name| {
        members
            .iter()
            .find(|row| row.key == name)
            .map(|row| Live { id: row.native_id, value: row.value })
    });
    assert!(live[E2].is_none() || live[E1].is_some(), "a live e2 needs a live e1: {members:?}");
    Ok(live)
}

/// Both output families' values per entity: a live entity has exactly one
/// row in each, an entity that is not live has none.
pub fn read_outputs(host: &Host, branch: WorthQueryProductBranch, live: &[Option<Live>; 2]) -> ([u64; 2], [u64; 2]) {
    with_branch_request(host, branch, |request| {
        let mut outputs = ([0; 2], [0; 2]);
        for entity in ENTITIES {
            for (profile, values) in [("dst-a", &mut outputs.0), ("dst-b", &mut outputs.1)] {
                let read = request
                    .query(query::child_output::Read(format!("{profile}:{}", NAMES[entity])))
                    .execute();
                match (live[entity], read) {
                    (Some(_), Ok(rows)) => values[entity] = rows.rows()[0].value,
                    (None, Err(_)) => {}
                    (Some(_), Err(denial)) => panic!("{profile} of live {} must read: {denial:?}", NAMES[entity]),
                    (None, Ok(rows)) => panic!("{profile} of retired {} reads {:?}", NAMES[entity], rows.rows()),
                }
            }
        }
        outputs
    })
}

/// The #358 chain's values for model `a`: `dst-mid`'s, then `dst-leaf`'s.
/// Both are model-keyed, so each reads one row whatever is live.
pub fn read_chain(host: &Host, branch: WorthQueryProductBranch) -> (u64, u64) {
    with_branch_request(host, branch, |request| {
        let read = |profile: &str| match request.query(query::first_output::Read(format!("{profile}:a"))).execute() {
            Ok(rows) => rows.rows()[0].value,
            Err(denial) => panic!("{profile} of model a must read: {denial:?}"),
        };
        (read("dst-mid"), read("dst-leaf"))
    })
}

/// One `history_page` row.
pub struct Entry {
    pub step: WorthQueryHistoryNodeId,
    /// `None` is the `Root` boundary.
    pub parent: Option<WorthQueryHistoryNodeId>,
    pub label: Option<String>,
    pub metadata: Vec<u8>,
}

/// The selection, every `history_page` page and every `redo_choices` page.
pub struct Tree {
    pub current: WorthQueryHistoryNodeId,
    pub history: Vec<Vec<Entry>>,
    pub choice: WorthQueryHistoryRedoChoice,
    pub children: Vec<Vec<WorthQueryHistoryNodeId>>,
}

fn limits(limit: u32) -> WorthQueryHistoryPageLimits {
    let limit = NonZeroUsize::new(limit as usize).expect("page limits start at 1");
    WorthQueryHistoryPageLimits {
        maximum_entries: limit,
        maximum_examined_entries: limit,
        maximum_owned_bytes: NonZeroUsize::new(16 * 1024).unwrap(),
    }
}

/// Pages bound a walk; the DST's trees stay far inside it.
const PAGES: usize = 256;

/// Read the tree, each call under its own fresh request, walking each paged
/// read from its first page until it has no continuation.
pub fn read_tree(host: &Host, document: &Forest<'_>, limit: u32) -> Result<Tree, WorthQueryApplicationHistoryDenial> {
    let current = with_request(host, |request| document.current_step(request))?.id();
    let mut history = Vec::new();
    let mut after = None;
    for _ in 0..PAGES {
        let page = with_request(host, |request| document.history_page(request, limits(limit), after.as_ref()))?;
        history.push(page.rows().iter().map(|row| entry(row.step())).collect());
        after = page.continuation();
        if after.is_none() {
            break;
        }
    }
    assert!(after.is_none(), "history_page must end within {PAGES} pages");
    let first = with_request(host, |request| document.redo_choices(request, limits(limit), None))?;
    let choice = first.choice();
    let mut children = vec![ids(first.children())];
    let mut after = first.children().continuation();
    for _ in 0..PAGES {
        let Some(cursor) = after else { break };
        let page = with_request(host, |request| document.redo_choices(request, limits(limit), Some(&cursor)))?;
        children.push(ids(page.children()));
        after = page.children().continuation();
    }
    assert!(after.is_none(), "redo_choices must end within {PAGES} pages");
    Ok(Tree { current, history, choice, children })
}

fn ids(page: &WorthQueryHistoryPage) -> Vec<WorthQueryHistoryNodeId> {
    page.rows().iter().map(|row| row.step().id()).collect()
}

fn entry(step: &WorthQueryHistoryPageEntry) -> Entry {
    Entry {
        step: step.id(),
        parent: match step.parent() {
            WorthQueryHistoryParentBoundary::Root => None,
            WorthQueryHistoryParentBoundary::Retained(parent) => Some(parent),
            WorthQueryHistoryParentBoundary::Truncated => panic!("the DST retains every step"),
        },
        label: step.label().map(str::to_owned),
        metadata: step.metadata().bytes().to_vec(),
    }
}
