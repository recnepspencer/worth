//! Application bindings are measured before edits; the model receives only identity metadata.
mod interruption;
mod kernel_observation;
use super::*;
use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandProgress as Progress,
    WorthQueryApplicationPerformedMutationOutcome as Mutation, WorthQueryApplicationRequestExt,
};
use worth_query_host::facade::primary_graph::{
    advancement_observations_on_this_thread_for_test as observations,
    place_managed_computations_on_this_thread_for_test as place,
    WorthQueryAdvancementEventKindForTest as Kind,
    WorthQueryAdvancementIdentityForTest as Identity,
    WorthQueryExecutionPlacementForTest as Placement,
};
struct Restore(Placement);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
    }
}
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Observed {
    pub work: u64,
    pub span: u64,
    pub checkpoint_work: u64,
    pub checkpoint_span: u64,
    pub committed: Vec<u64>,
    pub facts: Vec<(u64, u64)>,
    pub settled: bool,
    pub overlap: bool,
}
macro_rules! settle {
    ($d:expr,$r:expr) => {
        (0..128)
            .find_map(|_| match $d.advance(&$r).unwrap() {
                Progress::Pending | Progress::Progressed => None,
                Progress::Settled(v) => Some(v),
            })
            .expect("demand settles in declared advance count")
    };
}
pub fn run(cell: cases::Cell) -> Observed {
    let workers = NonZeroUsize::new(cell.workers).unwrap();
    let _restore = Restore(place(if cell.dispatch == cases::UNSEEDED {
        Placement::Leased(workers)
    } else {
        Placement::Certified {
            workers,
            seed: cell.dispatch,
        }
    }));
    let mut world = seeded_world::world(FAMILIES[cell.family], SEED);
    let scale = std::env::var("WORTH_REQUIRED_PRICE_SCALE")
        .map(|s| s.parse::<u64>().unwrap())
        .unwrap_or(1);
    assert!([1, 2].contains(&scale), "declared price twin");
    for part in world
        .members
        .iter_mut()
        .flat_map(|member| &mut member.partitions)
    {
        part.work *= scale;
        assert!(
            part.work < 128,
            "twins preserve the encoded amount width and owner framing"
        );
    }
    let control = Arc::new(control::Control::new(
        world.clone(),
        cell.reuse,
        cell.workers > 1,
        cell.declaration,
        cell.family == 7,
    ));
    let app = installation::install(control.clone());
    let (scope, principal) = installation::authenticate(&app);
    let request = app.request(&principal, &scope);
    let mut declaration = world.members.iter().map(|m| m.key).collect::<Vec<_>>();
    if cell.declaration == 1 {
        declaration.reverse();
    }
    let mut demands = BTreeMap::new();
    for key in declaration {
        demands.insert(
            key,
            request
                .demand(producer::Demand(key))
                .start_in_program::<application::Program, application::Root>(&app)
                .unwrap(),
        );
    }
    let mut caller = request
        .demand(producer::Demand(100))
        .start_in_program::<application::Program, application::Root>(&app)
        .unwrap();
    let mut names = BTreeMap::new();
    for key in world.members.iter().map(|m| m.key).chain([100]) {
        let rows = request.query(source::Read { key }).execute().unwrap();
        names.insert(rows.observed_sources()[0].root_entity_for_test(), key);
    }
    observations();
    for m in &world.members {
        settle!(demands.get_mut(&m.key).unwrap(), request);
    }
    settle!(caller, request);
    let declarations = control.declarations.lock().unwrap();
    for key in world.members.iter().map(|m| m.key).chain([100]) {
        let mut expected = if key == 100 {
            world
                .requested
                .iter()
                .map(|i| world.members[*i].key)
                .collect::<Vec<_>>()
        } else {
            world
                .members
                .iter()
                .find(|m| m.key == key)
                .unwrap()
                .upstream
                .iter()
                .map(|i| world.members[*i].key)
                .collect()
        };
        if cell.declaration == 1 {
            expected.reverse();
        }
        assert_eq!(
            declarations[&key], expected,
            "the handler declares the chosen dependency permutation by value"
        );
    }
    drop(declarations);
    let initial = observations();
    let mut bindings = initial
        .iter()
        .flat_map(|r| &r.applied)
        .filter(|e| e.kind == Kind::Commit)
        .filter_map(|e| match &e.identity {
            Identity::Producer {
                producer,
                query,
                parameters,
                root,
                ..
            } if names[root] != 100 => {
                Some(((producer.clone(), *query, *parameters, *root), names[root]))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    bindings.sort_by(|a, b| a.0.cmp(&b.0));
    let ranks = bindings
        .into_iter()
        .enumerate()
        .map(|(rank, (_, key))| (key, rank))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        ranks.len(),
        world.members.len(),
        "one issued binding per declared member"
    );
    let waves = model::waves(&world, &ranks);
    let order = waves
        .iter()
        .flatten()
        .map(|i| world.members[*i].key)
        .collect::<Vec<_>>();
    let failure = cell.failure(&order);
    // Only independent source roots are edited. Downstream inputs are discovered
    // through their retained consumed edges, not replaced simultaneously.
    let mut edits = world
        .members
        .iter()
        .filter(|m| m.upstream.is_empty())
        .collect::<Vec<_>>();
    if cell.declaration == 1 {
        edits.reverse();
    }
    for m in edits {
        let source = request
            .query(source::Read { key: m.key })
            .execute()
            .unwrap();
        let outcome = request
            .advanced()
            .mutate(edit::Input {
                key: m.key,
                revision: 1,
            })
            .expect_source(source.observed_sources()[0].clone())
            .idempotency(&m.key)
            .execute_performed::<application::Program, application::Root>(
                &app,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        assert!(
            matches!(outcome, Mutation::Performed(_)),
            "source edit commits"
        );
    }
    let expected = if cell.fault == 10 {
        model::Advancement::present_interruption()
    } else {
        model::advancement(&world, &waves, failure, cell.family == 7)
    };
    {
        let mut state = control.state.lock().unwrap();
        state.armed = true;
        state.failure = failure;
        state.witnesses = expected
            .witnesses
            .iter()
            .take(cell.workers)
            .copied()
            .collect();
    }
    observations();
    let cancellation = worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource::new();
    let expired_scope =
        worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope::new(
            Some(std::time::Instant::now()),
            cancellation.token(),
        );
    let expired_request = app.request(&principal, &expired_scope);
    let outcome = caller.advance(if cell.fault == 10 {
        &expired_request
    } else {
        &request
    });
    let records = observations();
    if cell.fault == 10 {
        interruption::assert_deadline(&outcome, &records);
    }
    if let Some((_, phase)) = failure {
        let Err(worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(cause))=&outcome else{panic!("declared member failure returns a typed domain refusal")};
        assert_eq!(cause.kind(),worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::ProducerDomainDenied,"declared failure keeps its domain kind");
        assert_eq!(
            cause.domain_reason(),
            Some(match phase {
                control::Phase::Prepare => "matrix prepare refusal",
                control::Phase::Compute => "matrix compute refusal",
                control::Phase::Complete => "matrix completion refusal",
            }),
            "refusal happens in the declared phase"
        );
    }
    let [record] = records.as_slice() else {
        panic!("one caller advancement: {records:?}")
    };
    assert_eq!(
        record.workers,
        if cell.fault == 10 {
            None
        } else {
            Some(cell.workers)
        },
        "the request dispatch receives the width named by its cell"
    );
    let mut committed = Vec::new();
    for pair in record.applied.chunks(2) {
        assert_eq!(pair.len(), 2, "commit/publication pair");
        assert_eq!(pair[0].kind, Kind::Commit);
        assert_eq!(pair[1].kind, Kind::Publication);
        assert_eq!(pair[0].identity, pair[1].identity);
        let Identity::Producer { root, .. } = &pair[0].identity else {
            panic!("producer binding")
        };
        committed.push(names[root]);
    }
    assert_eq!(
        committed, expected.kept,
        "canonical kept prefix, never completion order"
    );
    let settled = matches!(outcome, Ok(Progress::Settled(_)));
    assert_eq!(
        settled,
        failure.is_none() && cell.fault != 10,
        "declared failure prevents settlement: {:?}",
        outcome.as_ref().err()
    );
    let old = model::values(&world, 1);
    let new = model::values(&world, 2);
    let mut facts = Vec::new();
    for m in &world.members {
        let rows = request.query(facts::Read { key: m.key }).execute().unwrap();
        let value = rows.rows()[0].value;
        let expected_value = if committed.contains(&m.key) {
            new[&m.key]
        } else {
            old[&m.key]
        };
        assert_eq!(
            value, expected_value,
            "facts follow declared values and kept prefix"
        );
        facts.push((m.key, value));
    }
    kernel_observation::assert_kernel_observations(&control, &world, &expected, cell, failure);
    let state = control.state.lock().unwrap();
    assert_eq!(
        state.active, 0,
        "every original member compute releases its overlap witness"
    );
    let mut entered = state.entered.clone();
    let mut dispatched = expected.dispatched.clone();
    entered.sort_unstable();
    dispatched.sort_unstable();
    assert_eq!(
        entered, dispatched,
        "every dispatched member computes, including the discarded failed-wave suffix"
    );
    let overlap = !expected.witnesses.is_empty();
    // Native owners can also contain parallel work, even in a serial wave.
    if expected.span < expected.work {
        assert!(
            record.report.charged_span() < record.report.charged_work(),
            "independent members have critical-path span, never a sum of member spans"
        );
    }
    if cell.workers > 1 && overlap {
        assert!(
            state.peak >= 2,
            "leased wave has two computing members in flight"
        );
    } else {
        assert!(
            state.peak <= 1,
            "one worker has at most one computing member"
        );
    }
    assert!(
        !state.hang_guard_expired,
        "computing-member overlap hang guard expired"
    );
    Observed {
        work: record.report.charged_work(),
        span: record.report.charged_span(),
        checkpoint_work: expected.work,
        checkpoint_span: expected.span,
        committed,
        facts,
        settled,
        overlap,
    }
}
#[test]
fn required_matrix_cell() {
    let cell = std::env::var("WORTH_REQUIRED_MATRIX_CELL")
        .map(|text| cases::Cell::decode(&text))
        .unwrap_or(cases::Cell {
            family: 1,
            workers: 2,
            declaration: 0,
            dispatch: cases::UNSEEDED,
            fault: 0,
            reuse: true,
        });
    let _guard = crate::checkpoint_recovery::checkpoint_recovery_test_guard();
    let value = run(cell);
    assert!(
        value.work >= value.checkpoint_work && value.span >= value.checkpoint_span,
        "whole dispatched compute stays charged"
    );
    println!(
        "MODEL_REMAINDER:work={} span={}",
        value.work - value.checkpoint_work,
        value.span - value.checkpoint_span
    );
    println!("MATRIX_OBSERVATION:{value:?}");
}
