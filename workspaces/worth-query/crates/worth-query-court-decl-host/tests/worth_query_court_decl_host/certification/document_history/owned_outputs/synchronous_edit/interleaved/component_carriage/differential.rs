//! A positioned edit commits exactly when its stages commit one at a time, and
//! commits the same world: never a refusal of a valid edit, never a third world.
//!
//! The world holds the shape a CAD seed edit has: `first:a` is a parent whose
//! driving discovers the child `from-first:first:a`, and the child is derived
//! from the parent's input (model `a`). Stages may write that input without
//! reading any child, settle the parent early through a stage that needs it,
//! read the child, or link a new input member to the parent.
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
    /// Reads `first:a` and links a new member to model `a`: a new parent input.
    Attach { pending: bool },
}

fn stage() -> impl Strategy<Value = Stage> {
    prop_oneof![
        3 => (1..6u64).prop_map(Stage::Model),
        2 => (1..4u64).prop_map(Stage::After),
        2 => Just(Stage::Consume),
        1 => any::<bool>().prop_map(|pending| Stage::Attach { pending }),
    ]
}

/// Everything these stages can change, and each stage's own result.
#[derive(Debug, PartialEq, Eq)]
struct World {
    model: u64,
    first: Option<u64>,
    child: Option<u64>,
    attached: Vec<Option<u64>>,
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

/// Runs `stages` (each with its position in the generated edit, which names
/// its attached member) as one positioned edit. `Ok` holds each stage's
/// result; `Err` is a refusal that published nothing and kept the head,
/// holding the conflicting stages when the envelope refused a same-target pair.
fn execute(host: &Host, key: &str, stages: &[(usize, Stage)]) -> Result<Vec<u64>, Option<(usize, usize)>> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut edit = (request.edit_envelope().idempotency(key.to_owned()))
        .runtime_stages(host)
        .ordered_authored()
        .unwrap();
    let mut tokens = Vec::new();
    for (position, stage) in stages {
        tokens.push(match stage {
            Stage::Model(value) => edit.push_stage::<Tree, _>(fixture::model::Intent {
                key: "a".into(),
                value: *value,
                key_derived_target: false,
            }),
            Stage::After(value) => edit.push_stage::<AfterTree, _>(fixture::after_producer::Intent {
                key: "envelope-cap".into(),
                value: *value,
                key_derived_target: false,
            }),
            Stage::Consume => edit.push_stage::<ApplicationNoOutputGraph, _>(pending_consumer::Intent {
                model: "a".into(),
                key: "from-first:first:a".into(),
            }),
            Stage::Attach { pending } => edit.push_stage::<ApplicationNoOutputGraph, _>(attach::Intent {
                parent: "a".into(),
                child: format!("carriage:p{position}"),
                pending_child: *pending,
            }),
        }
        .unwrap());
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
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0, "a refusal published");
    assert_eq!(native_head(host), before, "a refusal moved the head");
    Err(conflict(&outcome))
}

/// The envelope's own rule (pinned by the positioned conflicting-intent
/// court): two authored stages that write one target refuse as
/// `ConflictingIntent`. `None` for any other refusal.
fn conflict<E>(
    outcome: &Result<Outcome, E>,
) -> Option<(usize, usize)> {
    use worth_query_host::facade::primary_graph::{
        WorthQueryApplicationUncommitted as Uncommitted, WorthQueryOutputDemandDenialKind as Kind,
    };
    let Ok(Outcome::Uncommitted(Uncommitted::Denied(cause))) = outcome else {
        return None;
    };
    match cause.coupled_producer_denial()?.kind() {
        Kind::ConflictingIntent { earlier, later, .. } => Some((earlier, later)),
        _ => None,
    }
}

fn observe(host: &Host, stages: &[(usize, Stage)], results: Vec<u64>) -> World {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let first = |key: String| {
        (request.query(query::first_output::Read(key)).execute())
            .ok()
            .and_then(|rows| rows.rows().first().map(|row| row.value))
    };
    World {
        model: request.query(query::model::Read("a".into())).execute().unwrap().rows()[0].value,
        first: first("first:a".into()),
        child: (request.query(query::child_output::Read("from-first:first:a".into())).execute())
            .ok()
            .and_then(|rows| rows.rows().first().map(|row| row.value)),
        attached: (stages.iter())
            .filter(|(_, stage)| matches!(stage, Stage::Attach { .. }))
            .map(|(position, _)| first(format!("first:carriage:p{position}")))
            .collect(),
        results,
    }
}

/// The stages as one positioned edit: `Err` when it refused atomically.
fn positioned(stages: &[(usize, Stage)]) -> Result<World, Option<(usize, usize)>> {
    let host = world();
    let results = execute(&host, "positioned", stages)?;
    Ok(observe(&host, stages, results))
}

/// The stages one commit each, in order: `None` when any one refuses.
fn sequential(stages: &[(usize, Stage)]) -> Option<World> {
    let host = world();
    let mut results = Vec::new();
    for (ordinal, stage) in stages.iter().enumerate() {
        let key = format!("sequential:{ordinal}");
        results.extend(execute(&host, &key, std::slice::from_ref(stage)).ok()?);
    }
    Some(observe(&host, stages, results))
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 32, max_shrink_iters: 64, ..ProptestConfig::default() })]
    #[test]
    fn a_positioned_edit_commits_exactly_what_one_commit_per_stage_commits(
        stages in proptest::collection::vec(stage(), 2..=4)
    ) {
        let stages: Vec<_> = stages.into_iter().enumerate().collect();
        let outcome = positioned(&stages);
        eprintln!("{:?}: {}", stages, if outcome.is_ok() { "committed" } else { "refused" });
        if let Err(Some((earlier, later))) = outcome {
            // The envelope refuses two stages that write one target.
            prop_assert_eq!(
                std::mem::discriminant(&stages[earlier].1),
                std::mem::discriminant(&stages[later].1),
                "{:?}: a conflict between stages that write different targets",
                stages
            );
        } else {
            // Liveness and safety (#224): otherwise the positioned edit commits
            // exactly when one commit per stage commits, the same world.
            prop_assert_eq!(
                outcome.ok(),
                sequential(&stages),
                "{:?}: a positioned edit differs from one commit per stage",
                stages
            );
        }
    }
}

/// The property is not vacuous: settling the parent, then reading its
/// discovered child, then changing nothing it read, commits.
#[test]
fn the_differential_world_commits_a_settled_child_read() {
    let stages = [(0, Stage::Model(3)), (1, Stage::After(1)), (2, Stage::Consume)];
    let committed = positioned(&stages).expect("this edit commits");
    assert_eq!(Some(committed), sequential(&stages));
}
