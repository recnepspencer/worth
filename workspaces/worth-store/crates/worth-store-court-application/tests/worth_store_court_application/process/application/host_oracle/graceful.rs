//! H3 graceful oracle; normal Store process lane, seeds 0..2.
//! Signal and changed-build dimensions extend this owner prefix in H5/H6.
use super::{opening, protocol::Request, session::Session, *};
use host_observation::{comparison, projection, receipts};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
mod script;

#[test]
fn graceful_home_oracle_seed_zero_preserves_relational_and_world_prefixes() {
    run(0);
}
#[test]
fn graceful_home_oracle_seed_one_preserves_relational_and_world_prefixes() {
    run(1);
}
fn run(seed: u64) {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("home");
    let request = opening::serial();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let initialized = Arc::new(AtomicUsize::new(0));
    let contacts = Arc::new(AtomicUsize::new(0));
    // Ordinary MemoryOnly host: no durable binding, recovered frontiers or reset.
    let reference = host_observation::opening::open(
        worth_query_host::facade::WorthQueryApplicationHomeLocation::MemoryOnly,
        opening::build(),
        initialized.clone(),
        contacts.clone(),
        None,
        Default::default(),
        &execution,
    );
    let mut correspondence = comparison::HostOracleComparison::default();
    let mut reference_originals = std::collections::HashMap::new();
    let (mut session, opened) = Session::start(&path, &root.path().join("session-0"), seed);
    assert_eq!(opened.initialized, 1);
    let mut directory_contacts = opened.contacts;
    let mut reference_state = projection::observe(reference.runtime(), &execution);
    comparison::compare(
        &mut correspondence,
        opened.state.as_ref().unwrap(),
        &reference_state,
    )
    .unwrap();
    for (index, step) in script::generate(seed).into_iter().enumerate() {
        println!("GRACEFUL_ORACLE seed={seed} prefix={index} operation={step:?}");
        let before_contacts = contacts.load(Ordering::SeqCst);
        let expected = super::execution::mutation(
            &reference,
            &HostCommand::Mutate {
                branch: BranchLabel::Main,
                key: step.key,
                value: step.value,
            },
            &mut reference_originals,
        )
        .unwrap();
        let actual = session.send(Request::Mutate {
            key: step.key,
            value: step.value,
        });
        let actual_mutation = actual.mutation.as_ref().unwrap();
        let expected_fate = match step.outcome {
            script::Outcome::Committed => receipts::MutationFate::Committed,
            script::Outcome::IntentDrift => receipts::MutationFate::IntentDrift,
        };
        assert_eq!(
            actual_mutation.fate, expected_fate,
            "Home seed {seed} prefix {index}"
        );
        assert_eq!(
            expected.fate, expected_fate,
            "Memory seed {seed} prefix {index}"
        );
        assert_eq!(actual.contacts - directory_contacts, step.handler_calls);
        assert_eq!(
            contacts.load(Ordering::SeqCst) - before_contacts,
            step.handler_calls
        );
        reference_state = projection::observe(reference.runtime(), &execution);
        let actual_state = actual.state.as_ref().unwrap();
        receipts::compare(
            correspondence.identities_mut(),
            actual_mutation,
            &expected,
            actual_state,
            &reference_state,
        )
        .unwrap_or_else(|cause| panic!("seed {seed} prefix {index} receipt: {cause:?}"));
        comparison::compare(&mut correspondence, actual_state, &reference_state)
            .unwrap_or_else(|cause| panic!("seed {seed} prefix {index} owner state: {cause:?}"));
        match step.restart {
            script::Restart::Close => {
                assert!(session.send(Request::Close).closed);
                session.finish();
            }
            script::Restart::QuiescentKill => session.kill(),
        }
        let (next, recovered) = Session::start(
            &path,
            &root.path().join(format!("session-{}", index + 1)),
            seed,
        );
        session = next;
        assert_eq!(recovered.initialized, 0, "initialization must not replay");
        assert_eq!(recovered.contacts, 0, "opening must not replay a handler");
        comparison::compare(
            &mut correspondence,
            recovered.state.as_ref().unwrap(),
            &reference_state,
        )
        .unwrap_or_else(|cause| {
            panic!("seed {seed} prefix {index} reopened owner state: {cause:?}")
        });
        directory_contacts = recovered.contacts;
    }
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    assert!(session.send(Request::Close).closed);
    session.finish();
    // bed4f954f5d / build-an-application §4.2: this close proves Directory
    // retirement only. The ordinary Memory reference has no physical Home.
    let refused = reference
        .close(&execution)
        .err()
        .expect("Memory close retains its documented lifecycle refusal");
    assert!(refused.directory_denial().is_none());
    assert!(refused.query_denial().is_none());
    let reference = refused.resume().unwrap();
    assert!(!reference.is_directory());
    drop(reference);
}
