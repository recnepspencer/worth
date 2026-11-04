use super::*;
use super::{protocol::Request, session::Session};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
#[test]
fn bounded_host_oracle_seed_zero_continues_across_real_process_reopens() {
    run(0);
}
#[test]
fn bounded_host_oracle_seed_one_continues_across_real_process_reopens() {
    run(1);
}
fn run(seed: u64) {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("home");
    let script = HostScript::generate(seed);
    let request = opening::serial();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let initialized = Arc::new(AtomicUsize::new(0));
    let contacts = Arc::new(AtomicUsize::new(0));
    let reference = host_observation::opening::open(
        worth_query_host::facade::WorthQueryApplicationHomeLocation::MemoryOnly,
        opening::build(),
        initialized.clone(),
        contacts.clone(),
        None,
        Default::default(),
        &execution,
    );
    let mut map = host_observation::comparison::HostOracleComparison::default();
    let mut originals = std::collections::HashMap::new();
    let (service, opening) = Session::start(&path, &root.path().join("session-0"), seed);
    let mut session = Some(service);
    let mut generation = 0;
    let mut directory_contacts = opening.contacts;
    host_observation::comparison::compare(
        &mut map,
        opening.state.as_ref().unwrap(),
        &host_observation::projection::observe(reference.runtime(), &execution),
    )
    .unwrap_or_else(|cause| {
        panic!(
            "seed {seed} initial prefix: {cause:?}; Home={:?}",
            opening.state
        )
    });
    let reference_head = reference
        .runtime()
        .runtime()
        .application_oracle_head_for_test();
    assert!(
        reference_head.durable().is_none(),
        "ordinary MemoryOnly reference"
    );
    drop(reference_head);
    for (index, command) in script.commands().iter().enumerate() {
        if let Some(owner) = command.activation().owner_merge() {
            println!("INACTIVE seed={seed} step={index} owner={owner} command={command:?}");
            continue;
        }
        println!("HOST_ORACLE seed={seed} step={index} command={command:?}");
        if matches!(command, HostCommand::Reopen) {
            if let Some(mut old) = session.take() {
                old.kill();
            }
            generation += 1;
            let (next, opened) = Session::start(
                &path,
                &root.path().join(format!("session-{generation}")),
                seed,
            );
            session = Some(next);
            directory_contacts = opened.contacts;
            assert_eq!(
                opened.initialized, 0,
                "reopen does not replay initialization"
            );
            host_observation::comparison::compare(&mut map,opened.state.as_ref().unwrap(),&host_observation::projection::observe(reference.runtime(),&execution)).unwrap_or_else(|cause|panic!("seed {seed} step {index} recovered prefix: {cause:?}; Home={:?}; reference={:?}",opened.state,host_observation::projection::observe(reference.runtime(),&execution)));
            continue;
        }
        let contacts_before = contacts.load(Ordering::SeqCst);
        let expected = super::execution::mutation(&reference, command, &mut originals);
        let expected_disposition = super::execution::disposition(&reference, command);
        let actual = session
            .as_mut()
            .unwrap()
            .send(if matches!(command, HostCommand::Open) {
                Request::Observe
            } else {
                Request::Step(index)
            });
        assert_eq!(
            actual.contacts - directory_contacts,
            contacts.load(Ordering::SeqCst) - contacts_before,
            "seed {seed} step {index}: an original retry must not run the handler"
        );
        directory_contacts = actual.contacts;
        match (&actual.mutation, &expected) {
            (None, None) => (),
            (Some(h), Some(r)) => host_observation::receipts::compare(map.identities_mut(), h, r)
                .unwrap_or_else(|cause| {
                    panic!(
                        "seed {seed} step {index} receipt: {cause:?}; Home={h:?}; reference={r:?}"
                    )
                }),
            (None, Some(_)) | (Some(_), None) => {
                panic!("seed {seed} step {index} response fate presence")
            }
        }
        assert_eq!(
            actual.disposition, expected_disposition,
            "seed {seed} step {index} original custody disposition"
        );
        if matches!(command, HostCommand::Initialize) {
            assert_eq!(
                (actual.initialized, initialized.load(Ordering::SeqCst)),
                (1, 1)
            );
        }
        if matches!(command, HostCommand::Close) {
            assert!(actual.closed);
            session.take().unwrap().child.wait().unwrap();
        } else {
            host_observation::comparison::compare(
                &mut map,
                actual.state.as_ref().unwrap(),
                &host_observation::projection::observe(reference.runtime(), &execution),
            )
            .unwrap_or_else(|cause| {
                panic!(
                    "seed {seed} step {index} state: {cause:?}; Home={:?}; reference={:?}",
                    actual.state,
                    host_observation::projection::observe(reference.runtime(), &execution)
                )
            });
        }
    }
}
