//! Application bindings are measured before edits; the model receives only identity metadata.
mod atomic_publication;
mod interruption;
mod kernel_observation;
mod member_observation;
#[cfg(test)]
mod source_affinity;
use super::*;
use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandProgress as Progress, WorthQueryApplicationRequestExt,
    WorthQueryEditEnvelopeOutcome as Envelope,
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
    let issued = edits
        .iter()
        .map(|m| {
            let rows = request
                .query(source::Read { key: m.key })
                .execute()
                .unwrap();
            (m.key, rows.observed_sources()[0].clone())
        })
        .collect::<Vec<_>>();
    let expected = if cell.fault == 10 {
        model::Advancement::present_interruption()
    } else {
        model::advancement(&world, &waves, failure, cell.family == 7)
    };
    {
        let mut state = control.state.lock().unwrap();
        state.armed = true;
        state.failure = failure;
        state.member_waves = expected.dispatch_waves.clone();
        state.wave_active = vec![0; state.member_waves.len()];
        // Native run_order claims its scheduling-order buffer by atomic index
        // (worth-execution/src/backend/native.rs). Thus the first ranked
        // computing members can fill these slots without waiting on queued work.
        state.member_rendezvous = expected
            .dispatch_waves
            .iter()
            .map(|wave| wave.iter().take(cell.workers).copied().collect())
            .collect();
        state.wave_peaks = vec![0; state.member_rendezvous.len()];
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
            Some(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system().now()),
            cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let expired_request = app.request(&principal, &expired_scope);
    let active_request = if cell.fault == 10 {
        &expired_request
    } else {
        &request
    };
    let mut envelope = active_request
        .edit_envelope()
        .idempotency(0x6a00_u64)
        .runtime_stages(&app)
        .ordered_authored()
        .unwrap();
    let tokens = issued
        .into_iter()
        .map(|(key, source)| {
            envelope
                .push_stage_with_source::<application::Root, _, _>(
                    edit::Input { key, revision: 1 },
                    source,
                )
                .unwrap()
        })
        .collect::<Vec<_>>();
    let (outcome, writes) =
        worth_relational::facade::inspection::observe_maintenance_work(1, || {
            envelope.execute_performed(
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
        });
    let records = observations();
    if cell.fault == 10 {
        interruption::assert_deadline(&outcome, &records);
    }
    let settled = matches!(&outcome, Ok(Envelope::Committed(_)));
    atomic_publication::assert_outcome(
        &outcome,
        &tokens,
        failure,
        cell.fault == 10,
        writes.published_rows.len() + writes.omitted_publications,
    );
    let [record] = records.as_slice() else {
        panic!("one envelope advancement: {records:?}")
    };
    assert_eq!(
        record.workers,
        if cell.fault == 10 {
            None
        } else {
            Some(cell.workers)
        },
        "the envelope receives the width named by its cell"
    );
    let old = model::values(&world, 1);
    let new = model::values(&world, 2);
    let mut committed = Vec::new();
    let mut facts = Vec::new();
    for m in &world.members {
        let rows = request.query(facts::Read { key: m.key }).execute().unwrap();
        let value = rows.rows()[0].value;
        let expected_value = if settled && expected.kept.contains(&m.key) {
            new[&m.key]
        } else {
            old[&m.key]
        };
        assert_eq!(
            value, expected_value,
            "member {} facts follow the independent values and atomic publication",
            m.key
        );
        if value != old[&m.key] {
            committed.push(m.key);
        }
        facts.push((m.key, value));
    }
    for m in &world.members {
        let source = request
            .query(source::Read { key: m.key })
            .execute()
            .unwrap();
        assert_eq!(
            source.rows()[0].revision,
            u64::from(settled && m.upstream.is_empty()),
            "only authored independent sources change, and only at final publication"
        );
    }
    let aggregate = request.query(facts::Read { key: 100 }).execute().unwrap();
    let expected_values = if settled { &new } else { &old };
    if aggregate.rows()[0].value
        != world
            .requested
            .iter()
            .map(|i| old[&world.members[*i].key])
            .sum::<u64>()
    {
        committed.push(100);
    }
    assert_eq!(
        aggregate.rows()[0].value,
        world
            .requested
            .iter()
            .map(|i| expected_values[&world.members[*i].key])
            .sum::<u64>(),
        "the aggregate consumes the independently computed required outputs"
    );
    // Atomic publication has no independently committed producer prefix. Report
    // the actual changed output set, not the model's predicted commit order.
    committed.sort_unstable();
    member_observation::assert_member_widths(&control, &expected.dispatch_waves, cell.workers);
    kernel_observation::assert_kernel_observations(&control, &world, &expected, cell, failure);
    member_observation::assert_completion_order(
        &control,
        &world,
        &waves,
        failure,
        cell.fault == 10,
    );
    atomic_publication::assert_current_outputs(&app, &world, &names, expected_values);
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

#[cfg(test)]
#[path = "runtime/serial.rs"]
mod serial;
