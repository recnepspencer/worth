//! Declared pure member costs and an author-owned schedule witness.
use super::*;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use worth_query_decl::facade::application_program::*;
use worth_query_host::facade::application_contribution::*;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Failure {
    Prepare,
    Compute,
    Complete,
}
#[derive(Default)]
pub(super) struct State {
    pub entered: Vec<bool>,
    pub runs: Vec<usize>,
    pub finished: Vec<bool>,
    pub computed: Vec<usize>,
    pub completed: Vec<usize>,
    pub watchdog: bool,
}
type OwnerCompletion = Arc<dyn Fn(usize) + Send + Sync>;
pub(super) struct Schedule {
    pub roots: Vec<String>,
    pub costs: Vec<usize>,
    pub failure: Option<(usize, Failure)>,
    pub overlap: bool,
    pub catches_denial: bool,
    pub state: Mutex<State>,
    pub owner_completion: Mutex<Option<OwnerCompletion>>,
    ready: Condvar,
}
impl Schedule {
    pub fn new(roots: [String; 2], failure: Option<(usize, Failure)>, overlap: bool) -> Arc<Self> {
        Self::declared(roots.into(), COSTS.into(), failure, overlap)
    }
    pub fn declared(
        roots: Vec<String>,
        costs: Vec<usize>,
        failure: Option<(usize, Failure)>,
        overlap: bool,
    ) -> Arc<Self> {
        assert_eq!(roots.len(), costs.len());
        let count = roots.len();
        Arc::new(Self {
            roots,
            costs,
            failure,
            overlap,
            catches_denial: false,
            owner_completion: Mutex::new(None),
            state: Mutex::new(State {
                entered: vec![false; count],
                runs: vec![0; count],
                finished: vec![false; count],
                ..State::default()
            }),
            ready: Condvar::new(),
        })
    }
}
pub(super) fn ordinary_predecessor(input: &str) -> Option<String> {
    let armed = ARMED.lock().unwrap();
    let schedule = armed.as_ref()?;
    (input == schedule.roots[1]).then(|| schedule.roots[0].clone())
}

pub(super) fn catches_denial() -> bool {
    ARMED
        .lock()
        .unwrap()
        .as_ref()
        .is_some_and(|schedule| schedule.catches_denial)
}

static ARMED: Mutex<Option<Arc<Schedule>>> = Mutex::new(None);
pub(super) fn arm(schedule: Option<Arc<Schedule>>) {
    *ARMED.lock().unwrap() = schedule;
}
pub(super) const COSTS: [usize; 2] = [3, 7];
pub(super) struct Input;
impl ApplicationComputationInput for Input {
    type Value = String;
    const IDENTITY: &'static str = "wave-source";
}
pub(super) struct Reuse;
impl ApplicationComputationReuse for Reuse {
    const IDENTITY: &'static str = "wave-exact";
}
pub(super) struct Stop;
impl ApplicationComputationStopped for Stop {
    const IDENTITY: &'static str = "wave-declared-failure";
}
pub(super) struct Computation;
impl ApplicationManagedComputation<CheckpointSchema, PlanarOutputFeature> for Computation {
    type Input = Input;
    type Output = Artifact;
    type Partition = ApplicationSingleComputationPartition;
    type Reuse = Reuse;
    type Stopped = Stop;
    const IDENTITY: &'static str = "wave-computation";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::Deterministic;
    const RESOURCES: ApplicationComputationResourceCeiling =
        ApplicationComputationResourceCeiling::new(128, 512);
}
pub(super) struct Prepared {
    rank: usize,
    schedule: Option<Arc<Schedule>>,
}
impl WorthQueryManagedComputationPrepared for Prepared {
    fn retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
pub(super) struct Owner;
impl WorthQueryManagedComputationOwner<CheckpointSchema, PlanarOutputFeature, Computation>
    for Owner
{
    type Prepared = Prepared;
    type Computed = u64;
    type Output = CostValue;
    type Stopped = ();
    fn prepare(&self, input: &String) -> Result<Prepared, ()> {
        // Counts witness only declared roots; an unrelated kernel returns zero
        // without joining this schedule or contributing to its run counts.
        let schedule = ARMED
            .lock()
            .unwrap()
            .clone()
            .filter(|schedule| schedule.roots.iter().any(|key| key == input));
        let rank = schedule
            .as_ref()
            .and_then(|s| s.roots.iter().position(|key| key == input))
            .unwrap_or(0);
        if schedule
            .as_ref()
            .is_some_and(|s| s.failure == Some((rank, Failure::Prepare)))
        {
            return Err(());
        }
        Ok(Prepared { rank, schedule })
    }
    fn compute(
        &self,
        prepared: &Prepared,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<u64, WorthQueryManagedComputationDenial<()>> {
        let Some(schedule) = &prepared.schedule else {
            return Ok(0);
        };
        let rank = prepared.rank;
        {
            let mut state = schedule.state.lock().unwrap();
            state.entered[rank] = true;
            state.runs[rank] += 1;
            if schedule.overlap && rank == 0 {
                // Failure watchdog only: a passing run wakes on the later member.
                let (next, guard) = schedule
                    .ready
                    .wait_timeout_while(state, Duration::from_secs(60), |s| {
                        !s.finished[1..].iter().all(|finished| *finished)
                    })
                    .unwrap();
                state = next;
                state.watchdog |= guard.timed_out();
            }
        }
        checkpoint
            .advance(schedule.costs[rank])
            .map_err(WorthQueryManagedComputationDenial::from)?;
        {
            let mut state = schedule.state.lock().unwrap();
            state.finished[rank] = true;
            state.computed.push(rank);
            schedule.ready.notify_all();
        }
        if schedule.failure == Some((rank, Failure::Compute)) {
            Err(WorthQueryManagedComputationDenial::Owner(()))
        } else {
            Ok(schedule.costs[rank] as u64)
        }
    }
    fn complete(&self, prepared: Prepared, computed: u64) -> Result<CostValue, ()> {
        if let Some(schedule) = prepared.schedule {
            schedule.state.lock().unwrap().completed.push(prepared.rank);
            if schedule.failure == Some((prepared.rank, Failure::Complete)) {
                return Err(());
            }
            let probe = schedule.owner_completion.lock().unwrap().clone();
            if let Some(probe) = probe {
                probe(prepared.rank);
            }
        }
        Ok(CostValue { units: computed })
    }
}

pub(super) struct CostValue {
    pub units: u64,
}
pub(super) struct Locality;
impl ApplicationLocalityScope for Locality {
    const IDENTITY: &'static str = "wave-member";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Root;
}
pub(super) struct Artifact;
impl ApplicationDerivedArtifact<CheckpointSchema, PlanarOutputFeature> for Artifact {
    type Output = PlanarDerivedBodyOutput;
    type Locality = Locality;
    const IDENTITY: &'static str = "wave-declared-cost";
    const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Retained;
    const SUCCESSION: ApplicationArtifactSuccession = ApplicationArtifactSuccession::Recompute;
    const REQUIRED: bool = true;
    const PRODUCER_FAMILY: &'static str =
        <PlanarOutputFamily as WorthQueryProducerOutputFamily<CheckpointSchema>>::IDENTITY;
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] =
        &[ApplicationArtifactDependency::new(
            <PlanarSourceFeature as ApplicationFeature<CheckpointSchema>>::IDENTITY,
        )];
    const REUSE_RULE: &'static str = "wave-exact";
    const RESOURCE_CEILING: ApplicationArtifactResourceCeiling =
        ApplicationArtifactResourceCeiling::new(Some(4096), 8192);
    const STOPPED_OUTCOME: &'static str = "wave-declared-failure";
}
