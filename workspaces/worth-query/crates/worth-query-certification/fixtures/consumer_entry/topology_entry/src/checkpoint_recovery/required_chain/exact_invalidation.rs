//! The exact-invalidation courtroom. A plain model of independent rings says
//! what every settled output must be, while production marking runs beside
//! full verification and fails on any difference.

use super::*;
use std::cell::Cell;
use worth_query_consumer_values::{
    PlanarDerivedOutput, PlanarOperation, PlanarVertex, PlanarVertexReplacement, PositiveLength,
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationOutputDemandDenial,
};
use worth_query_host::facade::primary_graph::inexact_native_deliveries_on_this_thread_for_test as inexact_deliveries;

type Runtime = application_installation::WorthQueryProgramApplicationRuntime<
    CheckpointSchema,
    program::ChainProgram,
>;
type Request<'application, 'principal, 'scope> =
    worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        'application,
        'principal,
        'scope,
        CheckpointSchema,
    >;
type Seed = fn(
    &mut worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphBootstrap<CheckpointSchema>,
);

/// What the invalidation owner may retain of delivered marks.
#[derive(Clone, Copy)]
struct Retained {
    bytes: u64,
    commit_positions: usize,
}

impl Retained {
    /// A window no courtroom history outgrows, and room for every row its
    /// retained versions hold.
    const AMPLE: Self = Self {
        bytes: 1 << 40,
        commit_positions: 512,
    };
}

/// Product observations enough for every output a courtroom history leaves
/// cached and every source its performed writes keep.
const AMPLE_OBSERVATIONS: u64 = 64;

/// A courtroom world over `seed`, with room for a hundred rings to keep their
/// three demands open and for the rows a random history leaves behind.
fn install(
    checkpoint: Option<application_installation::WorthQueryApplicationCheckpoint>,
    seed: Seed,
    retained: Retained,
) -> Runtime {
    install_observing(checkpoint, seed, retained, AMPLE_OBSERVATIONS)
}

/// A courtroom world whose branch admits `observations` product observations
/// at once.
fn install_observing(
    checkpoint: Option<application_installation::WorthQueryApplicationCheckpoint>,
    seed: Seed,
    retained: Retained,
    observations: u64,
) -> Runtime {
    let profile =
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard();
    let work = u64::try_from(profile.limits().source_currentness_work()).unwrap();
    support::install_program_with_limits::<program::ChainProgram>(
        checkpoint,
        profile,
        support::limits_with_room(
            4_096,
            observations,
            512,
            support::invalidation(retained.bytes, work, retained.commit_positions),
        ),
        seed,
    )
}

/// The plain model of one ring: only what the test itself wrote.
#[derive(Clone, Debug)]
struct Ring {
    index: usize,
    a_y: u64,
    /// The body `a` precedes: the seeded `source-b`, or its replacement.
    successor: String,
    successor_y: u64,
    far_y: u64,
    b_length: u64,
    c_length: u64,
    /// The root output the latest decision of `b` read, and the `b` output
    /// the latest decision of `c` read.
    b_read: Option<u64>,
    c_read: Option<u64>,
}

impl Ring {
    /// A ring the bootstrap seeded: every Length is its body's Y plus one.
    fn seeded(index: usize) -> Self {
        Self {
            index,
            a_y: 1,
            successor: ring_world::key(index, "source-b"),
            successor_y: 1,
            far_y: 10,
            b_length: 16,
            c_length: 11,
            b_read: None,
            c_read: None,
        }
    }

    /// A ring a commit created: every Length starts at one.
    fn created(index: usize) -> Self {
        Self {
            b_length: 1,
            c_length: 1,
            ..Self::seeded(index)
        }
    }

    fn key(&self, role: &str) -> String {
        ring_world::key(self.index, role)
    }

    fn root_output(&self) -> u64 {
        self.a_y + 1
    }
}

/// Every decision since the last judgment read exactly what the model says
/// its upstream publishes. Returns how many decisions ran.
fn judge_decisions(rings: &mut [Ring], at: &str) -> usize {
    let decisions = take_all_decisions();
    for (scope, read) in &decisions {
        let (index, role) = ring_world::ring_and_role(scope)
            .unwrap_or_else(|| panic!("{at}: {scope} is no ring body"));
        let ring = &mut rings[index];
        let (upstream, retained) = match role {
            "b" => (ring.root_output(), &mut ring.b_read),
            "c" => (ring.b_length, &mut ring.c_read),
            _ => panic!("{at}: {scope} is no chain node"),
        };
        assert_eq!(
            read,
            &[upstream],
            "{at}: {scope} decides over its upstream's current output"
        );
        *retained = Some(upstream);
    }
    decisions.len()
}

/// What settling one demand cost, from the existing observers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Cost {
    producer_contacts: usize,
    source_queries: u64,
}

impl Cost {
    /// A clean output: no producer contact, and no source query ran again.
    fn is_free(&self) -> bool {
        self.producer_contacts == 0 && self.source_queries == 0
    }
}

/// Whether a stopped demand may be asked again: its upstream is still
/// settling.
fn offers_retry(stop: &WorthQueryApplicationOutputDemandDenial) -> bool {
    matches!(
        stop,
        WorthQueryApplicationOutputDemandDenial::Demand(denial)
            if denial.recovery_posture()
                == primary_graph::WorthQueryOutputDemandRecoveryPosture::Retryable
    )
}

/// An open demand of the output of `body`.
struct Open<Demand> {
    body: String,
    demand: Demand,
}

/// One advance settles `$demand`, whether it starts its row or has settled
/// before: every row is funded here, the branch makes room for every caller,
/// and the application never polls or retries. The settlement is then judged
/// on what it reports.
macro_rules! settled {
    ($court:expr, $demand:expr, $at:expr) => {{
        let before = query_entries();
        let settlement = match $demand.demand.advance($court.request) {
            Ok(WorthQueryApplicationOutputDemandProgress::Settled(settled)) => settled,
            answer => panic!(
                "{}: one advance settles the funded demand of {}; it answers {:?}",
                $at,
                $demand.body,
                answer.map(|_| "Pending")
            ),
        };
        let cost = Cost {
            producer_contacts: settlement.producer_contacts_in_this_demand(),
            source_queries: query_entries() - before,
        };
        $court.judge_settlement(&$demand.body, settlement.observation(), &$at);
        cost
    }};
}

macro_rules! root {
    ($court:expr, $key:expr, $at:expr) => {{
        let body: String = $key;
        let demand = $court
            .request
            .demand(PlanarOutputDemand::new(body.clone()))
            .start_in_program::<program::ChainProgram, program::ChainRoot>($court.application)
            .unwrap_or_else(|denial| {
                panic!("{}: the root demand of {body} starts: {denial:?}", $at)
            });
        Open { body, demand }
    }};
}

macro_rules! consumer {
    ($court:expr, $key:expr, $at:expr) => {{
        let body: String = $key;
        let demand = $court
            .request
            .demand(ChainDemand(body.clone()))
            .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(
                $court.application,
            )
            .unwrap_or_else(|denial| panic!("{}: the demand of {body} starts: {denial:?}", $at));
        Open { body, demand }
    }};
}

/// One request against one courtroom world.
struct Court<'court, 'application, 'principal, 'scope> {
    application: &'application Runtime,
    request: &'court Request<'application, 'principal, 'scope>,
    idempotency: Cell<u64>,
}

impl<'court, 'application, 'principal, 'scope> Court<'court, 'application, 'principal, 'scope> {
    fn new(
        application: &'application Runtime,
        request: &'court Request<'application, 'principal, 'scope>,
        idempotency: u64,
    ) -> Self {
        Self {
            application,
            request,
            idempotency: Cell::new(idempotency),
        }
    }
}

mod commits;
mod discontinuity;
mod equal_republication;
mod judges;
mod locality;
mod observation_room;
mod older_observation;
#[cfg(feature = "test-output-delivery-faults")]
mod other_entries;
mod randomized;
mod replaced_ready;
#[cfg(feature = "test-output-delivery-faults")]
mod undelivered_refresh;
