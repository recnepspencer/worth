//! The operative durable boundary refuses a divergent correction before effects.
use super::*;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationCorrectionDenial as Entry,
    primary_graph::WorthQueryCorrectionDerivationDenial as Derivation,
    product::{
        WorthQueryDurableStateAdmissionDenial as Durable, WorthQueryProductBranch,
        WorthQueryProductBranchCreateError as Fork,
        WorthQueryProductBranchCreationContinuation as Continued,
        WorthQueryProductBranchRecoveryDenial,
    },
};
#[test]
fn directory_divergent_correction_refuses_at_the_operative_owner_before_effect() {
    let path = fixture::location();
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open(path.clone(), execution);
    let original = edit(&home);
    let coordinates = original.correction_source().coordinates();
    drop(original);
    {
        let scope = scope();
        let runtime = home.runtime().runtime();
        let before = runtime.current_world();
        let child = fork(&home);
        let principal = label::authentication::authenticate_external(
            home.runtime().installed_schema(),
            "alice",
            Duration::from_secs(120),
            &scope,
        );
        let request = runtime.request(&principal, &scope).on_branch(child);
        let limits =
            WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::new(8).unwrap());
        let source = request
            .corrections()
            .select_coordinates(&coordinates, limits)
            .unwrap();
        let denial = request
            .corrections()
            .reverse(&source)
            .idempotency("divergent-refusal")
            .limits(limits)
            .prepare()
            .err()
            .expect("divergent continuation is not admitted");
        assert!(matches!(
            denial,
            Entry::Derivation(Derivation::DurableStateAdmission(
                Durable::UnsupportedDivergentCorrection
            ))
        ));
        assert_eq!(runtime.current_world(), before);
    }
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("no divergent effect: {cause:?}"))
        .reclaim();
    let home = open(path, execution);
    let scope = scope();
    let principal = label::authentication::authenticate_external(
        home.runtime().installed_schema(),
        "alice",
        Duration::from_secs(120),
        &scope,
    );
    let rows = home
        .runtime()
        .request(&principal, &scope)
        .query(label::AccountSummaryParameters)
        .execute()
        .unwrap();
    assert_eq!(rows.rows()[0].label, "changed");
    drop(rows);
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("preserved original close: {cause:?}"))
        .reclaim();
}

fn fork(home: &Home) -> WorthQueryProductBranch {
    let runtime = home.runtime().runtime();
    let request = scope();
    let created = runtime
        .with_application_advancement(&request, |phase| {
            runtime
                .branches()
                .fork(runtime.current_world())
                .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
        })
        .expect("the Home admits branch creation");
    let mut original = match created {
        Ok(child) => return child,
        Err(Fork::ProductUnpublished(original)) => original,
        Err(cause) => panic!("unexpected fork setup refusal: {cause:?}"),
    };
    let deadline = Instant::now() + Duration::from_secs(120);
    loop {
        assert!(Instant::now() < deadline, "the original fork must settle");
        let request = scope();
        let answer = runtime
            .with_application_advancement(&request, |phase| {
                original.continue_in_advancement(&phase)
            })
            .expect("the Home admits its original fork continuation");
        match answer {
            Ok(Continued::Performed(child)) => return child,
            Ok(Continued::NoEffect(cause)) => panic!("the original fork refused: {cause:?}"),
            Ok(Continued::ReleasedOriginal(cause)) => {
                panic!("the original fork released: {cause:?}")
            }
            Err(failure) => {
                assert_eq!(
                    failure.denial(),
                    WorthQueryProductBranchRecoveryDenial::Busy,
                    "original fork continuation: {failure:?}"
                );
                original = failure.into_recovery();
                std::thread::yield_now();
            }
        }
    }
}
