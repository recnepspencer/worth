//! A stopped non-atomic set resumes only its ordered suffix after Home reopen.
use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement_from_request,
    application_entry::WorthQueryApplicationRequestExt,
    application_installation::{
        ApplicationBuildIdentity, WorthQueryApplicationProgramRoster, WorthQueryProgramOwner,
    },
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    actor_generation::validated_actor_program,
    host::{host_limits, seed_host},
    model_lifecycle::validated_model_lifecycle_program,
    operator_identity::{authenticate_operator, request_scope},
    presented_request::set_retention,
    programs::{validated_first_program, validated_second_program, RetentionProgramP1},
    readback::read_retention,
    schema::DocumentRetentionSchema,
    settled_verdict::{settle, RetentionVerdict},
};

#[test]
fn directory_stopped_adoption_keeps_prefix_and_reprepares_only_the_suffix() {
    use worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock;
    use worth_query_host::facade::application_entry::WorthQueryBranchSetAdoptionProgress as Progress;
    let directory = worth_query_testkit_host::facade::host::application_home::location();
    let lease = worth_query_testkit_host::facade::host::checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let open = |initialize: bool| {
        let request = WorthQueryApplicationHomeRequest::new(
            WorthQueryApplicationHomeLocation::Directory(directory.clone()),
            validated_first_program(),
            WorthQueryApplicationProgramRoster::new()
                .support(validated_second_program())
                .support(validated_actor_program())
                .support(validated_model_lifecycle_program()),
            DocumentRetentionSchema::declaration().unwrap(),
            ((),),
            ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
            host_limits(WorthQueryAdmissionClock::system()),
            move |phase, graph, schema| {
                assert!(initialize);
                seed_host(phase, graph, schema)
            },
        );
        with_bootstrap_advancement_from_request(execution, |phase| {
            Box::new(request.open_in_advancement(&phase).unwrap())
        })
        .unwrap()
    };
    let home = open(true);
    let prefix_commit = {
        let host = home.runtime();
        let a = host.current_world();
        use worth_query_host::facade::product::{
            WorthQueryProductBranchCreateError as CreateError,
            WorthQueryProductBranchCreationContinuation as Continuation,
            WorthQueryProductBranchRecoveryDenial as RecoveryDenial,
        };
        let fork = || {
            host.with_host_advancement(|phase| {
                match host
                    .runtime()
                    .branches()
                    .fork(a)
                    .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                    .create(&phase)
                {
                    Ok(branch) => branch,
                    Err(CreateError::ProductUnpublished(mut recovery)) => {
                        let deadline =
                            std::time::Instant::now() + std::time::Duration::from_secs(10);
                        loop {
                            assert!(
                                std::time::Instant::now() < deadline,
                                "original Directory fork must settle"
                            );
                            match recovery.continue_in_advancement(&phase) {
                                Ok(Continuation::Performed(branch)) => break branch,
                                Ok(other) => panic!("fork terminal: {other:?}"),
                                Err(failure) => {
                                    assert_eq!(failure.denial(), RecoveryDenial::Busy);
                                    recovery = failure.into_recovery();
                                    std::thread::yield_now();
                                }
                            }
                        }
                    }
                    Err(cause) => panic!("fork: {cause:?}"),
                }
            })
            .unwrap()
        };
        let b = fork();
        let c = fork();
        let scope = request_scope();
        let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
        let target = *host
            .supported_program::<RetentionProgramP1>()
            .unwrap()
            .owned_revision();
        let coverage = host
            .runtime()
            .branches()
            .program_adoption_coverage(&[a, b, c], std::num::NonZeroUsize::new(3).unwrap())
            .unwrap();
        let mut set = host
            .runtime()
            .request(&principal, &scope)
            .on_branches(coverage, &[a, b, c])
            .unwrap()
            .programs()
            .adopt(&target)
            .unwrap()
            .prepare(64)
            .unwrap();
        assert_eq!(
            settle(Ok(
                set_retention(host, b, 8, 1).expect("source edit before adoption must admit")
            )),
            RetentionVerdict::Performed(8)
        );
        let Some(Progress::Performed { branch, adoption }) =
            set.advance(host.runtime(), &principal, &scope).unwrap()
        else {
            panic!("first branch publishes");
        };
        assert_eq!(*branch, a);
        let commit = adoption.product_commit();
        let prefix = (commit.owner_identity().durable().unwrap(), commit.ordinal());
        let Some(Progress::ProductUnpublished { branch, adoption }) =
            set.advance(host.runtime(), &principal, &scope).unwrap()
        else {
            panic!("the actual second head moved");
        };
        assert_eq!(*branch, b);
        assert_eq!(
            adoption.before_effect_refusal(),
            Some(worth_query_host::facade::runtime::NoEffectCause::StaleExpectedProductHead)
        );
        let stopped = set
            .begin_resume(host.runtime(), &scope)
            .unwrap_or_else(|(cause, _)| panic!("acknowledged stop: {cause:?}"));
        assert_eq!(stopped.remaining_branches(), &[b, c]);
        drop(stopped);
        prefix
    };
    drop(home); // No Native effect is left without a World occurrence.
    let home = open(false);
    {
        let host = home.runtime();
        let scope = request_scope();
        let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
        let mut stopped = host
            .with_host_advancement(|phase| {
                host.runtime()
                    .readmit_stopped_branch_set_adoptions(&phase)
                    .unwrap()
            })
            .unwrap();
        assert_eq!(stopped.len(), 1);
        let stopped = stopped.pop().unwrap();
        let [Progress::RecordedPerformed { commit, .. }, Progress::RecordedNoEffect { .. }] =
            stopped.progress()
        else {
            panic!("historical prefix and stop stay descriptive");
        };
        assert_eq!((commit.owner(), commit.ordinal()), prefix_commit);
        let remaining = stopped.remaining_branches().to_vec();
        assert_eq!(remaining.len(), 2);
        let widened = host
            .runtime()
            .branches()
            .program_adoption_coverage(
                &[host.current_world(), remaining[0], remaining[1]],
                std::num::NonZeroUsize::new(3).unwrap(),
            )
            .unwrap();
        let failure = host
            .runtime()
            .request(&principal, &scope)
            .on_branches(widened, &[host.current_world(), remaining[0], remaining[1]])
            .unwrap()
            .programs()
            .resume(stopped, 64)
            .err()
            .expect("fresh coverage cannot repeat the completed prefix");
        assert!(matches!(failure.denial(),worth_query_host::facade::application_entry::WorthQueryBranchSetAdoptionResumeDenial::CoverageMismatch));
        let stopped = failure.into_adoption();
        let coverage = host
            .runtime()
            .branches()
            .program_adoption_coverage(&remaining, std::num::NonZeroUsize::new(2).unwrap())
            .unwrap();
        let mut set = host
            .runtime()
            .request(&principal, &scope)
            .on_branches(coverage, &remaining)
            .unwrap()
            .programs()
            .resume(stopped, 64)
            .unwrap_or_else(|failure| panic!("fresh suffix: {:?}", failure.denial()));
        assert_eq!(set.progress().len(), 1);
        for expected in &remaining {
            assert!(
                matches!(set.advance(host.runtime(),&principal,&scope).unwrap(),Some(Progress::Performed { branch,.. }) if branch == expected)
            );
        }
        let closed = set.close().unwrap_or_else(|_| panic!("suffix completes"));
        assert_eq!(closed.progress().len(), 3);
        assert!(
            matches!(&closed.progress()[0],Progress::RecordedPerformed { commit,.. } if (commit.owner(),commit.ordinal()) == prefix_commit)
        );
        assert!(host
            .with_host_advancement(|phase| host
                .runtime()
                .readmit_stopped_branch_set_adoptions(&phase)
                .unwrap())
            .unwrap()
            .is_empty());
        for (key, branch) in std::iter::once(host.current_world())
            .chain(remaining)
            .enumerate()
        {
            assert_eq!(
                settle(Ok(set_retention(host, branch, 15, 10 + key as u64)
                    .unwrap_or_else(|cause| panic!(
                        "post-resume branch {key} mutation must admit: {cause:?}"
                    )))),
                RetentionVerdict::Performed(15)
            );
        }
    }
    drop(home);
    let home = open(false);
    let host = home.runtime();
    assert!(host
        .with_host_advancement(|phase| host
            .runtime()
            .readmit_stopped_branch_set_adoptions(&phase)
            .unwrap())
        .unwrap()
        .is_empty());
    assert_eq!(read_retention(host.runtime(), host.current_world()), 15);
}
