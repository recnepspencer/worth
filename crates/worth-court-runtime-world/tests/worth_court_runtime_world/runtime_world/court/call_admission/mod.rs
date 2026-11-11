//! Ordinary calls share admission; close excludes them and unwind releases the original entry.
use crate::court::ordinary_context::CompositeSupplyChainCourt;
use worth_runtime_world::facade::RuntimeWorldCloseDenial;
#[test]
fn public_call_ports_refuse_borrowed_preclose_and_serve_after_its_drop() {
    let court = CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let policy = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&policy);
    let publication = court.world.publication_port();
    let observation = court.world.observation_port();
    publication.with_execution_placement(|_| ()).unwrap();
    let admission = court.world.try_prepare_close_admission(&execution).unwrap();
    assert!(publication
        .with_execution_placement(|_| panic!("excluded call ran"))
        .is_err());
    assert!(observation.observe_root_product_branch().is_err());
    drop(admission);
    publication.with_execution_placement(|_| ()).unwrap();
    drop(root);
    court.finish();
}
#[test]
fn concurrent_public_call_ports_all_admit_and_exclude_close_until_their_return() {
    let court = CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let policy = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&policy);
    let publication = court.world.publication_port();
    let start = std::sync::Barrier::new(32);
    let finish = std::sync::Barrier::new(33);
    let (send, receive) = std::sync::mpsc::channel();
    std::thread::scope(|scope| {
        for _ in 0..32 {
            let (publication, start, finish, send) = (&publication, &start, &finish, send.clone());
            scope.spawn(move || {
                start.wait();
                let result = publication.with_execution_placement(|_| {
                    send.send(true).unwrap();
                    finish.wait();
                });
                if result.is_err() {
                    send.send(false).unwrap();
                    finish.wait();
                }
                assert!(result.is_ok());
            });
        }
        let admitted = (0..32).map(|_| receive.recv().unwrap()).collect::<Vec<_>>();
        let close = court.world.try_preflight_close_admission(&execution);
        finish.wait();
        assert!(admitted.into_iter().all(|admitted| admitted));
        assert!(matches!(
            close,
            Err(RuntimeWorldCloseDenial::InFlightCriticalSection)
        ));
    });
    court
        .world
        .try_preflight_close_admission(&execution)
        .unwrap();
    drop(root);
    court.finish();
}
#[test]
fn placement_scope_releases_its_original_entry_after_return_and_unwind() {
    let court = CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let policy = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&policy);
    let publication = court.world.publication_port();
    publication.with_execution_placement(|_| ()).unwrap();
    court
        .world
        .try_preflight_close_admission(&execution)
        .unwrap();
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        publication.with_execution_placement::<()>(|_| panic!("original installed-call unwind"))
    }));
    assert!(panic.is_err());
    court
        .world
        .try_preflight_close_admission(&execution)
        .unwrap();
    publication.with_execution_placement(|_| ()).unwrap();
    drop(root);
    court.assert_no_pending_attempts();
    court
        .world
        .try_prepare_close_admission(&execution)
        .unwrap()
        .begin();
    assert!(publication
        .with_execution_placement(|_| panic!("closing call ran"))
        .is_err());
    assert!(court
        .world
        .observation_port()
        .observe_root_product_branch()
        .is_err());
    court.finish_admitted_close();
}

#[test]
fn public_calls_refuse_unperformed_bootstrap_and_serve_after_bootstrap() {
    let court = CompositeSupplyChainCourt::compile();
    let publication = court.world.publication_port();
    assert!(publication
        .with_execution_placement(|_| panic!("unperformed call ran"))
        .is_err());
    let root = court.bootstrap();
    publication.with_execution_placement(|_| ()).unwrap();
    drop(root);
    court.finish();
}
