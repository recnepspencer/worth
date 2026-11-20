//! Every real branch retains its own activation, rather than inheriting main's.
use super::*;

fn fork(
    runtime: &worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<fixture::schema::HomeIdentitySchema, fixture::HomeProgram>,
) {
    let (_, scope) = super::super::pending_support::scope();
    let application = runtime.runtime();
    let created = application
        .with_application_advancement(&scope, |phase| {
            application
                .branches()
                .fork(application.current_world())
                .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
        })
        .unwrap();
    use worth_query_host::facade::product::{
        WorthQueryProductBranchCreateError as Create,
        WorthQueryProductBranchCreationContinuation as Continued,
        WorthQueryProductBranchRecoveryDenial,
    };
    let mut original = match created {
        Ok(_) => return,
        Err(Create::ProductUnpublished(original)) => original,
        Err(cause) => panic!("durable sibling creation: {cause:?}"),
    };
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    loop {
        assert!(
            std::time::Instant::now() < deadline,
            "the original sibling creation must settle"
        );
        let (_, scope) = super::super::pending_support::scope();
        let answer = application
            .with_application_advancement(&scope, |phase| original.continue_in_advancement(&phase))
            .expect("the Home admits its original sibling continuation");
        match answer {
            Ok(Continued::Performed(_)) => return,
            Ok(Continued::NoEffect(cause)) => panic!("the original sibling refused: {cause:?}"),
            Ok(Continued::ReleasedOriginal(cause)) => {
                panic!("the original sibling released: {cause:?}")
            }
            Err(failure) => {
                assert_eq!(
                    failure.denial(),
                    WorthQueryProductBranchRecoveryDenial::Busy,
                    "original sibling continuation: {failure:?}"
                );
                original = failure.into_recovery();
                std::thread::yield_now();
            }
        }
    }
}

#[test]
fn directory_reopen_validates_rostered_activation_on_every_unadopted_branch() {
    let path = fixture::location();
    let build = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
    let initialized = Arc::new(AtomicUsize::new(0));
    let contacts = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let request = fixture::request(path.clone(), build, initialized.clone(), contacts.clone());
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            request
                .open_in_advancement(&phase)
                .unwrap_or_else(|cause| panic!("A open: {cause:?}")),
        )
    })
    .unwrap();
    fork(home.runtime());
    (*home)
        .close(&execution)
        .unwrap_or_else(|cause| panic!("A close: {cause:?}"));
    let request = fixture::request(path, build, initialized.clone(), contacts.clone());
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            request
                .open_in_advancement(&phase)
                .unwrap_or_else(|cause| panic!("all-branch reopen: {cause:?}")),
        )
    })
    .unwrap();
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
    (*home)
        .close(&execution)
        .unwrap_or_else(|cause| panic!("reopened A close: {cause:?}"));
}

#[test]
fn directory_open_refuses_an_unrostered_activation_on_a_non_main_branch() {
    use worth_query_execution::facade::application_home_integration::QueryHomeCustodyReadmissionDenial;
    use worth_query_host::facade::{
        HomeCustodyReadmissionDenial, QueryHomeProgramActivationDenial,
        WorthQueryApplicationHomeOpenRefused,
    };
    let path = fixture::location();
    let build = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
    let initialized = Arc::new(AtomicUsize::new(0));
    let contacts = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let request = fixture::request(path.clone(), build, initialized.clone(), contacts.clone());
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            request
                .open_in_advancement(&phase)
                .unwrap_or_else(|cause| panic!("A open: {cause:?}")),
        )
    })
    .unwrap();
    fork(home.runtime());
    (*home)
        .close(&execution)
        .unwrap_or_else(|cause| panic!("A close: {cause:?}"));
    let before =
        fixture::request_with_initializer(path.clone(), build, contacts.clone(), |_, _, _| {
            panic!("no initializer on reopen")
        });
    // Transition only main to B. The genuine untouched sibling still activates
    // A, which is absent from B's supplied roster. Neither ID is fabricated.
    let request = successor(path, build, contacts.clone()).with_program_transition(
        before,
        resources(),
        |source, target| {
            let rows = source.entities("Account", 10_000)?;
            let [row] = rows.records() else {
                panic!("one authored Account");
            };
            let after = target
                .planned_field(
                    "Account",
                    FieldSelection::<HomeRecords, Account, AccountIdentity, u64>::at("Balance")
                        .aspect(),
                    "Balance",
                )
                .unwrap();
            let mut step = body(row.entity_id, after, 9);
            step.batches.push(successor::body(target));
            Ok(Plan {
                target: 0,
                schema_transition: Some((successor::schema_change(source, target), None)),
                steps: vec![step],
            })
        },
    );
    let refused = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .err()
    .expect("non-main A is not executable under B-only support");
    assert!(
        matches!(
            refused.phase(),
            Refused::Successor(WorthQueryApplicationHomeOpenRefused::CustodyReadmission {
                cause: HomeCustodyReadmissionDenial::Query(
                    QueryHomeCustodyReadmissionDenial::Activation(
                        QueryHomeProgramActivationDenial::UnrosteredCurrentActivation { .. }
                    )
                ),
                ..
            })
        ),
        "expected typed non-main unrostered refusal, got {refused:?}"
    );
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
    // This is refused opening custody, not a claim of successful Closed.
    drop(refused);
}
