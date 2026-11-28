//! A request born enrolled is reacquired only through the governed subject
//! read, never through the birth receipt: in the birth session once its
//! commit is observed, after a newer revision supersedes the one it was born
//! on, and after its Directory Home closes and reopens. The reacquired v1
//! locator advances; principals without a read path to the request see
//! nothing.
use crate::time::controlled_at;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;
use worth_query_host::facade::runtime::time::system_time_sampler;

use worth_query_decl::facade::application_program::{
    ApplicationWorkflowDefinitionIdentity, ApplicationWorkflowDefinitionReadTarget,
};
use worth_query_host::facade::application_discovery::{
    WorthQueryWorkflowIndexRead, WorthQueryWorkflowInstanceRead,
    WorthQueryWorkflowObservedState as State, WorthQueryWorkflowReadExtent,
};
use worth_query_host::facade::application_entry::*;

use super::birth::birth_in;
use super::disclosure::{RequestSubject, SubjectRead};
use super::program::{outsider, owner, Caller, OTHER_COMPANY};
use super::subject_control::{AdvanceRequest, Review};
use super::workflow::{open_home, publish, RequestWorkflow, WorkflowHome, LINEAGE};
use super::{at, COMPANY};

/// Each subject page entry: its state and, while live, its fresh locator.
pub type Entries = Vec<(State, Option<PublishedWorkflowInstanceRef>)>;

/// Every subject page for `request` at one current basis, `width` entries at
/// most per page, following each continuation under fresh admission.
pub fn pages(
    home: &WorkflowHome,
    caller: &Caller,
    request: (&str, u64),
    width: usize,
) -> Result<Vec<Entries>, String> {
    let read = home
        .runtime()
        .runtime()
        .request(&caller.principal, &caller.scope);
    let basis = read
        .retain_current_read()
        .map_err(|cause| format!("{cause:?}"))?;
    let at = read.at(&basis);
    let target = ApplicationWorkflowDefinitionReadTarget::<RequestWorkflow>::new(
        ApplicationWorkflowDefinitionIdentity::new(LINEAGE).unwrap(),
    );
    // Each continuation is its own borrow of the page before it.
    macro_rules! page {
        ($after:expr) => {
            at.read_workflow_instances::<RequestWorkflow, SubjectRead>(
                home.workflow_vocabulary::<RequestWorkflow>().unwrap(),
                &target,
                subject(request.0, request.1),
                WorthQueryWorkflowReadExtent::Subject,
                width,
                WorthQueryWorkflowIndexRead::PublishedOnly,
                $after,
            )
            .map_err(|cause| format!("{cause:?}"))
        };
    }
    let mut current = page!(None)?;
    let mut pages = Vec::new();
    loop {
        assert!(current.entries().len() <= width);
        pages.push(current.entries().iter().map(entry).collect());
        let Some(cursor) = current.continuation() else {
            return Ok(pages);
        };
        let next = page!(Some(cursor))?;
        current = next;
    }
}

fn entry(read: &WorthQueryWorkflowInstanceRead) -> (State, Option<PublishedWorkflowInstanceRef>) {
    match read {
        WorthQueryWorkflowInstanceRead::Live {
            instance,
            observation,
        } => (observation.state(), Some(instance.clone())),
        WorthQueryWorkflowInstanceRead::Historical { observation, .. } => {
            (observation.state(), None)
        }
        WorthQueryWorkflowInstanceRead::Missing { .. } => panic!("a page lists located records"),
    }
}

/// The single subject page for `request` in `company`.
fn lookup(
    home: &WorkflowHome,
    caller: &Caller,
    company: &str,
    request: u64,
) -> Result<Entries, String> {
    let mut pages = pages(home, caller, (company, request), 2)?;
    assert_eq!(pages.len(), 1, "one subject fits one page");
    Ok(pages.remove(0))
}

/// The one live instance a subject page discloses.
fn live(entries: &Entries) -> PublishedWorkflowInstanceRef {
    match entries.as_slice() {
        [(State::Ready, Some(instance))] => instance.clone(),
        other => panic!("one live born instance, got {} entries", other.len()),
    }
}

pub fn subject(company: &str, request: u64) -> RequestSubject {
    RequestSubject {
        company: company.to_owned(),
        request,
    }
}

/// Records the review proposal on `instance`, then advances it to the end.
fn complete(home: &WorkflowHome, caller: &Caller, instance: PublishedWorkflowInstanceRef) {
    let acme = || subject(COMPANY, 301);
    let request = || {
        home.runtime()
            .runtime()
            .request(&caller.principal, &caller.scope)
    };
    let proposal = request()
        .mutate(Review(acme()))
        .without_source()
        .idempotency(&10_u64)
        .prepare_workflow_proposal(
            home.workflow_vocabulary::<RequestWorkflow>().unwrap(),
            instance.clone(),
        )
        .unwrap_or_else(|denial| panic!("the reacquired locator admits a proposal: {denial:?}"))
        .execute();
    assert!(
        matches!(proposal, WorkflowProposalOutcome::Published(_)),
        "{proposal:?}"
    );
    let advance = request()
        .mutate(AdvanceRequest(acme()))
        .without_source()
        .idempotency(&20_u64)
        .prepare_workflow_advance(
            home.workflow_vocabulary::<RequestWorkflow>().unwrap(),
            instance,
        )
        .unwrap_or_else(|denial| panic!("the reacquired locator admits an advance: {denial:?}"))
        .execute();
    assert!(
        matches!(advance, WorkflowProgressOutcome::Completed(_)),
        "{advance:?}"
    );
}

fn scratch_directory() -> PathBuf {
    let nanos = system_time_sampler().wall().unwrap().unix_nanos();
    let process = std::process::id();
    std::env::temp_dir().join(format!("worth-birth-subject-{process}-{nanos}"))
}

#[test]
fn a_born_request_is_reacquired_by_subject_after_supersession_and_reopen() {
    let directory = scratch_directory();
    let clock = controlled_at(at(2026, 11, 26, 15, 30));
    let (home, signing) = open_home(clock.clone(), Some(&directory));
    let caller = owner(home.runtime().runtime().installed_schema());
    let v1 = publish(&home, &caller, 1, None);

    // The birth session discovers the instance once its commit is observed,
    // without consulting the birth receipt.
    let (send, observed) = mpsc::channel();
    let watch = home
        .on_commit(move || {
            let _ = send.send(());
        })
        .unwrap();
    let born = birth_in(&home, &v1, &caller, COMPANY, 301);
    observed.recv_timeout(Duration::from_secs(5)).unwrap();
    drop(watch);
    let found = live(&lookup(&home, &caller, COMPANY, 301).unwrap());
    assert_eq!(found.entity_id(), born.entity_id());
    birth_in(&home, &v1, &caller, OTHER_COMPANY, 302);
    let v2 = publish(&home, &caller, 2, Some(v1.clone()));
    assert_ne!(v2.content_identity(), v1.content_identity());
    drop(caller);
    if home.close_clean().is_err() {
        panic!("the Directory Home closes cleanly");
    }
    drop(signing);

    let (home, _signing) = open_home(clock, Some(&directory));
    let schema = home.runtime().runtime().installed_schema();
    let caller = owner(schema);
    let pinned = live(&lookup(&home, &caller, COMPANY, 301).unwrap());
    assert_eq!(pinned.entity_id(), born.entity_id());
    assert_eq!(
        pinned.definition_content_identity(),
        v1.content_identity(),
        "the reacquired instance stays pinned to the revision it was born on"
    );

    // No read path, no disclosure: an outsider holding only command grants,
    // the owner at a company it cannot read, and a request named under the
    // wrong company.
    lookup(&home, &outsider(schema), COMPANY, 301).unwrap_err();
    lookup(&home, &caller, OTHER_COMPANY, 302).unwrap_err();
    lookup(&home, &caller, COMPANY, 302).unwrap_err();

    complete(&home, &caller, pinned);
    let ended = lookup(&home, &caller, COMPANY, 301).unwrap();
    assert!(
        matches!(ended.as_slice(), [(State::Completed, None)]),
        "the advanced v1 instance ends; {} entries",
        ended.len()
    );
    drop(caller);
    if home.close_clean().is_err() {
        panic!("the reopened Directory Home closes cleanly");
    }
    let _ = std::fs::remove_dir_all(&directory);
}
