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
    pub entered: [bool; 2],
    pub runs: [usize; 2],
    pub finished: [bool; 2],
    pub completed: Vec<usize>,
    pub watchdog: bool,
}
pub(super) struct Schedule {
    pub roots: [String; 2],
    pub failure: Option<(usize, Failure)>,
    pub overlap: bool,
    pub state: Mutex<State>,
    ready: Condvar,
}
impl Schedule {
    pub fn new(roots: [String; 2], failure: Option<(usize, Failure)>, overlap: bool) -> Arc<Self> {
        Arc::new(Self {
            roots,
            failure,
            overlap,
            state: Mutex::new(State::default()),
            ready: Condvar::new(),
        })
    }
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
        let schedule = ARMED.lock().unwrap().clone();
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
                    .wait_timeout_while(state, Duration::from_secs(60), |s| !s.finished[1])
                    .unwrap();
                state = next;
                state.watchdog |= guard.timed_out();
            }
        }
        checkpoint
            .advance(COSTS[rank])
            .map_err(WorthQueryManagedComputationDenial::from)?;
        {
            let mut state = schedule.state.lock().unwrap();
            state.finished[rank] = true;
            schedule.ready.notify_all();
        }
        if schedule.failure == Some((rank, Failure::Compute)) {
            Err(WorthQueryManagedComputationDenial::Owner(()))
        } else {
            Ok(COSTS[rank] as u64)
        }
    }
    fn complete(&self, prepared: Prepared, computed: u64) -> Result<CostValue, ()> {
        if let Some(schedule) = prepared.schedule {
            schedule.state.lock().unwrap().completed.push(prepared.rank);
            if schedule.failure == Some((prepared.rank, Failure::Complete)) {
                return Err(());
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
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] = &[];
    const REUSE_RULE: &'static str = "wave-exact";
    const RESOURCE_CEILING: ApplicationArtifactResourceCeiling =
        ApplicationArtifactResourceCeiling::new(4096, 8192);
    const STOPPED_OUTCOME: &'static str = "wave-declared-failure";
}
