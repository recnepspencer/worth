//! Public #382: a reopened Directory Home corrects a source that later commits
//! follow, from before or after the reopen. Reversing the head skips history;
//! an older source needs the authored-write history that a reopen drops. A
//! source from before the reopen also needs index-keyed history across
//! checkpoint-restored commits, held on public #384.
use super::numeric_home::*;
use super::*;
use model::{
    authentication::{authenticate_operator, request_scope},
    correction::RecordedEdit,
    edit::{Edit, Edited, Value},
};
use worth_query_host::facade::{
    application_entry::WorthQueryCorrectionHistoryLimits as Limits,
    WorthQueryAnyCorrectionCustodyOutcome as Outcome, WorthQueryAnySourceCustody,
    WorthQueryAnySourceCustodyStep as Step, WorthQuerySourceCustody,
};

type OtherAction = worth_query_decl::facade::application_program::ApplicationActionReference<
    model::schema::NumericSchema,
    worth_query_decl::facade::application_program::ApplicationRootComposition,
    model::program::NumericFeature,
    model::other_edit::OtherBinding,
    model::program::OtherCompletion,
>;

const KEYS: [&str; 2] = ["numeric", "numeric-b"];

/// Commits a plain edit of `key` and returns its retained receipt.
macro_rules! edit {
    ($home:expr, $program:ty, $key:expr, $value:expr, $id:expr) => {{
        let scope = request_scope();
        let principal = authenticate_operator($home.primary_graph().installed_schema(), &scope);
        let request = $home.runtime().request(&principal, &scope);
        let described = request
            .mutate(Edit { key: $key.to_owned(), value: Value::Primary($value) })
            .without_source()
            .idempotency_owned($id.to_owned())
            .unwrap_or_else(|(_, _, denial)| panic!("edit: {denial:?}"));
        match WorthQuerySourceCustody::<_, _, $program, model::program::Action>::Described(described)
            .erase()
            .resume($home.runtime(), &request)
        {
            Step::Committed { evidence, .. } => evidence
                .into_evidence::<Edited>()
                .unwrap_or_else(|_| panic!("Edited"))
                .live_receipt()
                .cloned()
                .expect("live receipt"),
            _ => panic!("edit must commit"),
        }
    }};
}

/// Reverses a source under `key` with `limits`, selected from its coordinates
/// (a Directory Home) or its retained receipt (a Memory Home, whose history
/// lives only while held); the refusal is rendered for comparison.
macro_rules! undo {
    ($home:expr, $program:ty, $key:expr, $select:ident($source:expr), $limits:expr) => {{
        let scope = request_scope();
        let principal = authenticate_operator($home.primary_graph().installed_schema(), &scope);
        let request = $home.runtime().request(&principal, &scope);
        let undone: Result<(), String> = (|| {
            let selected = request
                .corrections()
                .$select($source, $limits)
                .map_err(|denial| format!("select: {denial:?}"))?;
            let prepared = request
                .corrections()
                .reverse(&selected)
                .idempotency($key)
                .limits($limits)
                .prepare_operation::<RecordedEdit, $program, OtherAction>($home.runtime())
                .map_err(|denial| format!("prepare: {denial:?}"))?;
            match WorthQueryAnySourceCustody::accept_correction(prepared, &request) {
                Outcome::Custody(Step::Committed { .. }) => Ok(()),
                _ => Err("custody: not committed".to_owned()),
            }
        })();
        undone
    }};
}

fn values<P: worth_query_decl::facade::application_program::ApplicationProgramDefinition<model::schema::NumericSchema>>(
    runtime: &worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<model::schema::NumericSchema, P>,
) -> [u64; 2] {
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    KEYS.map(|key| {
        runtime.runtime().request(&principal, &scope).query(model::query::Read { key: key.to_owned() })
            .execute().unwrap().rows()[0].primary
    })
}

fn refused_by_history(outcome: &Result<(), String>) -> bool {
    matches!(outcome, Err(cause) if cause.contains("GenerationUnavailable"))
}

#[test]
#[ignore = "held on public #384: prepare: Derivation(RestoredSupport(Native(IncompleteIndexHistory)))"]
fn a_reopened_directory_home_reverses_a_source_from_before_the_reopen() {
    let location = fixture::location();
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open_program_home_with(location.clone(), execution, true, STAGE_NUMBERS);
    let a = edit!(home, DirectoryProgram, KEYS[0], 10, "edit-a").correction_source().coordinates();
    let b = edit!(home, DirectoryProgram, KEYS[1], 20, "edit-b").correction_source().coordinates();
    undo!(home, DirectoryProgram, "undo-b", select_coordinates(&b), limits()).expect("the head reverses");
    drop(home);
    let home = open_program_home_with(location, execution, false, STAGE_NUMBERS);
    undo!(home, DirectoryProgram, "undo-a", select_coordinates(&a), limits())
        .unwrap_or_else(|cause| panic!("#382 the older source reverses after reopen: {cause}"));
    assert_eq!(values(home.runtime()), [5, 5]);
    let again = undo!(home, DirectoryProgram, "again-a", select_coordinates(&a), limits());
    assert!(matches!(&again, Err(cause) if cause.contains("SourceAlreadyReversed")), "{again:?}");
}

#[test]
fn a_reopened_directory_home_reverses_an_older_source_from_after_the_reopen() {
    let location = fixture::location();
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    drop(open_program_home_with(location.clone(), execution, true, STAGE_NUMBERS));
    let home = open_program_home_with(location, execution, false, STAGE_NUMBERS);
    let c = edit!(home, DirectoryProgram, KEYS[0], 10, "edit-c").correction_source().coordinates();
    edit!(home, DirectoryProgram, KEYS[1], 20, "edit-d");
    undo!(home, DirectoryProgram, "undo-c", select_coordinates(&c), limits())
        .unwrap_or_else(|cause| panic!("#382 a post-reopen commit keeps its history: {cause}"));
    assert_eq!(values(home.runtime()), [5, 20]);
}

#[test]
fn a_memory_home_reverses_an_older_source() {
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open_memory_program_home_with(execution, STAGE_NUMBERS);
    let a = edit!(home, model::program::Program, KEYS[0], 10, "edit-a");
    let b = edit!(home, model::program::Program, KEYS[1], 20, "edit-b");
    undo!(home, model::program::Program, "undo-b", select(&b), limits()).expect("the head reverses");
    undo!(home, model::program::Program, "undo-a", select(&a), limits()).expect("the older source reverses");
    assert_eq!(values(home.runtime()), [5, 5]);
}

/// A later write to the reversed key lies in a checkpointed commit. The seed
/// must see it: the reopened Home refuses exactly as a Memory Home does.
#[test]
#[ignore = "held on public #384: prepare: Derivation(RestoredSupport(Native(IncompleteIndexHistory)))"]
fn a_reopened_directory_home_sees_a_checkpointed_write_to_the_reversed_key() {
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let memory = open_memory_program_home_with(execution, STAGE_NUMBERS);
    let a = edit!(memory, model::program::Program, KEYS[0], 10, "edit-a");
    edit!(memory, model::program::Program, KEYS[0], 20, "edit-b");
    let expected = undo!(memory, model::program::Program, "undo-a", select(&a), limits());
    assert!(expected.is_err() && !refused_by_history(&expected), "{expected:?}");
    let location = fixture::location();
    let home = open_program_home_with(location.clone(), execution, true, STAGE_NUMBERS);
    let a = edit!(home, DirectoryProgram, KEYS[0], 10, "edit-a").correction_source().coordinates();
    edit!(home, DirectoryProgram, KEYS[0], 20, "edit-b");
    drop(home);
    let home = open_program_home_with(location, execution, false, STAGE_NUMBERS);
    let reopened = undo!(home, DirectoryProgram, "undo-a", select_coordinates(&a), limits());
    assert_eq!(reopened, expected, "#382 the reopened Home refuses as the Memory Home does");
    assert_eq!(values(home.runtime()), [20, 5]);
}

/// The ancestor limit still bounds a reopened Home's correction.
#[test]
fn a_reopened_directory_home_refuses_a_source_past_its_ancestor_limit() {
    let location = fixture::location();
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open_program_home_with(location.clone(), execution, true, STAGE_NUMBERS);
    let a = edit!(home, DirectoryProgram, KEYS[0], 10, "edit-a").correction_source().coordinates();
    edit!(home, DirectoryProgram, KEYS[1], 20, "edit-b");
    edit!(home, DirectoryProgram, KEYS[1], 30, "edit-c");
    drop(home);
    let home = open_program_home_with(location, execution, false, STAGE_NUMBERS);
    let one = Limits::new(std::num::NonZeroUsize::new(1), std::num::NonZeroUsize::new(8).unwrap());
    let refused = undo!(home, DirectoryProgram, "undo-a", select_coordinates(&a), one);
    assert!(refused.is_err() && !refused_by_history(&refused), "a typed limit refusal: {refused:?}");
    assert_eq!(values(home.runtime()), [10, 30]);
}
