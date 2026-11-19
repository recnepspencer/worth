//! A positioned edit commits exactly when its stages commit one at a time, and
//! commits the same world: never a refusal of a valid edit, never a third world.
//!
//! The world holds the shape a CAD seed edit has: `first:a` is a parent whose
//! driving discovers the child `from-first:first:a`, and the child is derived
//! from the parent's input (model `a`). Stages may write that input without
//! reading any child, settle the parent early through a stage that needs it,
//! or read the child.
use super::*;
use crate::certification::document_history::owned_outputs::synchronous_edit::courts::native_head;
use proptest::prelude::*;
use worth_query_host::facade::{
    application_entry::WorthQueryEditEnvelopeOutcome as Outcome, runtime::ExecutionAllocationPolicy,
};
use worth_relational::facade::inspection::observe_maintenance_work;

type Host = WorthQueryProgramApplicationRuntime<Schema, Program>;

#[derive(Clone, Debug)]
enum Stage {
    /// Writes the parent's input, model `a`; reads no output.
    Model(u64),
    /// Needs `first:a`, so the parent (and the child it discovers) settle
    /// before it; writes the envelope cap.
    After(u64),
    /// Reads the discovered child's value.
    Consume,
}

fn stage() -> impl Strategy<Value = Stage> {
    prop_oneof![
        3 => (1..6u64).prop_map(Stage::Model),
        2 => (1..4u64).prop_map(Stage::After),
        2 => Just(Stage::Consume),
    ]
}

/// Everything these stages can change, and each stage's own result.
#[derive(Debug, PartialEq, Eq)]
struct World {
    model: u64,
    first: Option<u64>,
    child: Option<u64>,
    results: Vec<u64>,
}

/// A world in which the parent and its discovered child already exist.
fn world() -> Host {
    let host: Host = world::install_configured(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()), 1, None,
        world::Configuration { envelope_records: true, interleaved_records: true, ..Default::default() });
    execute(&host, "differential-world", &[(0, Stage::Model(2))]).expect("the world commits");
    host
}

/// Runs `stages` (each with its position in the generated edit) as one
/// positioned edit. `Ok` holds each stage's
/// result; `Err` is a refusal that published nothing and kept the head.
fn execute(host: &Host, key: &str, stages: &[(usize, Stage)]) -> Result<Vec<u64>, Refusal> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut edit = (request.edit_envelope().idempotency(key.to_owned()))
        .runtime_stages(host)
        .ordered_authored()
        .unwrap();
    let mut tokens = Vec::new();
    for (_, stage) in stages {
        tokens.push(
            match stage {
                Stage::Model(value) => edit.push_stage::<Tree, _>(fixture::model::Intent {
                    key: "a".into(),
                    value: *value,
                    key_derived_target: false,
                }),
                Stage::After(value) => {
                    edit.push_stage::<AfterTree, _>(fixture::after_producer::Intent {
                        key: "envelope-cap".into(),
                        value: *value,
                        key_derived_target: false,
                    })
                }
                Stage::Consume => {
                    edit.push_stage::<ApplicationNoOutputGraph, _>(value_observation::Intent {
                        model: "a".into(),
                        source: value_observation::ValueSource::Child("from-first:first:a".into()),
                    })
                }
            }
            .unwrap(),
        );
    }
    let before = native_head(host);
    let (outcome, writes) = observe_maintenance_work(32, || {
        edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
    });
    if let Ok(Outcome::Committed(receipt)) = &outcome {
        return Ok((tokens.iter())
            .map(|token| receipt.result(token).unwrap().value)
            .collect());
    }
    assert!(
        matches!(
            outcome,
            Err(_)
                | Ok(Outcome::Uncommitted(_)
                    | Outcome::DomainDenied(_)
                    | Outcome::StageExecutionDenied(_))
        ),
        "{outcome:?}"
    );
    assert_eq!(
        writes.published_rows.len() + writes.omitted_publications,
        0,
        "a refusal published"
    );
    assert_eq!(native_head(host), before, "a refusal moved the head");
    Err(refusal(&outcome))
}

/// Why a positioned edit refused atomically.
#[derive(Debug, PartialEq, Eq)]
enum Refusal {
    /// The envelope's own rule (pinned by the positioned conflicting-intent
    /// court): two authored stages that write one target, `(earlier, later)`.
    Conflict(usize, usize),
    /// The authored stage (its ordinal) that changed the input of a producer
    /// prepared earlier in the envelope; held until in-envelope producer
    /// supersession (#229).
    Supersession(usize),
    /// Any other refusal, as it was reported.
    Other(String),
}

fn refusal<E: std::fmt::Debug>(outcome: &Result<Outcome, E>) -> Refusal {
    use worth_query_host::facade::primary_graph::{
        WorthQueryApplicationUncommitted as Uncommitted, WorthQueryCoupledSupportHold as Hold,
        WorthQueryInvariantExecutionDenialKind as Kind2, WorthQueryOutputDemandDenialKind as Kind,
    };
    let Ok(Outcome::Uncommitted(Uncommitted::Denied(cause))) = outcome else {
        return Refusal::Other(format!("{outcome:?}"));
    };
    if let Some(Kind::ConflictingIntent { earlier, later, .. }) =
        cause.coupled_producer_denial().map(|denial| denial.kind())
    {
        return Refusal::Conflict(earlier, later);
    }
    let Some(failure) = cause.invariant_execution_failure() else {
        return Refusal::Other(format!("{cause:?}"));
    };
    match failure.coupled_support_hold() {
        Some(Hold::ProducerSupersession) => {
            assert_eq!(
                failure.kind(),
                Kind2::CoupledReadSupportChanged,
                "{failure:?}"
            );
            let writer = (failure.detail().split_once("later writer authored stage "))
                .and_then(|(_, rest)| rest.split(' ').next()?.parse().ok())
                .unwrap_or_else(|| {
                    panic!("a hold not attributed to an authored stage: {failure:?}")
                });
            Refusal::Supersession(writer)
        }
        // Holds for other open issues (#238, #239) are not this property's.
        Some(
            Hold::DiscoverySupportChanged
            | Hold::ReaderProducerSupersession
            | Hold::ConsumerSupersession,
        )
        | None => Refusal::Other(format!("{cause:?}")),
    }
}

/// The held shape (#229), established from the stages alone. A root is only
/// derived inside the envelope at a pre-write barrier: a later stage that
/// reads an output (`After` or `Consume`) while a demand is already pending
/// from an earlier stage. `After` queues its own root only after its stage
/// and a model change makes `first:a` pending; neither derives anything by
/// itself. The held stage is the first model write that changes the model
/// after such a barrier. `None` when no stage supersedes an in-envelope
/// derivation, so the edit must commit what one commit per stage commits.
fn superseding_stage(stages: &[(usize, Stage)]) -> Option<usize> {
    let (mut model, mut pending, mut derived) = (2, false, false);
    for (ordinal, (_, stage)) in stages.iter().enumerate() {
        match stage {
            Stage::Model(value) => {
                if std::mem::replace(&mut model, *value) != *value {
                    if derived {
                        return Some(ordinal);
                    }
                    pending = true;
                }
            }
            Stage::After(_) => {
                derived |= pending;
                pending = true;
            }
            Stage::Consume => derived |= pending,
        }
    }
    None
}

fn observe(host: &Host, results: Vec<u64>) -> World {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let first = |key: String| {
        (request.query(query::first_output::Read(key)).execute())
            .ok()
            .and_then(|rows| rows.rows().first().map(|row| row.value))
    };
    World {
        model: request
            .query(query::model::Read("a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        first: first("first:a".into()),
        child: (request
            .query(query::child_output::Read("from-first:first:a".into()))
            .execute())
        .ok()
        .and_then(|rows| rows.rows().first().map(|row| row.value)),
        results,
    }
}

/// The stages as one positioned edit: `Err` when it refused atomically.
fn positioned(stages: &[(usize, Stage)]) -> Result<World, Refusal> {
    let host = world();
    let results = execute(&host, "positioned", stages)?;
    Ok(observe(&host, results))
}

/// The stages one commit each, in order. Every generated edit is valid one
/// stage at a time, so a refusal here is a failure of the oracle itself,
/// reported with the stage that refused.
fn sequential(stages: &[(usize, Stage)]) -> World {
    let host = world();
    let mut results = Vec::new();
    for (ordinal, stage) in stages.iter().enumerate() {
        let key = format!("sequential:{ordinal}");
        match execute(&host, &key, std::slice::from_ref(stage)) {
            Ok(stage_results) => results.extend(stage_results),
            Err(refusal) => {
                panic!("{stages:?}: stage {ordinal} refused one commit at a time: {refusal:?}")
            }
        }
    }
    observe(&host, results)
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 32, max_shrink_iters: 64, ..ProptestConfig::default() })]
    #[test]
    fn a_positioned_edit_commits_exactly_what_one_commit_per_stage_commits(
        stages in proptest::collection::vec(stage(), 2..=4)
    ) {
        let stages: Vec<_> = stages.into_iter().enumerate().collect();
        let expected = sequential(&stages);
        let outcome = positioned(&stages);
        eprintln!("{:?}: {:?}", stages, outcome.as_ref().map(|_| "committed"));
        if let Err(Refusal::Conflict(earlier, later)) = outcome {
            // The envelope refuses two stages that write one target.
            prop_assert_eq!(
                std::mem::discriminant(&stages[earlier].1),
                std::mem::discriminant(&stages[later].1),
                "{:?}: a conflict between stages that write different targets",
                stages
            );
        } else if let Err(Refusal::Supersession(writer)) = outcome {
            // Held on #229, only for its shape, attributed to its exact stage.
            prop_assert_eq!(
                Some(writer),
                superseding_stage(&stages),
                "{:?}: a supersession hold without its evidence",
                stages
            );
        } else {
            // Liveness and safety (#224): otherwise the positioned edit commits
            // the world one commit per stage commits.
            prop_assert_eq!(outcome, Ok(expected), "{:?}: a positioned edit differs from one commit per stage", stages);
        }
    }
}

/// The property is not vacuous: settling the parent, then reading its
/// discovered child, then changing nothing it read, commits.
#[test]
fn the_differential_world_commits_a_settled_child_read() {
    let stages = [
        (0, Stage::Model(3)),
        (1, Stage::After(1)),
        (2, Stage::Consume),
    ];
    assert_eq!(positioned(&stages), Ok(sequential(&stages)));
}

/// A leading child read demands no producer: it reads the child committed
/// under model 2, so the later model write supersedes nothing and the edit
/// must commit what one commit per stage commits.
#[test]
fn a_committed_child_read_before_a_model_write_commits() {
    let stages = [(0, Stage::Consume), (1, Stage::Model(4))];
    assert_eq!(superseding_stage(&stages), None);
    assert_eq!(positioned(&stages), Ok(sequential(&stages)));
}

/// A leading `After` reads the committed `first:a` and queues its root only
/// after its stage; the model write reads no output, so nothing derives before
/// it and the edit must commit what one commit per stage commits.
#[test]
fn a_leading_after_read_before_a_model_write_commits() {
    let stages = [(0, Stage::After(3)), (1, Stage::Model(4))];
    assert_eq!(superseding_stage(&stages), None);
    assert_eq!(positioned(&stages), Ok(sequential(&stages)));
}

/// The held shape (#229): `first:a` is prepared for the consumer, then a later
/// stage changes its model. One commit per stage commits first 30; the
/// positioned edit refuses atomically with the supersession hold and nothing
/// else, attributed to the later authored stage.
#[test]
fn a_later_stage_changing_a_prepared_producers_input_is_held_on_supersession() {
    let stages = [
        (0, Stage::After(1)),
        (1, Stage::Consume),
        (2, Stage::Model(3)),
    ];
    assert_eq!(sequential(&stages).first, Some(30));
    assert_eq!(superseding_stage(&stages), Some(2));
    assert_eq!(positioned(&stages), Err(Refusal::Supersession(2)));
}
