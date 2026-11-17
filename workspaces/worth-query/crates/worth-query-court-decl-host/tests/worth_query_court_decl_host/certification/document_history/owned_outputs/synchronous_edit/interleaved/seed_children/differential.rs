//! A positioned edit over a parent whose discovered child reads the parent's
//! input commits what its stages commit one at a time, or refuses atomically.
//!
//! This is the CAD seed-edit shape: a stage that reads the parent settles the
//! parent and its discovered child; a later stage writes the seed (the input
//! both of them read) without reading either. The child must then commit from
//! the final seed, or the edit must refuse.
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
    /// Writes the seed (model `a`); its driving discovers `second:a`.
    Write(u64),
    /// Reads `first:a`, so the parent and its child settle before it.
    Read(u64),
    /// Demands no output and moves a record no producer reads: the CAD
    /// correction's final document stage, after its source stage.
    Frontier(u64),
}

fn stage() -> impl Strategy<Value = Stage> {
    prop_oneof![
        3 => (1..6u64).prop_map(Stage::Write),
        2 => (1..9u64).prop_map(Stage::Read),
        2 => (5..9u64).prop_map(Stage::Frontier),
    ]
}

#[derive(Debug, PartialEq, Eq)]
struct World {
    model: u64,
    parent: Option<u64>,
    child: Option<u64>,
    frontier: u64,
    results: Vec<u64>,
}

/// A world in which the parent and its discovered child already exist.
fn world() -> Host {
    let host: Host = world::install_configured(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()), 1, None,
        world::Configuration { envelope_records: true, interleaved_records: true, ..Default::default() });
    execute(&host, "seed-world", &[Stage::Write(2)]).expect("the world commits");
    assert!(observe(&host, Vec::new()).child.is_some(), "the world discovers its child");
    host
}

/// `Ok` holds each Read stage's result; `Err` is an atomic refusal.
fn execute(host: &Host, key: &str, stages: &[Stage]) -> Result<Vec<u64>, String> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut edit = (request.edit_envelope().idempotency(key.to_owned()))
        .runtime_stages(host)
        .ordered_authored()
        .unwrap();
    let mut tokens = Vec::new();
    for stage in stages {
        match stage {
            Stage::Write(value) => {
                edit.push_stage::<SeedWriteTree, _>(fixture::model::Intent {
                    key: "a".into(),
                    value: *value,
                    key_derived_target: false,
                })
                .unwrap();
            }
            Stage::Frontier(frontier) => {
                edit.push_stage::<ApplicationNoOutputGraph, _>(frontier::Intent { frontier: *frontier })
                    .unwrap();
            }
            Stage::Read(add) => tokens.push(
                edit.push_stage::<SeedReadTree, _>(dependent::Intent { add: *add }).unwrap(),
            ),
        }
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
    Err(format!("{outcome:?}").chars().take(1200).collect())
}

fn observe(host: &Host, results: Vec<u64>) -> World {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let model = |key: &str| {
        (request.query(query::model::Read(key.into())).execute())
            .ok()
            .and_then(|rows| rows.rows().first().map(|row| row.value))
    };
    World {
        model: model("a").expect("model a exists"),
        frontier: request.query(super::super::reading::Read).execute().unwrap().rows()[0].frontier,
        parent: (request.query(query::first_output::Read("first:a".into())).execute())
            .ok()
            .and_then(|rows| rows.rows().first().map(|row| row.value)),
        child: (request.query(query::second_output::Read("second:a".into())).execute())
            .ok()
            .and_then(|rows| rows.rows().first().map(|row| row.value)),
        results,
    }
}

fn positioned(stages: &[Stage]) -> Option<World> {
    let host = world();
    match execute(&host, "positioned", stages) {
        Ok(results) => Some(observe(&host, results)),
        Err(refusal) => {
            eprintln!("{stages:?} refused: {refusal}");
            None
        }
    }
}

fn sequential(stages: &[Stage]) -> Option<World> {
    let host = world();
    let mut results = Vec::new();
    for (ordinal, stage) in stages.iter().enumerate() {
        let key = format!("sequential:{ordinal}");
        results.extend(execute(&host, &key, std::slice::from_ref(stage)).ok()?);
    }
    Some(observe(&host, results))
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 32, max_shrink_iters: 64, ..ProptestConfig::default() })]
    #[test]
    fn a_seed_write_after_its_child_settled_commits_current_or_refuses(
        stages in proptest::collection::vec(stage(), 2..=4)
    ) {
        let outcome = positioned(&stages);
        eprintln!("{:?}: {:?}", stages, outcome);
        if let Some(committed) = outcome {
            prop_assert_eq!(
                Some(committed),
                sequential(&stages),
                "{:?}: a positioned commit differs from one commit per stage",
                stages
            );
        }
    }
}

/// The property is not vacuous: writing the seed, then reading the parent,
/// commits, and the child follows the final seed.
#[test]
fn a_seed_write_then_a_parent_read_commits_the_current_child() {
    let stages = [Stage::Write(3), Stage::Read(7)];
    let committed = positioned(&stages).expect("this edit commits");
    assert_eq!(Some(committed), sequential(&stages));
}

/// The CAD seed edit itself: a source stage writes the seed and a final
/// stage demands nothing; or the parent and child settle, then a stage
/// writes the seed. Committed or refused, never a stale child.
#[test]
fn a_seed_write_after_a_settled_child_never_commits_a_stale_child() {
    for stages in [
        vec![Stage::Read(7), Stage::Write(4)],
        vec![Stage::Write(3), Stage::Read(7), Stage::Write(5)],
        vec![Stage::Write(4), Stage::Frontier(5)],
        vec![Stage::Read(7), Stage::Write(4), Stage::Frontier(6)],
    ] {
        let outcome = positioned(&stages);
        eprintln!("{stages:?}: {outcome:?}");
        if let Some(committed) = outcome {
            assert_eq!(Some(committed), sequential(&stages), "{stages:?}");
        }
    }
}
