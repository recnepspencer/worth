//! The partitioned owner of the region totals, and the handler that runs it
//! under a real request.

use std::sync::{Mutex, MutexGuard, PoisonError};

use worth_foundational::facade::ExecutionReport;
use worth_query_consumer_values::{PlanarAdjustmentResult, PlanarMutationDenial};
use worth_query_decl::facade::application_operation::ApplicationCandidateRequirements;
use worth_query_host::facade::application_contribution::{
    PartitionItemId, WorthQueryComputationPartitionPlan, WorthQueryComputationPartitionView,
    WorthQueryDeterministicReducer, WorthQueryManagedComputationCheckpoint,
    WorthQueryManagedComputationDenial, WorthQueryManagedComputationExecution,
    WorthQueryPartitionedComputationDenial, WorthQueryPartitionedComputationOwner,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerResult, OperationHandler,
    WorthQueryInvariantMutationTarget,
};

use super::*;

/// One value of the input.
#[derive(Clone, Copy, Debug)]
pub(super) struct RegionEntry {
    /// The entry's own identity, whatever its place in the input.
    pub(super) id: u64,
    pub(super) region: u32,
    pub(super) value: f64,
    /// Work the region's kernel charges before it reads the value.
    pub(super) work: usize,
    pub(super) fault: Option<RegionFault>,
}

/// What a region's kernel does instead of reading an entry.
#[derive(Clone, Copy, Debug)]
pub(super) enum RegionFault {
    /// Refuses the region, naming it.
    Refuse,
    Panic,
    /// The probing owner's faults. The totals owner reads through them.
    Probe(super::probe::ProbeFault),
}

/// Sums each region's values in entry identity order, then the regions' sums.
/// Addition's identity is negative zero: adding it changes no sum, and keeps
/// the sign of a sum that is itself a zero.
pub(super) struct RegionTotalsOwner;
impl<Computation>
    WorthQueryPartitionedComputationOwner<CheckpointSchema, PlanarFinalOutputFeature, Computation>
    for RegionTotalsOwner
where
    Computation: ApplicationManagedComputation<
        CheckpointSchema,
        PlanarFinalOutputFeature,
        Input = RegionEntries,
        Partition = RegionKey,
    >,
{
    type PartitionResult = f64;
    type Output = f64;
    type Stopped = u32;

    fn partitions(
        &self,
        entries: &<RegionEntries as ApplicationComputationInput>::Value,
    ) -> WorthQueryComputationPartitionPlan<RegionKey> {
        WorthQueryComputationPartitionPlan::keyed(
            entries,
            |entry| PartitionItemId(entry.id),
            |entry| RegionKey(entry.region),
        )
    }

    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<
            '_,
            RegionKey,
            <RegionEntries as ApplicationComputationInput>::Value,
        >,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<f64, WorthQueryManagedComputationDenial<u32>> {
        let mut total = -0.0;
        for item in partition.items() {
            let entry = partition.input()[item.position()];
            checkpoint.advance(entry.work)?;
            match entry.fault {
                Some(RegionFault::Refuse) => {
                    return Err(WorthQueryManagedComputationDenial::Owner(partition.key().0))
                }
                Some(RegionFault::Panic) => panic!("the region kernel panics"),
                Some(RegionFault::Probe(_)) | None => total += entry.value,
            }
        }
        Ok(total)
    }

    fn reducer(&self) -> WorthQueryDeterministicReducer<f64> {
        WorthQueryDeterministicReducer::canonical(|| -0.0, |left, right| left + right)
    }

    fn complete(&self, reduced: f64) -> Result<f64, u32> {
        Ok(reduced)
    }
}

/// The partitioned computation bound to its partitioned owner.
pub(super) struct PartitionedOwner;
impl RegionTotalsBinding for PartitionedOwner {
    type Computation = RegionTotals;

    fn install(setup: &mut Setup<'_>) -> Installed {
        RegionTotalsHandler::running::<RegionTotals, _>(setup, RegionTotalsOwner)
    }
}

/// The same owner under a computation whose declared bytes are the largest
/// a declaration can name.
pub(super) struct UnboundedBytesOwner;
impl RegionTotalsBinding for UnboundedBytesOwner {
    type Computation = UnboundedBytesTotals;

    fn install(setup: &mut Setup<'_>) -> Installed {
        RegionTotalsHandler::running::<UnboundedBytesTotals, _>(setup, RegionTotalsOwner)
    }
}

pub(super) type Entries = <RegionEntries as ApplicationComputationInput>::Value;
pub(super) type Setup<'setup> = WorthQueryApplicationContributionSetup<'setup, CheckpointSchema>;
pub(super) type Installed =
    Result<RegionTotalsHandler, primary_graph::WorthQueryPrimaryGraphInstallationDenial>;

/// A completed run: the total's bits and what the run was charged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RegionTotal {
    pub(super) bits: u64,
    pub(super) charged_work: u64,
    pub(super) report: ExecutionReport,
}

pub(super) type RegionOutcome = Result<RegionTotal, WorthQueryPartitionedComputationDenial<u32>>;

/// The entries the next decision totals, and what each decision's run came to.
/// The tests that use it hold the checkpoint recovery guard.
struct RegionRoom {
    entries: Vec<RegionEntry>,
    outcomes: Vec<RegionOutcome>,
    /// The partitions whose kernel the last demand entered.
    kernels: usize,
}

static ROOM: Mutex<RegionRoom> = Mutex::new(RegionRoom {
    entries: Vec::new(),
    outcomes: Vec::new(),
    kernels: 0,
});

fn room() -> MutexGuard<'static, RegionRoom> {
    ROOM.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A kernel says it was entered.
pub(super) fn enter_kernel() {
    room().kernels += 1;
}

/// The partitions whose kernel the last demand entered.
pub(super) fn kernels_entered() -> usize {
    room().kernels
}

type Run =
    Box<dyn Fn(&Entries, WorthQueryManagedComputationExecution<'_>) -> RegionOutcome + Send + Sync>;

/// Decides the planar source adjustment as the topology does, after running
/// the region totals with the deciding request's execution.
pub(super) struct RegionTotalsHandler(Option<Run>);

impl RegionTotalsHandler {
    /// The topology's own decision: no installed owner to run.
    pub(super) const fn idle() -> Self {
        Self(None)
    }

    /// Installs the computation's partitioned owner and runs it at every
    /// decision.
    pub(super) fn running<Computation, Owner>(setup: &mut Setup<'_>, owner: Owner) -> Installed
    where
        Computation: ApplicationManagedComputation<
            CheckpointSchema,
            PlanarFinalOutputFeature,
            Input = RegionEntries,
        >,
        Owner: WorthQueryPartitionedComputationOwner<
            CheckpointSchema,
            PlanarFinalOutputFeature,
            Computation,
            Output = f64,
            Stopped = u32,
        >,
    {
        let installed =
            setup.partitioned_computation::<PlanarFinalOutputFeature, Computation, _>(owner)?;
        Ok(Self(Some(Box::new(
            move |entries: &Entries, execution: WorthQueryManagedComputationExecution<'_>| {
                let computed = installed.prepare(entries)?.compute(execution)?;
                let charged_work = computed.charged_work();
                let report = computed.execution_report();
                Ok(RegionTotal {
                    bits: computed.complete()?.to_bits(),
                    charged_work,
                    report,
                })
            },
        ))))
    }
}

impl OperationHandler<CheckpointSchema, PlanarSourceAdjustmentBinding<CheckpointSchema>>
    for RegionTotalsHandler
{
    fn decide(
        &self,
        input: &PlanarSourceAdjustment,
        reader: &mut DecisionReader<
            '_,
            '_,
            '_,
            CheckpointSchema,
            PlanarSourceAdjustmentBinding<CheckpointSchema>,
        >,
    ) -> HandlerResult<
        WorthQueryInvariantMutationTarget<CheckpointSchema, Body>,
        PlanarMutationDenial,
    > {
        if let Some(run) = &self.0 {
            let entries = room().entries.clone();
            let outcome = run(&entries, reader.managed_computation_execution());
            room().outcomes.push(outcome);
        }
        PlanarSourceAdjustmentHandler.decide(input, reader)
    }

    fn candidate_requirements(
        &self,
        input: &PlanarSourceAdjustment,
        target: &WorthQueryInvariantMutationTarget<CheckpointSchema, Body>,
    ) -> ApplicationCandidateRequirements {
        OperationHandler::<CheckpointSchema, PlanarSourceAdjustmentBinding<CheckpointSchema>>::candidate_requirements(
            &PlanarSourceAdjustmentHandler,
            input,
            target,
        )
    }

    fn build_candidate(
        &self,
        input: &PlanarSourceAdjustment,
        target: WorthQueryInvariantMutationTarget<CheckpointSchema, Body>,
        writer: &mut CandidateWriter<
            '_,
            CheckpointSchema,
            PlanarSourceAdjustmentBinding<CheckpointSchema>,
        >,
    ) -> HandlerResult<PlanarAdjustmentResult, PlanarMutationDenial> {
        PlanarSourceAdjustmentHandler.build_candidate(input, target, writer)
    }
}

/// The region totals under their partitioned owner.
pub(super) fn with_region_totals(
    test: impl FnOnce(&mut dyn FnMut(&[RegionEntry]) -> RegionOutcome),
) {
    with_totals::<PartitionedOwner>(test);
}

/// Installs the binding's program and hands the test a demand: each call
/// totals the given entries inside one performed source adjustment and returns
/// what that decision's run came to.
pub(super) fn with_totals<Binding: RegionTotalsBinding>(
    test: impl FnOnce(&mut dyn FnMut(&[RegionEntry]) -> RegionOutcome),
) {
    let _guard = checkpoint_recovery_test_guard();
    let application =
        support::install_program::<RegionTotalsProgram<Binding>>(None, Default::default());
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut demands = 0_u64;
    test(&mut |entries| {
        demands += 1;
        {
            let mut room = room();
            room.entries = entries.to_vec();
            room.outcomes.clear();
            room.kernels = 0;
        }
        let observed = request
            .query(PlanarRead {
                body_key: "anchor-a".to_owned(),
            })
            .execute()
            .expect("the adjusted source is readable");
        // Every demand moves the source, so every decision is a new one.
        request
            .mutate(PlanarSourceAdjustment {
                scope_key: "anchor-a".to_owned(),
                replacement_y: length(2 + demands % 2),
            })
            .expect_source(observed.observed_sources()[0].clone())
            .idempotency(&demands)
            .execute_performed::<RegionTotalsProgram<Binding>, CheckpointRoot>(&application)
            .expect("the source adjustment performs whatever the totals came to");
        let mut outcomes = std::mem::take(&mut room().outcomes);
        assert_eq!(outcomes.len(), 1, "one decision runs the totals once");
        outcomes.remove(0)
    });
}
