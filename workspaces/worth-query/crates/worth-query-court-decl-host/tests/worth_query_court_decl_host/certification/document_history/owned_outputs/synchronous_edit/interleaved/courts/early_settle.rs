//! A producer prepared early in a positioned edit, whose input a later stage
//! then writes without reading its output, commits what the same stages
//! commit one at a time, or the edit refuses atomically. Never a third state.
use super::*;

/// One positioned stage over the cabinet world.
#[derive(Clone, Debug)]
pub(super) enum Stage {
    /// Demands `first:a` and reads its output: `first` settles before it.
    Dependent(u64),
    /// Writes `first`'s source input (model `a`) without reading `first`.
    Model(u64),
}

/// Everything a committed edit can leave behind that these stages touch.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct World {
    model: u64,
    first: Option<u64>,
    second: Option<u64>,
    dependent: Vec<u64>,
}

fn observe(host: &Host, dependent: Vec<u64>) -> World {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    World {
        model: request
            .query(query::model::Read("a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        first: (request
            .query(query::first_output::Read("first:a".into()))
            .execute())
        .ok()
        .and_then(|r| r.rows().first().map(|row| row.value)),
        second: (request
            .query(query::second_output::Read("second:a".into()))
            .execute())
        .ok()
        .and_then(|r| r.rows().first().map(|row| row.value)),
        dependent,
    }
}

/// A fresh world in which `first:a` already exists, so a dependent stage
/// alone has an output to read.
fn world() -> Host {
    let host = host(None, None);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let made = (host.request(&principal, &scope).edit_envelope())
        .stage::<ModelTree, _>(fixture::model::Intent {
            key: "a".into(),
            value: 2,
            key_derived_target: false,
        })
        .idempotency("early-settle-world".to_owned())
        .execute_performed::<Program>(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(made, Outcome::Committed(_)), "{made:?}");
    host
}

/// Runs `stages` as one positioned edit on `host`. `Ok(values)` holds each
/// dependent stage's result when it committed; `Err(())` is a refusal that
/// published nothing and left the head unchanged.
fn execute(host: &Host, key: String, stages: &[Stage]) -> Result<Vec<u64>, ()> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut edit = (request.edit_envelope().idempotency(key))
        .runtime_stages(host)
        .ordered_authored()
        .unwrap();
    let mut tokens = Vec::new();
    for stage in stages {
        match stage {
            Stage::Dependent(add) => tokens.push(
                edit.push_stage::<ProducedTree, _>(dependent::Intent { add: *add })
                    .unwrap(),
            ),
            Stage::Model(value) => {
                edit.push_stage::<ModelTree, _>(fixture::model::Intent {
                    key: "a".into(),
                    value: *value,
                    key_derived_target: false,
                })
                .unwrap();
            }
        }
    }
    let before = super::super::super::courts::native_head(host);
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
            Err(_) | Ok(Outcome::Uncommitted(_) | Outcome::DomainDenied(_))
        ),
        "{outcome:?}"
    );
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
    assert_eq!(super::super::super::courts::native_head(host), before);
    Err(())
}

/// The stages as one positioned edit: `Some(world)` when it committed,
/// `None` when it refused atomically.
pub(super) fn positioned(stages: &[Stage]) -> Option<World> {
    let host = world();
    let values = execute(&host, "early-settle".to_owned(), stages).ok()?;
    Some(observe(&host, values))
}

/// The same stages, each committed alone, in order: `None` when any of them
/// refuses, because then the whole edit must refuse too.
pub(super) fn sequential(stages: &[Stage]) -> Option<World> {
    let host = world();
    let mut values = Vec::new();
    for (ordinal, stage) in stages.iter().enumerate() {
        let key = format!("sequential:{ordinal}");
        values.extend(execute(&host, key, std::slice::from_ref(stage)).ok()?);
    }
    Some(observe(&host, values))
}

/// A positioned commit equals the sequential one; otherwise it refused.
pub(super) fn assert_sequential_or_refused(stages: &[Stage]) {
    let expected = sequential(stages);
    match positioned(stages) {
        Some(committed) => {
            eprintln!("{stages:?}: committed {committed:?}");
            assert_eq!(
                Some(committed),
                expected,
                "{stages:?}: a positioned commit differs from one commit per stage"
            );
        }
        None => eprintln!("{stages:?}: refused atomically (sequential {expected:?})"),
    }
}

/// `first` settles for the dependent stage; the next stage then writes
/// `first`'s source without reading it. The commit must carry `first` (and
/// its discovered `second`) recomputed from the final model, or refuse.
#[test]
fn a_producer_settled_early_then_fed_by_a_later_stage_commits_current_or_refuses() {
    for stages in [
        vec![Stage::Dependent(7), Stage::Model(4)],
        vec![Stage::Model(3), Stage::Dependent(7), Stage::Model(5)],
        vec![Stage::Dependent(7), Stage::Model(4), Stage::Dependent(2)],
    ] {
        assert_sequential_or_refused(&stages);
    }
}
