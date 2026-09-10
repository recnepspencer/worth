#[path = "cost/active_attempts.rs"]
mod active_attempts;
#[path = "cost/independent_writers.rs"]
mod independent_writers;
#[path = "cost/live.rs"]
mod live;
#[path = "cost/retained_partials.rs"]
mod retained_partials;
#[path = "cost/timing.rs"]
mod timing;
#[path = "cost/touched_records.rs"]
mod touched_records;

use crate::product_query_support::fork_relational_product;
use crate::query_probe::{assert_ordinary_work, read, QueryWork};
use crate::world::CourtroomWorld;

pub(super) use active_attempts::assert_active_attempt_slopes;
pub(super) use independent_writers::assert_independent_writer_slopes;
pub(super) use independent_writers::assert_sibling_writer_progress_while_combined_is_parked;
pub(super) use live::{assert_graph_work_capacity_bounds, assert_live_publication_slopes};
pub(super) use retained_partials::assert_retained_partial_slopes;
pub(super) use timing::run_scheduled_public_read_timings;
pub(super) use touched_records::assert_touched_record_slopes;

pub(super) fn assert_public_population_slopes() {
    for axis in [
        Axis::Branches,
        Axis::History,
        Axis::ComponentPins,
        Axis::Observations,
        Axis::Live,
    ] {
        let mut baseline = None;
        for size in [1, 8, 64] {
            let (work, population) = probe(axis, size);
            population.assert_exact(axis, size);
            assert_ordinary_work(work);
            assert_eq!(
                *baseline.get_or_insert(work),
                work,
                "axis={axis:?}, size={size}"
            );
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Axis {
    Branches,
    History,
    ComponentPins,
    Observations,
    Live,
}

#[derive(Debug)]
pub(super) struct Population {
    indexed_branches: usize,
    installed_history: usize,
    unique_component_pins: usize,
    component_obligations: usize,
    observations: usize,
    active_application_attempts: usize,
    active_world_publications: usize,
    active_live_consumers: usize,
}

impl Population {
    pub(super) fn capture(world: &CourtroomWorld) -> Self {
        let retention = world.application.world_retention_snapshot_for_test();
        Self {
            indexed_branches: world.application.indexed_product_branch_count_for_test(),
            installed_history: world
                .application
                .world_history_snapshot_for_test()
                .installed_commits(),
            unique_component_pins: retention.unique_pins(),
            component_obligations: retention.component_obligations(),
            observations: retention.observations(),
            active_application_attempts: world.application.active_application_attempts_for_test(),
            active_world_publications: retention.active_publication_attempts(),
            active_live_consumers: world.application.active_live_consumers_for_test(),
        }
    }

    fn assert_exact(&self, axis: Axis, requested: usize) {
        let actual = match axis {
            Axis::Branches => self.indexed_branches,
            Axis::History => self.installed_history,
            Axis::ComponentPins => self.unique_component_pins,
            Axis::Observations => self.observations,
            Axis::Live => self.active_live_consumers,
        };
        let expected = match axis {
            Axis::ComponentPins => requested + 1,
            _ => requested,
        };
        eprintln!(
            "axis={axis:?} requested={requested} actual={actual} B={} H={} U={} obligations={} O={} A_query={} A_world={} L={} covariation={}",
            self.indexed_branches,
            self.installed_history,
            self.unique_component_pins,
            self.component_obligations,
            self.observations,
            self.active_application_attempts,
            self.active_world_publications,
            self.active_live_consumers,
            axis.covariation(),
        );
        assert_eq!(actual, expected, "axis={axis:?}");
    }
}

impl Axis {
    const fn covariation(self) -> &'static str {
        match self {
            Self::Branches => "each exact-reuse branch adds one indexed head and two head obligations; retained history and unique component pins stay shared",
            Self::History => "each retained Relational successor adds one exact component pin; its undelivered performed change retains one World observation",
            Self::ComponentPins => "N product heads require N distinct Relational pins plus one shared Signal pin; B, H, and O also grow",
            Self::Observations => "observations add obligations while sharing one product occurrence and its two component pins",
            Self::Live => "each live consumer retains one Query basis lease, one provider graph-work reservation, one product observation, and its component obligations",
        }
    }
}

fn probe(axis: Axis, size: usize) -> (QueryWork, Population) {
    match axis {
        Axis::Branches => with_branches(size),
        Axis::History => with_history(size),
        Axis::ComponentPins => with_component_pins(size),
        Axis::Observations => with_observations(size),
        Axis::Live => live::read_work(size),
    }
}

fn with_branches(size: usize) -> (QueryWork, Population) {
    let world = CourtroomWorld::publish("ready");
    let root = world.application.current_world();
    let branches = (1..size)
        .map(|_| {
            world
                .application
                .branches()
                .fork(root)
                .components(|components| {
                    components
                        .reuse_exact_relational_basis()
                        .reuse_exact_signal_basis()
                })
                .create()
                .expect("the configured branch population must be admitted")
        })
        .collect::<Vec<_>>();
    let population = Population::capture(&world);
    let work = read(&world, root).work;
    for branch in branches {
        assert!(world
            .application
            .on_branch(branch)
            .close()
            .unwrap()
            .is_complete());
    }
    (work, population)
}

fn with_history(size: usize) -> (QueryWork, Population) {
    let world = CourtroomWorld::publish("ready");
    let root = world.application.current_world();
    for ordinal in 1..size {
        world
            .change_input_on_branch_with_ordinal(root, &format!("history-{ordinal}"), ordinal as u8)
            .require_committed()
            .expect("the configured history population must be admitted");
    }
    let population = Population::capture(&world);
    (read(&world, root).work, population)
}

fn with_component_pins(size: usize) -> (QueryWork, Population) {
    let world = CourtroomWorld::publish("ready");
    let root = world.application.current_world();
    let source = world.application.on_branch(root).select().unwrap();
    let branches = (1..size)
        .map(|ordinal| {
            fork_relational_product(
                &world,
                &source,
                &format!("pin-{ordinal}"),
                &format!("pin-data-{ordinal}"),
            )
        })
        .collect::<Vec<_>>();
    let observations = branches
        .iter()
        .map(|branch| world.application.on_branch(*branch).select().unwrap())
        .collect::<Vec<_>>();
    let population = Population::capture(&world);
    let work = read(&world, root).work;
    drop(observations);
    drop(source);
    for branch in branches {
        assert!(world
            .application
            .on_branch(branch)
            .close()
            .unwrap()
            .is_complete());
    }
    (work, population)
}

fn with_observations(size: usize) -> (QueryWork, Population) {
    let world = CourtroomWorld::publish("ready");
    let root = world.application.current_world();
    let observations = (0..size)
        .map(|_| world.application.on_branch(root).select().unwrap())
        .collect::<Vec<_>>();
    let population = Population::capture(&world);
    let work = read(&world, root).work;
    drop(observations);
    (work, population)
}
