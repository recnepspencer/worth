//! Public #368: every stage of a multi-stage envelope is its own correction
//! source on both Homes. One ordered three-stage envelope sets three Numbers
//! `[10, 20, 30]` in one commit; reversing stage `n` restores only Number `n`.
use super::numeric_home::*;
use super::*;
use worth_query_host::facade::{
    application_entry::{WorthQueryApplicationRequestExt, WorthQueryEditEnvelopeOutcome},
    primary_graph::{
        WorthQueryApplicationCommitReceipt, WorthQueryCorrectionSourceCoordinates,
        WorthQueryCorrectionSourceMember,
    },
    WorthQueryAnyCorrectionCustodyOutcome,
    WorthQueryAnySourceCustody, WorthQueryAnySourceCustodyStep,
};
use model::{
    authentication::{authenticate_operator, request_scope},
    correction::RecordedEdit,
    edit::{Edit, Value},
};

type OtherAction = worth_query_decl::facade::application_program::ApplicationActionReference<
    model::schema::NumericSchema,
    worth_query_decl::facade::application_program::ApplicationRootComposition,
    model::program::NumericFeature,
    model::other_edit::OtherBinding,
    model::program::OtherCompletion,
>;

const KEYS: [&str; 3] = ["numeric", "numeric-b", "numeric-c"];
const SET: [u64; 3] = [10, 20, 30];

/// Commits one ordered envelope whose stage `n` sets `KEYS[n]` to `SET[n]`; returns each stage's receipt.
macro_rules! envelope {
    ($home:expr) => {{
        let scope = request_scope();
        let principal = authenticate_operator($home.primary_graph().installed_schema(), &scope);
        let request = $home.runtime().request(&principal, &scope);
        let mut envelope = request
            .edit_envelope()
            .idempotency("three-stage".to_owned())
            .runtime_stages($home.runtime())
            .ordered_authored()
            .unwrap();
        let tokens: Vec<_> = (0..3)
            .map(|n| {
                envelope
                    .push_stage::<worth_query_decl::facade::application_program::ApplicationNoOutputGraph, _>(
                        Edit { key: KEYS[n].to_owned(), value: Value::Primary(SET[n]) },
                    )
                    .unwrap()
            })
            .collect();
        let outcome = envelope
            .execute_performed(worth_execution::ExecutionAllocationPolicy::SystemAllocation)
            .unwrap_or_else(|denial| panic!("three-stage envelope: {denial:?}"));
        let WorthQueryEditEnvelopeOutcome::Committed(done) = outcome else {
            panic!("three-stage envelope must commit: {outcome:?}")
        };
        assert_eq!(done.stage_count(), 3);
        assert_eq!(values($home.runtime()), SET);
        let receipts: Vec<WorthQueryApplicationCommitReceipt> =
            tokens.iter().map(|token| done.stage_receipt(token).unwrap().clone()).collect();
        receipts
    }};
}

/// Reverses one selected stage under `key`; `select` maps the corrections surface to a selection.
macro_rules! reverse {
    ($home:expr, $program:ty, $key:expr, |$corrections:ident| $select:expr) => {{
        let scope = request_scope();
        let principal = authenticate_operator($home.primary_graph().installed_schema(), &scope);
        let request = $home.runtime().request(&principal, &scope);
        let reversed: Result<(), String> = (|| {
            let $corrections = request.corrections();
            let selected = $select.map_err(|denial| format!("select: {denial:?}"))?;
            let prepared = request
                .corrections()
                .reverse(&selected)
                .idempotency($key)
                .limits(limits())
                .prepare_operation::<RecordedEdit, $program, OtherAction>($home.runtime())
                .map_err(|denial| format!("prepare: {denial:?}"))?;
            use WorthQueryAnyCorrectionCustodyOutcome as Outcome;
            match WorthQueryAnySourceCustody::accept_correction(prepared, &request) {
                Outcome::Custody(WorthQueryAnySourceCustodyStep::Committed { .. }) => Ok(()),
                Outcome::Custody(WorthQueryAnySourceCustodyStep::Declined { .. }) => Err("custody: declined".to_owned()),
                Outcome::Custody(WorthQueryAnySourceCustodyStep::Pending { cause, .. }) => Err(format!("custody: pending {cause:?}")),
                Outcome::PublicationRefused(denial) => Err(format!("custody: refused {denial:?}")),
                Outcome::IdempotencyIntentDrift => Err("custody: intent drift".to_owned()),
                Outcome::PublicationPending => Err("custody: publication pending".to_owned()),
                Outcome::ProductUnpublished(_) => Err("custody: product unpublished".to_owned()),
            }
        })();
        reversed
    }};
}

fn values<P: worth_query_decl::facade::application_program::ApplicationProgramDefinition<model::schema::NumericSchema>>(
    runtime: &worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<model::schema::NumericSchema, P>,
) -> [u64; 3] {
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    KEYS.map(|key| {
        runtime.runtime().request(&principal, &scope).query(model::query::Read { key: key.to_owned() })
            .execute().unwrap().rows()[0].primary
    })
}

/// `SET` with the listed stages restored to the seeded 5.
fn restored(stages: &[usize]) -> [u64; 3] {
    std::array::from_fn(|n| if stages.contains(&n) { 5 } else { SET[n] })
}

/// For each stage `n` on a fresh Home: reverse `n`, then the next stage of the
/// same commit under its own key, then replay `n`'s key; each restores only its
/// Number. A fresh key on the already reversed stage `n` refuses as reversed.
macro_rules! each_stage {
    ($open:expr, $program:ty, $label:expr) => {{
        let mut refused = Vec::new();
        for n in 0..3_usize {
            let lease = checkpoint_capture::lease(256 << 20);
            let home = $open(worth_execution::ExecutionRequest::leased(&lease));
            let receipts = envelope!(home);
            let next = (n + 1) % 3;
            let steps = [(n, "undo-stage", vec![n]), (next, "undo-next", vec![n, next]), (n, "undo-stage", vec![n, next])];
            for (stage, key, expected) in steps {
                match reverse!(home, $program, key, |corrections| corrections.select(&receipts[stage], limits())) {
                    Ok(()) if values(home.runtime()) == restored(&expected) => {}
                    Ok(()) => refused.push(format!("{} stage {n}, {key} on {stage}: values {:?}", $label, values(home.runtime()))),
                    Err(cause) => { refused.push(format!("{} stage {n}, {key} on {stage}: {cause}", $label)); break; }
                }
            }
            match reverse!(home, $program, "undo-again", |corrections| corrections.select(&receipts[n], limits())) {
                Err(cause) if cause.contains("SourceAlreadyReversed") && values(home.runtime()) == restored(&[n, next]) => {}
                other => refused.push(format!("{} stage {n}, undo-again: {other:?} values {:?}", $label, values(home.runtime()))),
            }
        }
        assert!(refused.is_empty(), "#368 {}:
{}", $label, refused.join("
"));
    }};
}

/// Coordinates naming a stage the commit does not have refuse as a typed
/// member denial before any correction work, and leave every Number as set.
macro_rules! foreign_member {
    ($home:expr, $program:ty) => {{
        let real = envelope!($home)[2].correction_source().coordinates();
        let WorthQueryCorrectionSourceMember::AuthoredStage { identity, .. } = real.member() else {
            panic!("stage 2 is an authored stage member: {:?}", real.member())
        };
        let forged = WorthQueryCorrectionSourceCoordinates::from_untrusted_parts(
            real.world_owner(),
            real.world_commit(),
            real.relational().clone(),
            WorthQueryCorrectionSourceMember::AuthoredStage { ordinal: 7, identity },
        );
        let refused = reverse!($home, $program, "undo-forged", |corrections| corrections.select_coordinates(&forged, limits()));
        assert!(
            matches!(&refused, Err(cause) if cause.contains("SourceMemberUnavailable")),
            "#368 a foreign member refuses as unavailable: {refused:?}"
        );
        assert_eq!(values($home.runtime()), SET);
    }};
}

#[test]
fn each_stage_of_a_three_stage_step_corrects_on_a_memory_home() {
    each_stage!(|execution| open_memory_program_home_with(execution, STAGE_NUMBERS), model::program::Program, "memory");
}

#[test]
fn each_stage_of_a_three_stage_step_corrects_on_a_directory_home() {
    each_stage!(|execution| open_program_home_with(fixture::location(), execution, true, STAGE_NUMBERS), DirectoryProgram, "directory");
}

#[test]
fn a_memory_home_corrects_the_last_stage_from_its_coordinates() {
    let lease = checkpoint_capture::lease(256 << 20);
    let home = open_memory_program_home_with(worth_execution::ExecutionRequest::leased(&lease), STAGE_NUMBERS);
    let coordinates = envelope!(home)[2].correction_source().coordinates();
    reverse!(home, model::program::Program, "undo-last", |corrections| corrections.select_coordinates(&coordinates, limits()))
        .unwrap_or_else(|cause| panic!("#368 memory coordinates: {cause}"));
    assert_eq!(values(home.runtime()), restored(&[2]));
}

#[test]
fn a_reopened_directory_home_corrects_the_last_stage_from_its_coordinates() {
    let location = fixture::location();
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open_program_home_with(location.clone(), execution, true, STAGE_NUMBERS);
    let coordinates = envelope!(home)[2].correction_source().coordinates();
    // Graceful close after any committed edit envelope is refused today
    // (public #375), so this reopen follows an abrupt end of the first Home.
    drop(home);
    let home = open_program_home_with(location, execution, false, STAGE_NUMBERS);
    reverse!(home, DirectoryProgram, "undo-last", |corrections| corrections.select_coordinates(&coordinates, limits()))
        .unwrap_or_else(|cause| panic!("#368 directory reopen: {cause}"));
    assert_eq!(values(home.runtime()), restored(&[2]));
}

#[test]
fn a_memory_home_refuses_coordinates_naming_a_foreign_member() {
    let lease = checkpoint_capture::lease(256 << 20);
    let home = open_memory_program_home_with(worth_execution::ExecutionRequest::leased(&lease), STAGE_NUMBERS);
    foreign_member!(home, model::program::Program);
}

#[test]
fn a_directory_home_refuses_coordinates_naming_a_foreign_member() {
    let lease = checkpoint_capture::lease(256 << 20);
    let home = open_program_home_with(fixture::location(), worth_execution::ExecutionRequest::leased(&lease), true, STAGE_NUMBERS);
    foreign_member!(home, DirectoryProgram);
}

/// Reverses stages 1 and 2 of one commit, reopens, and proves both reversals
/// stay distinct: both stay active and each refuses a second reversal.
fn reopen_after_reversing_two_stages(
    location: std::path::PathBuf,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> (NumericHome, Vec<WorthQueryCorrectionSourceCoordinates>) {
    let home = open_program_home_with(location.clone(), execution, true, STAGE_NUMBERS);
    let sources: Vec<_> = envelope!(home).iter().map(|receipt| receipt.correction_source().coordinates()).collect();
    for (stage, key) in [(1, "undo-1"), (2, "undo-2")] {
        reverse!(home, DirectoryProgram, key, |corrections| corrections.select_coordinates(&sources[stage], limits()))
            .unwrap_or_else(|cause| panic!("#368 reverses stage {stage}: {cause}"));
    }
    // Graceful close after a committed edit envelope is refused (public #375).
    drop(home);
    let home = open_program_home_with(location, execution, false, STAGE_NUMBERS);
    assert_eq!(values(home.runtime()), restored(&[1, 2]), "#368 both reversals stay active");
    for (stage, key) in [(1, "again-1"), (2, "again-2")] {
        let again = reverse!(home, DirectoryProgram, key, |corrections| corrections.select_coordinates(&sources[stage], limits()));
        assert!(
            matches!(&again, Err(cause) if cause.contains("SourceAlreadyReversed")),
            "#368 stage {stage} stays reversed after reopen: {again:?}"
        );
    }
    assert_eq!(values(home.runtime()), restored(&[1, 2]));
    (home, sources)
}

#[test]
fn a_directory_home_reopens_after_reversing_two_stages_of_one_commit() {
    let lease = checkpoint_capture::lease(256 << 20);
    reopen_after_reversing_two_stages(fixture::location(), worth_execution::ExecutionRequest::leased(&lease));
}

/// Two corrections head the reopened Home, so stage 0 needs history from
/// before the reopen (public #382). That history is index-keyed and crosses
/// checkpoint-restored commits.
#[test]
fn a_reopened_directory_home_reverses_the_first_stage_after_two_later_reversals() {
    let lease = checkpoint_capture::lease(256 << 20);
    let (home, sources) =
        reopen_after_reversing_two_stages(fixture::location(), worth_execution::ExecutionRequest::leased(&lease));
    reverse!(home, DirectoryProgram, "undo-0", |corrections| corrections.select_coordinates(&sources[0], limits()))
        .unwrap_or_else(|cause| panic!("stage 0 reverses after reopen: {cause}"));
    assert_eq!(values(home.runtime()), restored(&[0, 1, 2]));
}
