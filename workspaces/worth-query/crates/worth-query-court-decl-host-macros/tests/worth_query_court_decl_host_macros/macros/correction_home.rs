//! The installed body inverse uses the same retained completion after reopen
//! and refuses cleanly after a conflicting edit.
use super::{
    actions::Complete,
    completion_correction::{CorrectCompletion, ReadDetails, RestoreDoneBinding, RestoredDone},
    program::*,
    schema::Schema,
};
use std::{
    num::NonZeroUsize,
    path::{Path, PathBuf},
};
use worth_query_decl::facade::{
    application_program::{
        ApplicationActionReference, ApplicationProgramAuthoring, ApplicationRootComposition,
    },
    application_schema::ApplicationSchema,
};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    application_contribution::with_bootstrap_advancement,
    application_entry::{
        WorthQueryApplicationCorrectionDenial, WorthQueryApplicationCorrectionPreparation,
        WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequest,
        WorthQueryApplicationRequestExt, WorthQueryCorrectionHistoryLimits,
        WorthQueryOperationCorrectionPreparation, WorthQuerySourceCompletion,
        WorthQuerySourceOnlyAdvance,
    },
    application_installation::{
        ApplicationBuildIdentity, WorthQueryApplicationLimits, WorthQueryApplicationProgramRoster,
    },
    primary_graph::{ExecutionRequest, WorthQueryCorrectionDerivationDenial},
    runtime::{
        CancellationToken, ExecutionAllocationPolicy::SystemAllocation, ExecutionResourceLease,
        LeaseRequest,
    },
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};

type Home = WorthQueryApplicationHome<Schema, Program>;
type RestoreAction = ApplicationActionReference<
    Schema,
    ApplicationRootComposition,
    BoardFeature,
    RestoreDoneBinding,
    RestoredDone,
>;

#[test]
fn recorded_authored_body_inverse_recovers_its_completed_command_after_reopen() {
    use WorthQueryApplicationMutationOutcome as Outcome;
    use WorthQueryOperationCorrectionPreparation as Prepared;
    use WorthQuerySourceOnlyAdvance as Advance;

    let path = temp_home_path("body-inverse-replay");
    let clock = WorthQueryAdmissionClock::system();
    let home = open(&path, &clock, true);
    let coordinate = {
        let host = home.runtime();
        let scope = scope(&clock);
        let principal = owner(host, &clock, &scope);
        let request = host.request(&principal, &scope);
        let outcome = request
            .advanced()
            .mutate(Complete {
                task: 3,
                done: true,
            })
            .without_source()
            .idempotency(&1)
            .execute_in_program(host, SystemAllocation)
            .unwrap();
        let Outcome::Committed {
            receipt,
            result: (),
        } = outcome
        else {
            panic!("the authored source completion commits");
        };
        assert!(task_done(&request));
        receipt.correction_source().coordinates()
    };
    close(home);

    let limits = WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::MIN);
    let home = open(&path, &clock, false);
    {
        let host = home.runtime();
        let scope = scope(&clock);
        let principal = owner(host, &clock, &scope);
        let request = host.request(&principal, &scope);
        assert!(task_done(&request));
        let corrections = request.corrections();
        let source = corrections.select_coordinates(&coordinate, limits).unwrap();
        let inverse = corrections.reverse(&source).idempotency("2").limits(limits);
        let Prepared::Prepared(inverse) = inverse
            .prepare_operation::<CorrectCompletion, Program, RestoreAction>(host)
            .unwrap()
        else {
            panic!("the first inverse prepares under its installed action");
        };
        let Advance::Completed(completed) = inverse.publish().unwrap() else {
            panic!("the inverse body commits");
        };
        assert!(matches!(
            completed.evidence(),
            WorthQuerySourceCompletion::Fresh { result: (), .. }
        ));
        completed
            .close(&request)
            .unwrap_or_else(|(_, denial)| panic!("{denial:?}"));
        assert!(!task_done(&request));
    }
    close(home);

    let home = open(&path, &clock, false);
    {
        let host = home.runtime();
        let scope = scope(&clock);
        let principal = owner(host, &clock, &scope);
        let request = host.request(&principal, &scope);
        let corrections = request.corrections();
        let source = corrections.select_coordinates(&coordinate, limits).unwrap();
        let inverse = corrections.reverse(&source).idempotency("2").limits(limits);
        let Prepared::Continued(Advance::Completed(completed)) = inverse
            .prepare_operation::<CorrectCompletion, Program, RestoreAction>(host)
            .unwrap()
        else {
            panic!("the retry recovers the native completion owner");
        };
        assert!(matches!(
            completed.evidence(),
            WorthQuerySourceCompletion::Recorded { .. }
        ));
        completed
            .close(&request)
            .unwrap_or_else(|(_, denial)| panic!("{denial:?}"));
        let replay = request
            .corrections()
            .reverse(&source)
            .idempotency("2")
            .limits(limits)
            .prepare()
            .unwrap();
        assert!(matches!(
            replay,
            WorthQueryApplicationCorrectionPreparation::Settled(Outcome::RecoveredCommitted(_))
        ));
        assert!(!task_done(&request));
    }
    close(home);
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn recorded_authored_body_inverse_refuses_after_conflicting_edit_and_preserves_state_on_reopen() {
    use WorthQueryApplicationMutationOutcome as Outcome;

    let path = temp_home_path("body-inverse-conflict");
    let clock = WorthQueryAdmissionClock::system();
    let limits = WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::new(4).unwrap());
    let home = open(&path, &clock, true);
    {
        let host = home.runtime();
        let scope = scope(&clock);
        let principal = owner(host, &clock, &scope);
        let request = host.request(&principal, &scope);
        let Outcome::Committed {
            receipt,
            result: (),
        } = request
            .advanced()
            .mutate(Complete {
                task: 3,
                done: true,
            })
            .without_source()
            .idempotency(&1)
            .execute_in_program(host, SystemAllocation)
            .unwrap()
        else {
            panic!("the authored source completion commits");
        };
        assert!(task_done(&request));
        let coordinate = receipt.correction_source().coordinates();

        // Conflicting edit supersedes the recorded completion on the same field.
        let Outcome::Committed { result: (), .. } = request
            .advanced()
            .mutate(Complete {
                task: 3,
                done: false,
            })
            .without_source()
            .idempotency(&2)
            .execute_in_program(host, SystemAllocation)
            .unwrap()
        else {
            panic!("the conflicting edit commits");
        };
        assert!(!task_done(&request));

        let corrections = request.corrections();
        let source = corrections.select_coordinates(&coordinate, limits).unwrap();
        let refusal = corrections
            .reverse(&source)
            .idempotency("3")
            .limits(limits)
            .prepare_operation::<CorrectCompletion, Program, RestoreAction>(host)
            .err()
            .expect("conflicting edit must refuse the recorded authored-body inverse");
        assert!(
            matches!(
                refusal,
                WorthQueryApplicationCorrectionDenial::Derivation(
                    WorthQueryCorrectionDerivationDenial::SourceDiverged
                )
            ),
            "conflicting edit must refuse with SourceDiverged: {refusal:?}"
        );
        assert!(!task_done(&request));
    }
    close(home);

    let home = open(&path, &clock, false);
    {
        let host = home.runtime();
        let scope = scope(&clock);
        let principal = owner(host, &clock, &scope);
        let request = host.request(&principal, &scope);
        assert!(
            !task_done(&request),
            "refused conflicting inverse leaves no partial effect across reopen"
        );
    }
    close(home);
    std::fs::remove_dir_all(path).unwrap();
}

fn temp_home_path(label: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(label);
    path.set_extension(std::process::id().to_string());
    let _ = std::fs::remove_dir_all(&path);
    path
}

fn open(path: &Path, clock: &WorthQueryAdmissionClock, fresh: bool) -> Home {
    let declaration = Schema::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<Schema, Program>::begin()
        .with_schema(&declaration)
        .validated_program()
        .unwrap();
    let request = WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(path.to_owned()),
        program,
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        (BoardRuntime::new(),),
        ApplicationBuildIdentity::parse("5a5a5a5a-5a5a-5a5a-5a5a-5a5a5a5a5a5a").unwrap(),
        WorthQueryApplicationLimits::bounded_serial(clock.clone())
            .install()
            .unwrap(),
        move |_, graph, installed| {
            assert!(fresh, "reopen must keep the original board");
            seed(graph, installed)
        },
    );
    with_bootstrap_advancement(
        WorthQueryApplicationLimits::SERIAL_EXECUTION_POLICY,
        |phase| request.open_in_advancement(&phase),
    )
    .unwrap()
    .unwrap()
}

fn close(home: Home) {
    let lease = ExecutionResourceLease::serial(LeaseRequest {
        policy: WorthQueryApplicationLimits::SERIAL_EXECUTION_POLICY,
        cancellation: CancellationToken::new(),
        deadline: None,
    })
    .expect("serial close lease");
    let execution = ExecutionRequest::leased(&lease);
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("close Directory Home: {refused:?}"))
        .reclaim();
}

fn task_done(request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>) -> bool {
    request.query(ReadDetails(3)).execute().unwrap().rows()[0].done
}
