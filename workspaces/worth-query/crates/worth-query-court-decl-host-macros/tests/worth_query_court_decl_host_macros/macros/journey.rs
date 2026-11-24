//! The derived board runs through the public Host: a local-process principal,
//! bounded serial limits, parameterized reads and source custody across frames.
use super::actions::*;
use super::program::{
    day, install, owner, scope, BoardFeature, Host, Program, RetitleFirstAction, TASKS,
};
use super::reads::{DueBy, Search, TaskRow, TitleSearch};
use super::relations::{Outline, ProjectKeyOnly};
use super::schema::Schema;
use std::fmt::Debug;
use worth_query_host::facade::{
    admission::authenticated_principal::*,
    application_entry::*,
    application_installation::WorthQueryProgramApplicationRuntime,
    declaration::{
        application_operation::{ApplicationMutationBinding, ApplicationMutationIntent},
        application_program::{
            ApplicationActionReference, ApplicationProgramDefinition, ApplicationRootComposition,
        },
        application_schema::ApplicationValueEncodeDenial,
    },
    domain::ApplicationSchema,
    WorthQuerySourceCustody, WorthQuerySourceCustodyPending, WorthQuerySourceCustodyStep,
};

type Key<S, I> =
    <<I as ApplicationMutationIntent<S>>::Binding as ApplicationMutationBinding<S>>::IdempotencyKey;
type Action<Binding, Delivery> =
    ApplicationActionReference<Schema, ApplicationRootComposition, BoardFeature, Binding, Delivery>;
type Step<I, A> = WorthQuerySourceCustodyStep<Schema, I, Program, A>;

/// Captures any sourceless command's input: one bound.
fn capture<S, I>(
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    intent: I,
    key: Key<S, I>,
) -> WorthQueryOperationDescription<S, I>
where
    S: ApplicationSchema,
    I: WorthQuerySourcelessMutation<S>,
{
    match request
        .mutate(intent)
        .without_source()
        .idempotency_owned(key)
    {
        Ok(described) => described,
        Err((_, _, denial)) => panic!("the command's input is captured: {denial:?}"),
    }
}

/// Captures any sourceless command and resumes it once: one bound per role.
fn submit<S, P, I, A>(
    runtime: &WorthQueryProgramApplicationRuntime<S, P>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    intent: I,
    key: Key<S, I>,
) -> WorthQuerySourceCustodyStep<S, I, P, A>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    I: WorthQuerySourcelessMutation<S>,
    A: WorthQuerySourceCompletionAction<S, I::Binding>,
{
    WorthQuerySourceCustody::Described(capture(request, intent, key)).resume(runtime, request)
}

/// Reads any projected query: one bound.
fn read<S, Q>(
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    intent: Q,
) -> Vec<WorthQueryProjectedValue<S, Q>>
where
    S: ApplicationSchema,
    Q: WorthQueryProjectedQuery<S>,
    WorthQueryProjectedValue<S, Q>: Clone,
{
    request
        .query(intent)
        .execute()
        .expect("the read executes")
        .rows()
        .to_vec()
}

/// Any command's terminal evidence reports why it stopped through one bound.
fn committed<I: ApplicationMutationIntent<Schema>, A>(step: Step<I, A>)
where
    WorthQueryTerminalSourceEvidence<Schema, I>: Debug,
{
    match step {
        WorthQuerySourceCustodyStep::Committed { .. } => {}
        WorthQuerySourceCustodyStep::Declined { evidence, .. } => {
            panic!("the command declined: {evidence:?}")
        }
        WorthQuerySourceCustodyStep::Pending { cause, .. } => panic!("still pending: {cause:?}"),
    }
}

fn keys(rows: Vec<TaskRow>) -> Vec<u64> {
    rows.into_iter().map(|row| row.key).collect()
}

fn search(q: &str) -> Search {
    Search {
        project: 1,
        q: q.to_owned(),
    }
}

#[test]
fn a_derived_board_reads_case_insensitively_and_by_native_due_date() {
    let clock = WorthQueryAdmissionClock::system();
    let (host, scope) = (install(&clock), scope(&clock));
    let principal = owner(&host, &clock, &scope);
    let request = host.runtime().request(&principal, &scope);
    // "FLANGE" matches a title in another case and a note; project 2 is out of scope.
    assert_eq!(keys(read(&request, search("FLANGE"))), [1, 2]);
    assert_eq!(keys(read(&request, search("matte"))), [3]);
    assert_eq!(keys(read(&request, search("absent"))), Vec::<u64>::new());
    // Derived `#[derive(WorthQuery)]` parameterized read matches the same title.
    assert_eq!(
        keys(read(
            &request,
            TitleSearch {
                project: 1,
                q: "housing".to_owned(),
            }
        )),
        [1]
    );
    let due = read(
        &request,
        DueBy {
            project: 1,
            max_due: day(100),
        },
    );
    assert_eq!(
        due,
        [
            row(1, "Mill housing", false, Some(100)),
            row(3, "Paint", false, Some(50)),
        ]
    );
}

#[test]
fn a_derived_projection_reads_related_tasks_and_their_project_in_reverse() {
    let clock = WorthQueryAdmissionClock::system();
    let (host, scope) = (install(&clock), scope(&clock));
    let principal = owner(&host, &clock, &scope);
    let request = host.runtime().request(&principal, &scope);
    let [outline] =
        <[_; 1]>::try_from(read(&request, Outline { project: 1 })).expect("one project outline");
    assert_eq!(outline.key, 1);
    let expected = TASKS
        .iter()
        .filter(|(_, project, ..)| *project == 1)
        .map(|(task, _, title, ..)| (*task, *title))
        .collect::<Vec<_>>();
    let read_tasks = outline
        .tasks
        .iter()
        .map(|task| (task.key(), task.title.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(read_tasks, expected);
    assert!(outline
        .tasks
        .iter()
        .all(|task| task.project == ProjectKeyOnly { key: 1 }));
}

#[test]
fn derived_and_written_actions_commit_through_source_custody_across_frames() {
    let clock = WorthQueryAdmissionClock::system();
    let host: Host = install(&clock);
    let principal = owner(&host, &clock, &scope(&clock));
    let custody = cancelled_before_retention(&host, &principal, &clock);

    // A fresh frame resumes the same custody, then commits further commands.
    let fresh = scope(&clock);
    let request = host.runtime().request(&principal, &fresh);
    committed(custody.resume(&host, &request));
    let later = Reschedule {
        task: 2,
        due: Some(day(80)),
    };
    committed::<_, Action<RescheduleBinding, Rescheduled>>(submit(&host, &request, later, 2));
    let cleared = Reschedule { task: 1, due: None };
    committed::<_, Action<RescheduleBinding, Rescheduled>>(submit(&host, &request, cleared, 3));
    let done = Complete {
        task: 3,
        done: true,
    };
    committed::<_, Action<CompleteBinding, Completed>>(submit(&host, &request, done, 4));
    let fixed = RetitleFirst {
        title: "Mill housing rev2".to_owned(),
    };
    committed::<_, RetitleFirstAction>(submit(&host, &request, fixed, 5));

    assert_eq!(
        keys(read(
            &request,
            TitleSearch {
                project: 1,
                q: "rev2".to_owned(),
            }
        )),
        [1]
    );
    assert_eq!(keys(read(&request, search("bore"))), [2]);
    let due = read(
        &request,
        DueBy {
            project: 1,
            max_due: day(100),
        },
    );
    assert_eq!(
        due,
        [
            row(2, "Bore FLANGE holes", false, Some(80)),
            row(3, "Paint", true, Some(50)),
        ]
    );
}

#[test]
fn a_derived_validator_refuses_a_blank_rename_at_capture_before_any_effect() {
    let clock = WorthQueryAdmissionClock::system();
    let (host, scope) = (install(&clock), scope(&clock));
    let principal = owner(&host, &clock, &scope);
    let request = host.runtime().request(&principal, &scope);
    let blank = Rename {
        task: 2,
        title: "  ".to_owned(),
    };
    let refused = request.mutate(blank).without_source().idempotency_owned(1);
    let Err((_, 1, WorthQueryOperationInputCaptureDenial::Encoding(denial))) = refused else {
        panic!("a blank title is refused at capture with its key returned");
    };
    let ApplicationValueEncodeDenial::Validation(denial) = denial else {
        panic!("the action's validator refused it: {denial:?}");
    };
    assert_eq!(denial.reason(), "a task needs a title");
    assert_eq!(
        denial.binding_identity().as_str(),
        "macros.tasks.rename.v1.input"
    );

    // Nothing was written, and the refused key still admits a valid rename.
    assert_eq!(keys(read(&request, search("drill"))), [2]);
    let valid = Rename {
        task: 2,
        title: "Ream holes".to_owned(),
    };
    committed::<_, Action<RenameBinding, Renamed>>(submit(&host, &request, valid, 1));
    assert_eq!(keys(read(&request, search("drill"))), Vec::<u64>::new());
    assert_eq!(keys(read(&request, search("ream"))), [2]);
}

#[test]
fn terminal_source_evidence_formats_why_a_command_stopped() {
    type Evidence = WorthQueryTerminalSourceEvidence<Schema, Rename>;
    let cancelled = Evidence::Outcome(WorthQueryApplicationMutationOutcome::Cancelled);
    let unprogrammed = Evidence::AdmissionStopped(
        WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired,
    );
    assert_eq!(format!("{cancelled:?}"), "Outcome(Cancelled)");
    assert_eq!(
        format!("{unprogrammed:?}"),
        "AdmissionStopped(ApplicationProgramRequired)"
    );
}

/// A frame captures a rename, then is cancelled before durable retention;
/// the command stays pending with its native retention refusal.
fn cancelled_before_retention(
    host: &Host,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    clock: &WorthQueryAdmissionClock,
) -> WorthQuerySourceCustody<Schema, Rename, Program, Action<RenameBinding, Renamed>> {
    let cancellation = WorthQueryCancellationSource::new();
    let cancelled = WorthQueryRequestScope::new(None, cancellation.token(), clock.clone());
    let request = host.runtime().request(principal, &cancelled);
    let rename = Rename {
        task: 2,
        title: "Bore FLANGE holes".to_owned(),
    };
    let described = capture(&request, rename, 1);
    cancellation.cancel();
    match WorthQuerySourceCustody::Described(described).resume(host, &request) {
        WorthQuerySourceCustodyStep::Pending {
            custody,
            cause: WorthQuerySourceCustodyPending::Retention(_),
        } => custody,
        _ => panic!("a cancelled frame cannot retain the command"),
    }
}

fn row(key: u64, title: &str, done: bool, due: Option<i32>) -> TaskRow {
    TaskRow {
        key,
        title: title.to_owned(),
        done,
        due: due.map(day),
    }
}
