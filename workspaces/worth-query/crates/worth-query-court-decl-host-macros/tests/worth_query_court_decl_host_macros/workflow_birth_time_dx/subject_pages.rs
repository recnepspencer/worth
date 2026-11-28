//! A born instance migrated onto a newer revision: its coordinate-recorded
//! successor and the relation-recorded birth both belong to the subject, so
//! a subject page lists each exactly once, and a page too narrow for both
//! carries the birth to its continuation.
use crate::time::controlled_at;
use worth_query_host::facade::application_discovery::WorthQueryWorkflowObservedState as State;
use worth_query_host::facade::application_entry::*;

use super::birth::birth_in;
use super::program::owner;
use super::subject_control::StartRequest;
use super::subject_lookup::{pages, subject};
use super::workflow::{open_home, publish, RequestWorkflow};
use super::{at, COMPANY};

#[test]
fn a_migrated_birth_follows_its_successor_onto_the_next_subject_page() {
    let clock = controlled_at(at(2026, 11, 26, 15, 30));
    let (home, _signing) = open_home(clock, None);
    let caller = owner(home.runtime().runtime().installed_schema());
    let v1 = publish(&home, &caller, 1, None);
    let born = birth_in(&home, &v1, &caller, COMPANY, 303);
    let v2 = publish(&home, &caller, 2, Some(v1));
    let migration = home
        .runtime()
        .runtime()
        .request(&caller.principal, &caller.scope)
        .mutate(StartRequest(subject(COMPANY, 303)))
        .without_source()
        .idempotency(&30_u64)
        .prepare_workflow_instance_migration(
            home.workflow_vocabulary::<RequestWorkflow>().unwrap(),
            born.clone(),
            v2.clone(),
            "review",
        )
        .unwrap_or_else(|denial| panic!("the born instance admits migration: {denial:?}"))
        .execute();
    let WorkflowInstanceStartOutcome::Started(done) = migration else {
        panic!("the born instance migrates: {migration:?}");
    };
    let successor = done.instance().clone();
    assert_ne!(successor.entity_id(), born.entity_id());
    assert_eq!(
        successor.definition_content_identity(),
        v2.content_identity()
    );

    let wide = pages(&home, &caller, (COMPANY, 303), 2).unwrap();
    let [page] = wide.as_slice() else {
        panic!("both fit one page; got {} pages", wide.len());
    };
    let [(State::Ready, Some(live)), (State::Migrated, None)] = page.as_slice() else {
        panic!(
            "the successor then its migrated birth; got {} entries",
            page.len()
        );
    };
    assert_eq!(live.entity_id(), successor.entity_id());

    let narrow = pages(&home, &caller, (COMPANY, 303), 1).unwrap();
    let [first, second] = narrow.as_slice() else {
        panic!("one entry per page; got {} pages", narrow.len());
    };
    assert!(
        matches!(first.as_slice(), [(State::Ready, Some(live))] if live.entity_id() == successor.entity_id())
    );
    assert!(matches!(second.as_slice(), [(State::Migrated, None)]));
}
