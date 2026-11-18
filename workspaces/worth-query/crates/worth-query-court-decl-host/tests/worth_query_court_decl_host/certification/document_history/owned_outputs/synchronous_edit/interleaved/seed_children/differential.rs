//! A positioned edit over a parent whose discovered child reads the parent's
//! input commits exactly what its stages commit one at a time.
//!
//! This is the CAD seed-edit shape: a stage that reads the parent settles the
//! parent and its discovered child; a later stage writes the seed (the input
//! both of them read) without reading either. The child must then commit from
//! the final seed.
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

/// Why a positioned edit refused atomically.
#[derive(Debug, PartialEq, Eq)]
enum Refusal {
    /// The envelope's own rule (pinned by the positioned conflicting-intent
    /// court): two authored stages that write one target, `(earlier, later)`.
    Conflict(usize, usize),
    /// The authored stage (its ordinal) that changed the input of a producer
    /// prepared earlier in the envelope; held on #229.
    Supersession(usize),
    /// A Read stage's discovery was changed by the producer (its family) that
    /// a later Write made derive the parent again. Discovery never settles
    /// without proof that its selection was replaced, so this refuses plainly.
    Rediscovered(String),
    /// Any other refusal.
    Other,
}

/// `Ok` holds each Read stage's result; `Err` is an atomic refusal.
fn execute(
    host: &Host,
    key: &str,
    stages: &[Stage],
) -> Result<Vec<u64>, (Refusal, String)> {
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
                edit.push_stage::<ApplicationNoOutputGraph, _>(frontier::Intent { frontier: *frontier, grandchild: false })
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
    Err((refusal(&outcome), format!("{outcome:?}").chars().take(1200).collect()))
}

fn refusal<E>(outcome: &Result<Outcome, E>) -> Refusal {
    use worth_query_host::facade::primary_graph::{
        WorthQueryApplicationUncommitted as Uncommitted, WorthQueryCoupledSupportHold as Hold,
        WorthQueryInvariantExecutionDenialKind as Kind2, WorthQueryOutputDemandDenialKind as Kind,
    };
    let Ok(Outcome::Uncommitted(Uncommitted::Denied(cause))) = outcome else {
        return Refusal::Other;
    };
    if let Some(Kind::ConflictingIntent { earlier, later, .. }) =
        cause.coupled_producer_denial().map(|denial| denial.kind())
    {
        return Refusal::Conflict(earlier, later);
    }
    let Some(failure) = cause.invariant_execution_failure() else {
        return Refusal::Other;
    };
    match failure.coupled_support_hold() {
        None if failure.detail().contains("discovery support") => {
            assert_eq!(failure.kind(), Kind2::CoupledReadSupportChanged, "{failure:?}");
            let family = (failure.detail().split_once("later writer producer "))
                .and_then(|(_, rest)| Some(rest.split(' ').nth(1)?.trim_end_matches(':').to_owned()))
                .unwrap_or_else(|| panic!("discovery not attributed to a producer: {failure:?}"));
            Refusal::Rediscovered(family)
        }
        Some(Hold::ProducerSupersession) => {
            assert_eq!(failure.kind(), Kind2::CoupledReadSupportChanged, "{failure:?}");
            let writer = (failure.detail().split_once("later writer authored stage "))
                .and_then(|(_, rest)| rest.split(' ').next()?.parse().ok())
                .unwrap_or_else(|| panic!("a hold not attributed to an authored stage: {failure:?}"));
            Refusal::Supersession(writer)
        }
        None => Refusal::Other,
    }
}

/// The held shape (#229), established from the stages alone: the first Write
/// that actually changes the seed after an earlier Read derived the parent.
fn superseding_stage(stages: &[Stage]) -> Option<usize> {
    let (mut model, mut derived) = (2, false);
    for (ordinal, stage) in stages.iter().enumerate() {
        match stage {
            Stage::Write(value) if derived && std::mem::replace(&mut model, *value) != *value => {
                return Some(ordinal)
            }
            Stage::Write(value) => model = *value,
            Stage::Read(_) => derived = true,
            Stage::Frontier(_) => {}
        }
    }
    None
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

fn positioned(stages: &[Stage]) -> Result<World, Refusal> {
    let host = world();
    match execute(&host, "positioned", stages) {
        Ok(results) => Ok(observe(&host, results)),
        Err((refusal, detail)) => {
            eprintln!("{stages:?} refused: {detail}");
            Err(refusal)
        }
    }
}

/// Liveness and safety (#224): a positioned edit commits the world one commit
/// per stage commits. The admitted refusals are the envelope's own (two stages
/// that write one target), and, only on their own evidence (a Read, then a
/// Write that changes the seed), the #229 hold attributed to exactly that
/// Write or the strict discovery refusal attributed to the parent's producer.
fn equals_sequential(stages: &[Stage]) -> Result<(), String> {
    let expected = sequential(stages);
    match positioned(stages) {
        Err(Refusal::Conflict(earlier, later))
            if std::mem::discriminant(&stages[earlier])
                == std::mem::discriminant(&stages[later]) =>
        {
            Ok(())
        }
        Err(Refusal::Supersession(writer)) if Some(writer) == superseding_stage(stages) => Ok(()),
        Err(Refusal::Rediscovered(family))
            if family == "first" && superseding_stage(stages).is_some() =>
        {
            Ok(())
        }
        Ok(world) if world == expected => Ok(()),
        outcome => Err(format!(
            "{stages:?}: positioned {outcome:?} differs from one commit per stage {expected:?}"
        )),
    }
}

/// Every generated edit is valid one stage at a time, so a refusal here is a
/// failure of the oracle itself, reported with the stage that refused.
fn sequential(stages: &[Stage]) -> World {
    let host = world();
    let mut results = Vec::new();
    for (ordinal, stage) in stages.iter().enumerate() {
        let key = format!("sequential:{ordinal}");
        match execute(&host, &key, std::slice::from_ref(stage)) {
            Ok(stage_results) => results.extend(stage_results),
            Err((_, refusal)) => panic!("{stages:?}: stage {ordinal} refused one commit at a time: {refusal}"),
        }
    }
    observe(&host, results)
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 32, max_shrink_iters: 64, ..ProptestConfig::default() })]
    #[test]
    fn a_seed_write_after_its_child_settled_commits_what_one_commit_per_stage_commits(
        stages in proptest::collection::vec(stage(), 2..=4)
    ) {
        if let Err(difference) = equals_sequential(&stages) {
            prop_assert!(false, "{}", difference);
        }
    }
}

/// The property is not vacuous: writing the seed, then reading the parent,
/// commits, and the child follows the final seed.
#[test]
fn a_seed_write_then_a_parent_read_commits_the_current_child() {
    let stages = [Stage::Write(3), Stage::Read(7)];
    assert_eq!(positioned(&stages).expect("this edit commits"), sequential(&stages));
}

/// The CAD seed edit itself: a source stage writes the seed and a final
/// stage demands nothing; or the parent and child settle, then a stage
/// writes the seed. It commits exactly what one commit per stage commits.
#[test]
fn a_seed_write_after_a_settled_child_never_commits_a_stale_child() {
    for stages in [
        vec![Stage::Read(7), Stage::Write(4)],
        vec![Stage::Write(3), Stage::Read(7), Stage::Write(5)],
        vec![Stage::Write(4), Stage::Frontier(5)],
        vec![Stage::Read(7), Stage::Write(4), Stage::Frontier(6)],
    ] {
        equals_sequential(&stages).unwrap();
    }
}
