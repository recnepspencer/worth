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
mod decisive;
pub(super) use child::{continue_original, release_original};

#[test]
fn recovered_original_release_kill_reopen_stays_released() {
    script_journey(false, false);
}
#[test]
fn recovered_original_continue_kill_reopen_publishes_once() {
    script_journey(true, false);
}
#[test]
fn killed_after_continued_resumes_original_occurrence_and_forbids_release() {
    script_journey(true, true);
}
#[test]
fn recovered_original_release_kill_reopen_stays_released_before_first_checkpoint() {
    checkpoint_posture_journey(false, false, false);
}
#[test]
fn recovered_original_continue_kill_reopen_publishes_once_before_first_checkpoint() {
    checkpoint_posture_journey(true, false, false);
}
#[test]
fn killed_after_continued_resumes_original_occurrence_and_forbids_release_before_first_checkpoint()
{
    checkpoint_posture_journey(true, true, false);
}
#[test]
fn recovered_original_release_kill_reopen_stays_released_after_first_checkpoint() {
    checkpoint_posture_journey(false, false, true);
}
#[test]
fn recovered_original_continue_kill_reopen_publishes_once_after_first_checkpoint() {
    checkpoint_posture_journey(true, false, true);
}
#[test]
fn killed_after_continued_resumes_original_occurrence_and_forbids_release_after_first_checkpoint() {
    checkpoint_posture_journey(true, true, true);
}

fn start(path: &Path, root: &Path, generation: &mut usize) -> (Session, super::protocol::Response) {
    *generation += 1;
    Session::start(path, &root.join(format!("session-{generation}")), 0)
}
fn assert_checkpoint_posture(session: &mut Session, expect_checkpointed: bool, label: &str) {
    let Some(Disposition::Cadence { journals }) = session.send(Request::InspectCadence).disposition
    else {
        panic!("expected cadence inspection ({label})");
    };
    if expect_checkpointed {
        assert!(
            journals.iter().any(|(_, covered, _)| *covered > 0),
            "post-checkpoint journey must have journal coverage ({label}): {journals:?}"
        );
    } else {
        assert!(
            journals.iter().all(|(_, covered, _)| *covered == 0),
            "pre-checkpoint journey must have zero checkpoint coverage ({label}): {journals:?}"
        );
    }
}
fn script_journey(continuing: bool, interrupt_continued: bool) {
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
    complete_recovered_journey(
        session,
        &path,
        root.path(),
        &mut generation,
        original,
        key,
        value,
        continuing,
        interrupt_continued,
        None,
    );
}
fn checkpoint_posture_journey(
    continuing: bool,
    interrupt_continued: bool,
    take_first_checkpoint: bool,
) {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("home");
    let mut generation = 0;
    let (mut session, _) = start(&path, root.path(), &mut generation);
    let seeded = session.send(Request::Mutate { key: 1, value: 10 });
    assert_eq!(
        seeded.mutation.as_ref().map(|m| m.fate),
        Some(host_observation::receipts::MutationFate::Committed),
        "initial mutation must commit before checkpoint posture split"
    );
    if take_first_checkpoint {
        let checkpointed = session.send(Request::Checkpoint);
        assert!(
            matches!(
                checkpointed.disposition,
                Some(Disposition::Checkpointed { generation }) if generation >= 1
            ),
            "first home checkpoint must land before abandonment: {:?}",
            checkpointed.disposition
        );
    }
    assert_checkpoint_posture(&mut session, take_first_checkpoint, "before abandonment");
    let (key, value) = (2, 20);
    let abandoned = session.send(Request::AbandonOriginal { key, value });
    let original = abandoned.state.unwrap();
    complete_recovered_journey(
        session,
        &path,
        root.path(),
        &mut generation,
        original,
        key,
        value,
        continuing,
        interrupt_continued,
        Some(take_first_checkpoint),
    );
}
#[allow(clippy::too_many_arguments)]
fn complete_recovered_journey(
    mut session: Session,
    path: &Path,
    root: &Path,
    generation: &mut usize,
    original: host_observation::projection::StateProjection,
    key: u64,
    value: u64,
    continuing: bool,
    interrupt_continued: bool,
    expected_checkpoint_posture: Option<bool>,
) {
    let [original_custody] = original.custody.as_slice() else {
        panic!("one actual original abandoned effect")
    };
    assert!(original_custody.original_effect.is_some());
    session.kill();
    let (mut session, reopened) = start(path, root, generation);
    let state = reopened.state.unwrap();
    let [custody] = state.custody.as_slice() else {
        panic!("actual custody reaches reopened service")
    };
    assert_eq!(custody.identity, original_custody.identity);
    assert_eq!(custody.original_effect, original_custody.original_effect);
    assert_eq!(state.head, original.head);
    assert_eq!(reopened.contacts, 0);
    assert_eq!(reopened.initialized, 0);
    if let Some(expect_checkpointed) = expected_checkpoint_posture {
        assert_checkpoint_posture(&mut session, expect_checkpointed, "after first kill/reopen");
    }
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
        let (next, recovered) = start(path, root, generation);
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
    let released = if continuing {
        None
    } else {
        let observed = session.send(Request::ObserveReleased { key, value });
        assert_eq!(
            observed.contacts, 0,
            "terminal retries never contact the handler"
        );
        Some(observed.disposition.unwrap())
    };
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
    let (mut session, recovered) = start(path, root, generation);
    let stable = recovered.state.unwrap();
    assert!(stable.custody.is_empty());
    assert_eq!(stable.head, selected.head);
    assert_eq!(stable.occurrences.len(), selected.occurrences.len());
    assert_eq!(recovered.contacts, 0);
    if let Some(expect_checkpointed) = expected_checkpoint_posture {
        assert_checkpoint_posture(&mut session, expect_checkpointed, "after final kill/reopen");
    }
    if let Some(released) = released {
        let observed = session.send(Request::ObserveReleased { key, value });
        assert_eq!(observed.contacts, 0);
        assert_eq!(
            observed.disposition.as_ref(),
            Some(&released),
            "kill and reopen preserve the complete terminal binding and original source"
        );
    }
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
