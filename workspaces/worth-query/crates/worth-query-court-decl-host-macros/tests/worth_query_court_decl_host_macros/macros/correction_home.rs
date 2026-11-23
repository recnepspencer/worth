//! The installed body inverse uses the same retained completion after reopen.
use super::{
    actions::Complete,
    completion_correction::{CorrectCompletion, ReadDetails, RestoreDoneBinding, RestoredDone},
    program::*,
    schema::Schema,
};
use std::{num::NonZeroUsize, path::Path};
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
        WorthQueryApplicationCorrectionPreparation, WorthQueryApplicationMutationOutcome,
        WorthQueryApplicationRequest, WorthQueryApplicationRequestExt,
        WorthQueryCorrectionHistoryLimits, WorthQueryOperationCorrectionPreparation,
        WorthQuerySourceCompletion, WorthQuerySourceOnlyAdvance,
    },
    application_installation::{
        ApplicationBuildIdentity, WorthQueryApplicationLimits, WorthQueryApplicationProgramRoster,
    },
    runtime::ExecutionAllocationPolicy::SystemAllocation,
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

    let mut path = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("body-inverse");
    path.set_extension(std::process::id().to_string());
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
    home.close_with_policy(WorthQueryApplicationLimits::SERIAL_EXECUTION_POLICY)
        .unwrap()
        .reclaim();
}

fn task_done(request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>) -> bool {
    request.query(ReadDetails(3)).execute().unwrap().rows()[0].done
}
