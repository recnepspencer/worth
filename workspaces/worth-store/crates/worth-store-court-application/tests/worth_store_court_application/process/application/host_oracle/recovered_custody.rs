//! Real Host children kill after original dispositions, never supplied fates.
use super::*;
use super::{
    protocol::{Disposition, Request},
    session::Session,
};
use host_observation::{receipts, requests, OracleHome};
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationIdempotencyResolution as Resolution,
    WorthQueryProductUnpublishedRecoveryReleaseDenial as ReleaseDenial,
    WorthQueryProductUnpublishedRecoveryReleaseFailure as ReleaseFailure,
};
use worth_runtime_world::facade::{
    ProductUnpublishedNextAction, RuntimeWorldRecoveryDenial,
    WorldRecoveredContinuationOutcome as Continued,
};

mod child;
pub(super) use child::{continue_original, release_original};

#[test]
fn recovered_original_release_kill_reopen_stays_released() {
    journey(false, false);
}
#[test]
fn recovered_original_continue_kill_reopen_publishes_once() {
    journey(true, false);
}
#[test]
fn killed_after_continued_resumes_original_occurrence_and_forbids_release() {
    journey(true, true);
}

fn start(path: &Path, root: &Path, generation: &mut usize) -> (Session, super::protocol::Response) {
    *generation += 1;
    Session::start(path, &root.join(format!("session-{generation}")), 0)
}
fn journey(continuing: bool, interrupt_continued: bool) {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("home");
    let script = HostScript::generate(0);
    let target = script
        .commands()
        .iter()
        .position(|command| matches!(command, HostCommand::AbandonCaller { .. }))
        .unwrap();
    let HostCommand::AbandonCaller { key, value } = &script.commands()[target] else {
        unreachable!()
    };
    let (key, value) = (*key, *value);
    let mut generation = 0;
    let (mut session, _) = start(&path, root.path(), &mut generation);
    let mut abandoned = None;
    for (index, command) in script.commands().iter().enumerate().take(target + 1) {
        if command.activation().owner_merge().is_some() {
            continue;
        }
        if matches!(command, HostCommand::Reopen) {
            session.kill();
            (session, _) = start(&path, root.path(), &mut generation);
        } else {
            abandoned = Some(session.send(Request::Step(index)));
        }
    }
    let original = abandoned.unwrap().state.unwrap();
    let [original_custody] = original.custody.as_slice() else {
        panic!("one actual original abandoned effect")
    };
    assert!(original_custody.original_effect.is_some());
    session.kill();
    let (mut session, reopened) = start(&path, root.path(), &mut generation);
    let state = reopened.state.unwrap();
    let [custody] = state.custody.as_slice() else {
        panic!("actual custody reaches reopened service")
    };
    assert_eq!(custody.identity, original_custody.identity);
    assert_eq!(custody.original_effect, original_custody.original_effect);
    assert_eq!(state.head, original.head);
    assert_eq!(reopened.contacts, 0);
    assert_eq!(reopened.initialized, 0);
    if interrupt_continued {
        session.dispatch(Request::ContinueOriginal {
            key,
            value,
            pause_after_disposition: true,
        });
        let arrival = session.channel.join("continued");
        let until = Instant::now() + Duration::from_secs(120);
        while !arrival.is_file() {
            assert!(
                session.child.try_wait().unwrap().is_none() && Instant::now() < until,
                "actual Continued boundary was not reached"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(
            !session.channel.join("response").exists(),
            "no response before the occurrence"
        );
        session.kill();
        let (next, recovered) = start(&path, root.path(), &mut generation);
        session = next;
        let state = recovered.state.unwrap();
        assert_eq!(state.head, original.head);
        assert_eq!(state.custody[0].identity, original_custody.identity);
        assert_eq!(
            state.custody[0].original_effect,
            original_custody.original_effect
        );
        assert!(matches!(
            session
                .send(Request::ReleaseOriginal {
                    key,
                    value,
                    expect_continued: true
                })
                .disposition,
            Some(Disposition::ContinuedReleaseRefused)
        ));
        assert_eq!(recovered.contacts, 0);
    }
    let completed = if continuing {
        session.send(Request::ContinueOriginal {
            key,
            value,
            pause_after_disposition: false,
        })
    } else {
        session.send(Request::ReleaseOriginal {
            key,
            value,
            expect_continued: false,
        })
    };
    assert_eq!(
        completed.contacts, 0,
        "recovered disposition cannot run a handler"
    );
    let selected = completed.state.unwrap();
    assert!(selected.custody.is_empty());
    if continuing {
        assert_eq!(selected.head.generation, original.head.generation + 1);
        assert_eq!(selected.occurrences.len(), original.occurrences.len() + 1);
        let Some(Disposition::Continued(mutation)) = completed.disposition else {
            panic!("actual original receipt")
        };
        let receipt = mutation.receipt.unwrap();
        assert_eq!(
            receipt.attempt,
            original_custody.original_effect.as_ref().unwrap().attempt
        );
        assert_eq!(receipt.before, original.head);
    } else {
        assert_eq!(selected.head, original.head);
    }
    session.kill();
    let (mut session, recovered) = start(&path, root.path(), &mut generation);
    let stable = recovered.state.unwrap();
    assert!(stable.custody.is_empty());
    assert_eq!(stable.head, selected.head);
    assert_eq!(stable.occurrences.len(), selected.occurrences.len());
    assert_eq!(recovered.contacts, 0);
    if continuing {
        let retry = session.send(Request::ContinueOriginal {
            key,
            value,
            pause_after_disposition: false,
        });
        assert_eq!(retry.contacts, 0);
        assert_eq!(
            retry.state.unwrap().head,
            selected.head,
            "same key does not publish twice"
        );
    }
}
