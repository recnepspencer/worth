//! The oracle of partition reuse: a program whose producer keeps a region
//! output over a set of entries, and how its tests seed, edit and demand it.
//! Every run the producer's decision makes is left in a room with how it ran
//! and which of the owner's calls it entered.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

use worth_query_host::facade::application_contribution::{
    WorthQueryComputationPartitionMembers, WorthQueryComputationPartitionPlan,
    WorthQueryComputationPartitionView, WorthQueryDeterministicReducer,
    WorthQueryManagedComputationCheckpoint, WorthQueryManagedComputationDenial,
    WorthQueryPartitionedComputationDenial, WorthQueryPartitionedComputationOwner,
    WorthQueryPartitionedComputationRun,
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationOutputDemandProgress,
    WorthQueryApplicationRequest,
};
use worth_query_host::facade::primary_graph::partitioned_computation_runs_on_this_thread_for_test as runs_on_this_thread;

use super::demand::{RegionTotalsDemandBinding, RegionTotalsHandler};
use super::entry_edit::{EntryEdit, EntryEditBinding, EntryEditHandler};
use super::facts::{self, Entry, EntryData, Graph, InputDenial, Reader, RegionEntry, Set};
use super::output_producer::{
    RegionOutputDemand, RegionOutputProducer, RegionOutputProvider, RegionOutputReadiness,
};
use super::region_output::{
    RegionOutputBinding, RegionOutputHandler, TotalRegionOutput, DECISION_FACT_BUDGET, LARGEST_SET,
};
use super::*;

mod counts;
mod differential;
mod parallel_history_reuse;
mod program;
mod worker_axis;

use program::{OracleRoot, RegionArtifact, RegionConnection, TOTALS_RETAINED_BYTES, TOTALS_WORK};

/// What one entry may cost the computation: its digest, its key's encoding
/// and routing, its kernel and its share of the combines.
const WORK_PER_ENTRY: usize = 512;
/// The input's digest.
const WORK_BESIDE_ENTRIES: usize = 4_096;
/// The width every other operation of the program fits, the host's default.
const WIDTH_BESIDE_DECISION: usize = 4_096;

/// The totals of one region output, declared for the largest set its
/// decision reads.
struct OracleTotals;
impl ApplicationManagedComputation<CheckpointSchema, PlanarFinalOutputFeature> for OracleTotals {
    type Input = RegionEntries;
    type Output = RegionArtifact;
    type Partition = RegionKey;
    type Reuse = NoWarmStart;
    type Stopped = RegionStopped;
    const IDENTITY: &'static str = "checkpoint-region-output-totals";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::DeterministicPartitioned;
    const RESOURCES: ApplicationComputationResourceCeiling =
        ApplicationComputationResourceCeiling::new(TOTALS_WORK, TOTALS_RETAINED_BYTES);
}

static PLANS: AtomicUsize = AtomicUsize::new(0);
static KEYS: AtomicUsize = AtomicUsize::new(0);
static GATHERS: AtomicUsize = AtomicUsize::new(0);
static KERNELS: AtomicUsize = AtomicUsize::new(0);

/// How often a decision's run entered each of the owner's calls.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OwnerCalls {
    plans: usize,
    keys: usize,
    gathers: usize,
    kernels: usize,
}

fn take_calls() -> OwnerCalls {
    OwnerCalls {
        plans: PLANS.swap(0, Ordering::Relaxed),
        keys: KEYS.swap(0, Ordering::Relaxed),
        gathers: GATHERS.swap(0, Ordering::Relaxed),
        kernels: KERNELS.swap(0, Ordering::Relaxed),
    }
}

/// Totals each region in entry order, then the regions. An even region also
/// sums the weight its set lends it, so the weight is a fact every even
/// region gathers.
struct OracleOwner;
impl WorthQueryPartitionedComputationOwner<CheckpointSchema, PlanarFinalOutputFeature, OracleTotals>
    for OracleOwner
{
    type Operation = TotalRegionOutput;
    type Item = Entry;
    type Gathered = Vec<EntryData>;
    type PartitionResult = f64;
    type Output = f64;
    type Stopped = u32;

    fn partitions(
        &self,
        reader: &mut Reader<'_, '_, '_, TotalRegionOutput>,
        set: &Set,
    ) -> Result<WorthQueryComputationPartitionPlan<Entry>, InputDenial> {
        PLANS.fetch_add(1, Ordering::Relaxed);
        Ok(facts::entries(reader, set)?)
    }

    fn partition_key(
        &self,
        reader: &mut Reader<'_, '_, '_, TotalRegionOutput>,
        _: &Set,
        entry: &Entry,
    ) -> Result<RegionKey, InputDenial> {
        KEYS.fetch_add(1, Ordering::Relaxed);
        Ok(RegionKey(facts::region(reader, entry)?))
    }

    fn gather(
        &self,
        reader: &mut Reader<'_, '_, '_, TotalRegionOutput>,
        set: &Set,
        partition: WorthQueryComputationPartitionMembers<'_, RegionKey, Entry>,
    ) -> Result<Vec<EntryData>, InputDenial> {
        GATHERS.fetch_add(1, Ordering::Relaxed);
        let mut gathered = Vec::new();
        if partition.key().0.is_multiple_of(2) {
            gathered.push(EntryData {
                value: facts::weight(reader, set)?,
                work: 1,
                fault: None,
            });
        }
        gathered.extend(facts::gathered(reader, partition.items())?);
        Ok(gathered)
    }

    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, RegionKey, Vec<EntryData>>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<f64, WorthQueryManagedComputationDenial<u32>> {
        KERNELS.fetch_add(1, Ordering::Relaxed);
        owner::total_region(partition.key().0, partition.gathered(), checkpoint)
    }

    fn reducer(&self) -> WorthQueryDeterministicReducer<f64> {
        WorthQueryDeterministicReducer::canonical(|| -0.0, |left, right| left + right)
    }

    fn complete(&self, reduced: f64) -> Result<f64, u32> {
        Ok(reduced)
    }
}

/// One run of the producer's decision: the total's bits and what the run was
/// charged, or its denial; how it ran; and the owner's calls it entered.
#[derive(Debug)]
struct OracleRun {
    outcome: Result<(u64, u64), WorthQueryPartitionedComputationDenial<u32>>,
    runs: Vec<WorthQueryPartitionedComputationRun>,
    calls: OwnerCalls,
}

/// The runs of the current demand. The tests that fill it hold the
/// checkpoint recovery guard.
static ROOM: Mutex<Vec<OracleRun>> = Mutex::new(Vec::new());

fn room() -> MutexGuard<'static, Vec<OracleRun>> {
    ROOM.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The topology plus the region output's producer and the owner of its
/// totals.
struct OracleContribution;
impl ApplicationSchemaContribution<CheckpointSchema> for OracleContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        <TopologyContribution as ApplicationSchemaContribution<CheckpointSchema>>::IDENTITY;
    fn register_members(
        builder: ApplicationSchemaDeclarationBuilder<CheckpointSchema>,
    ) -> ApplicationSchemaDeclarationBuilder<CheckpointSchema> {
        <TopologyContribution as ApplicationSchemaContribution<CheckpointSchema>>::register_members(
            builder,
        )
    }
}
impl WorthQueryApplicationContribution<CheckpointSchema> for OracleContribution {
    type Configuration = TopologyConfiguration;

    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<CheckpointSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        <TopologyContribution as WorthQueryApplicationContribution<CheckpointSchema>>::contracts(
            contracts,
        )?;
        contracts.producer::<RegionOutputProducer<CheckpointSchema>>()?;
        contracts.conditional::<RegionOutputReadiness<CheckpointSchema>>()?;
        Ok(())
    }

    fn configure(
        configuration: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, CheckpointSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        let installed = setup
            .partitioned_computation::<PlanarFinalOutputFeature, OracleTotals, _>(OracleOwner)?;
        let handler = RegionOutputHandler::running(move |reader, set| {
            take_calls();
            let outcome = (|| {
                let computed = installed
                    .prepare(reader, set)?
                    .compute(reader.managed_computation_execution())?;
                let charged_work = computed.charged_work();
                Ok((computed.complete()?.to_bits(), charged_work))
            })();
            let runs = runs_on_this_thread().into_iter().map(|(run, _)| run);
            room().push(OracleRun {
                outcome,
                runs: runs.collect(),
                calls: take_calls(),
            });
        });
        setup.handler::<RegionOutputBinding<CheckpointSchema>, _>(handler)?;
        setup.handler::<RegionTotalsDemandBinding<CheckpointSchema>, _>(
            RegionTotalsHandler::idle(),
        )?;
        setup.handler::<EntryEditBinding<CheckpointSchema>, _>(EntryEditHandler)?;
        setup.producer::<RegionOutputProducer<CheckpointSchema>>(RegionOutputProvider)?;
        setup.conditional::<RegionOutputReadiness<CheckpointSchema>>(())?;
        TopologyContribution::configure_topology(configuration, setup)
    }
}

struct OracleProgram;
impl ApplicationProgramDefinition<CheckpointSchema> for OracleProgram {
    type Contributions = (OracleContribution,);
    type Outputs = ApplicationProgramOutputs<OracleRoot>;
    type Rules = CheckpointRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("checkpoint-region-output-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        demand_policy::feature_specs_with_final_output(
            required_chain::output_feature_spec(),
            demand_policy::final_output_feature::<RegionArtifact>()
                .managed_computation::<OracleTotals>()
                .conditional_operation::<TotalRegionOutput>()
                .mutation::<EntryEditBinding<CheckpointSchema>>()
                .finish(),
        )
    }
}

type Application =
    application_installation::WorthQueryProgramApplicationRuntime<CheckpointSchema, OracleProgram>;
type Request<'application, 'principal, 'scope> =
    WorthQueryApplicationRequest<'application, 'principal, 'scope, CheckpointSchema>;

/// The scope whose output the producer keeps. Its ordinate names the set.
const SCOPE: &str = "anchor-isolated";
/// The scope's seeded ordinate, which names the even set, and the one that
/// names the odd set.
const EVEN_Y: u64 = 50;
const ODD_Y: u64 = 51;

/// Installs the default host, widened so its operations may be as wide as
/// the largest set's decision.
fn install(seed: impl FnOnce(&mut Graph)) -> Application {
    let width = u64::try_from(DECISION_FACT_BUDGET + WIDTH_BESIDE_DECISION).unwrap();
    let host = support::candidates();
    let candidates =
        worth_query_host::facade::runtime::WorthQueryApplicationCandidateResourceProfile::bounded(
            host.maximum_items().max(width),
            host.maximum_retained_representation_bytes().max(width),
            host.maximum_validator_work().max(width),
        )
        .and_then(|candidates| candidates.with_maximum_operation_width(width))
        .unwrap();
    const RETAINED_COMMITS: u64 = 32;
    let invalidation =
        support::invalidation(128 * 1_024 * 1_024, 1_000_000, RETAINED_COMMITS as usize);
    let limits = support::limits_with_room(RETAINED_COMMITS, 16, 64, invalidation, candidates);
    support::install_program_with_limits::<OracleProgram>(
        None,
        Default::default(),
        limits,
        |graph| {
            support::seed_cycle(graph);
            seed(graph);
        },
    )
}

/// Seeds the entry numbered `number` as a member of each of `sets`.
fn seed_entry(graph: &mut Graph, sets: &[&str], number: usize, entry: RegionEntry) {
    let key = format!("oracle-entry-{number}");
    facts::seed_entry(graph, &key, &entry);
    for set in sets {
        facts::seed_member(graph, set, &key);
    }
}

/// Demands the scope's output until it settles: the producer's contacts in
/// the demand, and the runs its decisions made.
fn demand(request: &Request<'_, '_, '_>, application: &Application) -> (usize, Vec<OracleRun>) {
    room().clear();
    let mut demand = request
        .demand(RegionOutputDemand(SCOPE.to_owned()))
        .start_dependent_in_program::<OracleProgram, RegionConnection>(application)
        .expect("the region output demand starts");
    let settled = (0..256)
        .find_map(|_| {
            match demand
                .advance(request)
                .expect("the region output demand advances")
            {
                WorthQueryApplicationOutputDemandProgress::Pending => None,
                WorthQueryApplicationOutputDemandProgress::Settled(settled) => Some(settled),
            }
        })
        .expect("the region output demand settles");
    let contacts = settled.producer_contacts_in_this_demand();
    (contacts, std::mem::take(&mut *room()))
}

/// Commits one entry edit.
fn edit(request: &Request<'_, '_, '_>, application: &Application, edit: EntryEdit, command: u64) {
    let observed = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the edit's source is readable");
    let made = format!("{edit:?}");
    let outcome = request
        .mutate(edit.commanded(command))
        .expect_source(observed.observed_sources()[0].clone())
        .idempotency(&command)
        .execute_in_program::<OracleProgram>(application);
    assert!(
        matches!(
            &outcome,
            Ok(WorthQueryApplicationMutationOutcome::Committed { .. })
        ),
        "the edit {made} commits: {outcome:?}"
    );
}

/// Moves the scope's ordinate to `y`, which names the set its output totals.
fn adjust(request: &Request<'_, '_, '_>, application: &Application, y: u64, command: u64) {
    let observed = request
        .query(PlanarRead {
            body_key: SCOPE.to_owned(),
        })
        .execute()
        .expect("the scope's source is readable");
    request
        .mutate(PlanarSourceAdjustment {
            scope_key: SCOPE.to_owned(),
            replacement_y: length(y),
        })
        .expect_source(observed.observed_sources()[0].clone())
        .idempotency(&command)
        .execute_performed::<OracleProgram, OracleRoot>(application)
        .expect("the scope's ordinate moves");
}
