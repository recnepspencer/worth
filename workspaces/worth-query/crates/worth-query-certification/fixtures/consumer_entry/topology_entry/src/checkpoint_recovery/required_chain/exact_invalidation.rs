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
    WorthQueryApplicationRequestQueryDenial,
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
    /// A window no courtroom history outgrows. Every live version of a mark
    /// root reserves a conservative bound for its whole window, so the
    /// reservation ceiling grows with the square of the window.
    const AMPLE: Self = Self {
        bytes: 1 << 40,
        commit_positions: 512,
    };
}

/// A courtroom world over `seed`, with room for a hundred rings to keep their
/// three demands open and for the rows a random history leaves behind.
fn install(
    checkpoint: Option<application_installation::WorthQueryApplicationCheckpoint>,
    seed: Seed,
    retained: Retained,
) -> Runtime {
    install_observing(checkpoint, seed, retained, 64)
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
    advances: usize,
    producer_contacts: usize,
    source_queries: u64,
}

impl Cost {
    /// A clean output: one advance, no producer contact, and no source query
    /// ran again.
    fn is_free(&self) -> bool {
        self.advances == 1 && self.producer_contacts == 0 && self.source_queries == 0
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
    /// Whether the demand has settled before.
    settled: bool,
}

/// The most advances that settle a demand which starts its own row: it walks
/// the start stages of an equal republication, or of a producer contact.
const START_STAGES: [usize; 2] = [3, 5];

/// One advance settles `$demand`: every row is funded here, and the
/// application never polls. A demand that has not settled before may walk
/// the start stages of a row it produces itself, and only the one stop the
/// court's world is sized to meet is asked again. The settlement is then
/// judged on what it reports.
macro_rules! settled {
    ($court:expr, $demand:expr, $at:expr) => {{
        let before = query_entries();
        let starts = !std::mem::replace(&mut $demand.settled, true);
        let mut advances = 0;
        let settlement = loop {
            advances += 1;
            match $demand.demand.advance($court.request) {
                Ok(WorthQueryApplicationOutputDemandProgress::Settled(settled)) => break settled,
                Ok(WorthQueryApplicationOutputDemandProgress::Pending)
                    if starts && advances < 64 => {}
                Err(stop) if advances < 64 && $court.retries(&stop) => {}
                answer => panic!(
                    "{}: one advance settles the funded demand of {}; advance {advances} answers {:?}",
                    $at,
                    $demand.body,
                    answer.map(|_| "Pending")
                ),
            }
        };
        let cost = Cost {
            advances,
            producer_contacts: settlement.producer_contacts_in_this_demand(),
            source_queries: query_entries() - before,
        };
        assert!(
            advances == 1
                || $court.retried.is_some()
                || starts && advances <= START_STAGES[usize::from(cost.producer_contacts != 0)],
            "{}: only the start of a row takes the demand of {} more than one advance: {cost:?}",
            $at,
            $demand.body
        );
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
        Open {
            body,
            demand,
            settled: false,
        }
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
        Open {
            body,
            demand,
            settled: false,
        }
    }};
}

/// One request against one courtroom world.
struct Court<'court, 'application, 'principal, 'scope> {
    application: &'application Runtime,
    request: &'court Request<'application, 'principal, 'scope>,
    idempotency: Cell<u64>,
    /// The one retryable stop this world is sized to make a funded demand or
    /// a read meet, and how often one did.
    retried: Option<WorthQueryOutputDemandDenialKind>,
    refusals: Cell<usize>,
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
            retried: None,
            refusals: Cell::new(0),
        }
    }
}

impl Court<'_, '_, '_, '_> {
    /// Whether `stop` is the retryable stop this world is sized to meet.
    fn retries(&self, stop: &WorthQueryApplicationOutputDemandDenial) -> bool {
        let named = offers_retry(stop)
            && matches!(
                stop,
                WorthQueryApplicationOutputDemandDenial::Demand(denial)
                    if Some(denial.kind()) == self.retried
            );
        self.refusals.set(self.refusals.get() + usize::from(named));
        named
    }

    /// Runs `read`. A read refused what this world is sized to refuse is
    /// asked again, and a retry is admitted.
    fn read<Output>(
        &self,
        mut read: impl FnMut() -> Result<Output, WorthQueryApplicationRequestQueryDenial>,
    ) -> Output {
        use primary_graph::{
            WorthQueryApplicationQueryAdmissionDenialKind as Admission,
            WorthQueryOperationAuthorizationDenialKind as Authorization,
        };
        for _ in 0..64 {
            match read() {
                Ok(output) => return output,
                Err(WorthQueryApplicationRequestQueryDenial::Admission(refused))
                    if matches!(
                        refused.kind(),
                        Admission::Authorization(Authorization::ProductSecurityBasis(product))
                            if Some(WorthQueryOutputDemandDenialKind::ProductSelection(product))
                                == self.retried
                    ) =>
                {
                    self.refusals.set(self.refusals.get() + 1);
                }
                Err(denial) => panic!("a courtroom read is refused: {denial:?}"),
            }
        }
        panic!("a read refused a product observation is never admitted again")
    }
}

mod commits;
mod discontinuity;
mod equal_republication;
mod judges;
mod locality;
mod other_entries;
mod randomized;
mod replaced_ready;
