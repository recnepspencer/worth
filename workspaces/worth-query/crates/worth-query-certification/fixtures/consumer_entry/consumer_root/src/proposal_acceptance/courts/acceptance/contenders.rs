//! Two ordinary callers share the application's one exclusively owned Model continuation.
use super::*;
use crate::proposal_acceptance::{
    application::{acceptance::Accepted, candidate::StageEvent},
    attempts::{data::StoredResult, query, result_query},
};
use std::sync::{mpsc, Mutex};

enum Reply {
    Accepted(Accepted),
    Cached(StoredResult),
}

fn drive<'host>(
    shared: &Mutex<&mut Model<'host>>,
    storage: &Storage<'host>,
    caller: &Caller<'_>,
    loaded: &LoadedProposal,
    ready: mpsc::Sender<()>,
    go: mpsc::Receiver<()>,
) -> Result<Option<Reply>, Failure> {
    let first = match shared.lock() {
        Ok(mut model) => model.accept(storage, caller, loaded, 1904),
        Err(_) => return Ok(None),
    };
    let _ = ready.send(());
    if go.recv().is_err() {
        return Ok(None);
    }
    assert!(matches!(first?, Event::Progress));
    loop {
        let event = match shared.lock() {
            Ok(mut model) => model.accept(storage, caller, loaded, 1904),
            Err(_) => return Ok(None),
        }?;
        match event {
            Event::Completed(accepted) => return Ok(Some(Reply::Accepted(accepted))),
            Event::Cached(result) => return Ok(Some(Reply::Cached(result))),
            Event::Progress
            | Event::Adopted
            | Event::Staged(StageEvent::Progress | StageEvent::StepCompleted) => {}
            Event::Staged(event) => panic!("a lawful contender cannot retry a refusal: {event:?}"),
            Event::Stopped => panic!("a lawful contender retains its ordinary progress"),
        }
    }
}

#[test]
fn equal_input_contenders_share_one_actual_acceptance_and_one_cached_result() {
    let host = host::install();
    let scope = authentication::request_scope();
    let author =
        authentication::authenticate(host.installed_schema(), Actor::Author, &scope).unwrap();
    let first_scope = authentication::request_scope();
    let first =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &first_scope)
            .unwrap();
    let second_scope = authentication::request_scope();
    let second =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &second_scope)
            .unwrap();
    let mut model = Model::create(&host, &author, "model-1", 1901, 7).unwrap();
    let other = Model::create(&host, &author, "model-2", 1902, 5).unwrap();
    let storage = Storage::create(&model, &author).unwrap();
    let reference = storage
        .record(
            &model,
            &author,
            "proposal-1",
            1903,
            vec![AuthoredIntent::Retention(edit(&model, 3))],
        )
        .unwrap();
    let loaded = storage.load_frozen(&first, &reference).unwrap();
    let source = model.branch();
    let replies = {
        let shared = Mutex::new(&mut model);
        let (ready, arrivals) = mpsc::channel();
        let (left_go, left_start) = mpsc::channel();
        let (right_go, right_start) = mpsc::channel();
        std::thread::scope(|workers| {
            let shared = &shared;
            let storage = &storage;
            let loaded = &loaded;
            let left_ready = ready.clone();
            let right_ready = ready.clone();
            let left =
                workers.spawn(|| drive(shared, storage, &first, loaded, left_ready, left_start));
            let right =
                workers.spawn(|| drive(shared, storage, &second, loaded, right_ready, right_start));
            drop(ready);
            let both_submitted = arrivals
                .recv_timeout(std::time::Duration::from_secs(10))
                .is_ok()
                && arrivals
                    .recv_timeout(std::time::Duration::from_secs(10))
                    .is_ok();
            if both_submitted {
                let _ = left_go.send(());
                let _ = right_go.send(());
            }
            drop(left_go);
            drop(right_go);
            let left = left.join();
            let right = right.join();
            assert!(
                both_submitted,
                "both requests submit before either finishes"
            );
            (
                left.unwrap().unwrap().unwrap(),
                right.unwrap().unwrap().unwrap(),
            )
        })
    };
    let (accepted, cached) = match replies {
        (Reply::Accepted(accepted), Reply::Cached(cached))
        | (Reply::Cached(cached), Reply::Accepted(accepted)) => (accepted, cached),
        _ => panic!("one actual completed adoption and one cached outcome"),
    };
    assert_eq!(cached, accepted.result);
    assert_eq!(accepted.result.claim.data.input, loaded.input(1904));
    let closed = accepted
        .performed
        .previous_close()
        .expect("actual previous-head close evidence");
    assert_eq!(closed.receipt.retired_component_count(), 1);
    assert!(closed.receipt.is_complete());
    assert!(closed.handoff.is_none());
    assert_ne!(model.branch(), source);
    assert_eq!(model.read(&first).unwrap().days, 3);
    assert_eq!(other.read(&second).unwrap().days, 5);
    {
        let claims = host
            .request(&first.principal, first.scope)
            .on_branch(storage.branch())
            .query(query::ReadClaims {
                model: model.identity.label.clone(),
            })
            .execute_governed(
                access::ReadDomain::reference(),
                access::InspectDomain::reference(),
                access::ReadAccess {
                    model: model.identity.label.clone(),
                },
            )
            .unwrap();
        assert_eq!(claims.rows().len(), 1);
        assert_eq!(claims.rows()[0].data.input, loaded.input(1904));
        let results = host
            .request(&second.principal, second.scope)
            .on_branch(storage.branch())
            .query(result_query::ReadResults {
                model: model.identity.label.clone(),
            })
            .execute_governed(
                access::ReadDomain::reference(),
                access::InspectDomain::reference(),
                access::ReadAccess {
                    model: model.identity.label.clone(),
                },
            )
            .unwrap();
        assert_eq!(results.rows().len(), 1);
        assert_eq!(results.rows()[0].data, accepted.result);
    }
    drop(accepted);
    storage.close(&author).unwrap();
    model.close(&author).unwrap();
    other.close(&author).unwrap();
    assert!(host.branches().pending_cleanup().is_empty());
}
