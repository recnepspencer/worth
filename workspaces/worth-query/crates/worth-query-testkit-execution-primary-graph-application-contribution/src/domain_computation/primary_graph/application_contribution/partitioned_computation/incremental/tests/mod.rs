//! A producer's next run of a partitioned computation, through a real
//! admitted operation. The test hands each attempt the prior a producer would
//! and keeps the run seal retains; a moved fact is the retained fact standing
//! in for an older one.

use worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation as QueryObservation;
use std::sync::Arc;
use std::sync::Mutex;
use worth_execution::PartitionItemId;
use worth_foundational::facade::ExecutionReport;
use worth_runtime_world::facade::RuntimeWorldExecutionPlacement;
use worth_query_declaration::facade::application_program::ApplicationManagedComputation;
use crate::domain_computation::primary_graph::application_contribution::partitioned_computation::attribution_tests::Computation;
use crate::domain_computation::primary_graph::application_contribution::partitioned_computation::attribution_tests::Feature;
use crate::domain_computation::primary_graph::application_contribution::partitioned_computation::attribution_tests::Input;
use crate::domain_computation::primary_graph::application_contribution::partitioned_computation::attribution_tests::Number;
use crate::domain_computation::primary_graph::application_contribution::partitioned_computation::attribution_tests::Parity;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::installed::ComputationRetention;
use worth_query_execution::facade::application_contribution::WorthQueryComputationInputDenial;
use worth_query_execution::facade::application_contribution::WorthQueryComputationPartitionMembers;
use worth_query_execution::facade::application_contribution::WorthQueryComputationPartitionPlan;
use worth_query_execution::facade::application_contribution::WorthQueryComputationPartitionView;
use worth_query_execution::facade::application_contribution::WorthQueryComputationReader;
use worth_query_execution::facade::application_contribution::WorthQueryDeterministicReducer;
use worth_query_execution::facade::application_contribution::WorthQueryInstalledPartitionedComputation;
use worth_query_execution::facade::application_contribution::WorthQueryPartitionedComputationDenial;
use worth_query_execution::facade::application_contribution::WorthQueryPartitionedComputationOwner;
use worth_query_execution::facade::domain_computation::partitioned_computation_runs_on_this_thread_for_test as runs;
use worth_query_execution::facade::domain_computation::partitioned_computation_tree_work_on_this_thread_for_test as tree_runs;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::carriage::ComputationPrior;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::retained::SealedComputationRun;
use worth_query_execution::facade::application_contribution::WorthQueryPartitionedComputationFullCause as Cause;
use worth_query_execution::facade::application_contribution::WorthQueryPartitionedComputationRunForTest as Run;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::computation_routing::ComputationRead;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::registry::semantic_edition::InstalledProducerEdition;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::QueryRequestExecution;
use crate::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use crate::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use crate::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use crate::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use crate::domain_computation::primary_graph::tests::fixture::Account;
use crate::domain_computation::primary_graph::tests::fixture::AccountLabel;
use crate::domain_computation::primary_graph::tests::fixture::AccountStatus;
use crate::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema as Schema;
use crate::domain_computation::primary_graph::tests::fixture::TouchAccountOperation;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantEntityIdentity;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationCheckpoint;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationDenial;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationExecution;
pub type Reader<'call, 'reader, 'runtime> =
    WorthQueryComputationReader<'call, 'reader, 'runtime, Schema, TouchAccountOperation>;
pub type Root = WorthQueryInvariantEntityIdentity<Schema, Account>;
pub type Installed = WorthQueryInstalledPartitionedComputation<Schema, Feature, Computation, Owner>;
pub type Outcome = Result<(u64, u64), WorthQueryPartitionedComputationDenial<u32>>;
#[derive(Clone, Copy)]
pub enum StatusRead {
    /// The gather of the partition of this parity.
    Gather(u64),
    ItemKeys,
}
pub struct Owner {
    pub modulus: u64,
    pub status: StatusRead,
    pub reducer: fn(&u64, &u64) -> u64,
    pub bump: Mutex<u64>,
    /// The work of each kernel, odd parity first.
    pub work: Mutex<[usize; 2]>,
    pub steps: Mutex<Option<[Vec<usize>; 2]>>,
    pub nested: std::sync::atomic::AtomicBool,
    pub gathered: Mutex<Vec<u64>>,
    pub calls: Mutex<[usize; 4]>,
}
impl WorthQueryPartitionedComputationOwner<Schema, Feature, Computation> for Owner {
    type Operation = TouchAccountOperation;
    type Item = Number;
    type Gathered = u64;
    type PartitionResult = u64;
    type Output = u64;
    type Stopped = u32;

    fn partitions(
        &self,
        _: &mut Reader<'_, '_, '_>,
        _: &Root,
    ) -> Result<WorthQueryComputationPartitionPlan<Number>, WorthQueryComputationInputDenial<u32>>
    {
        self.calls.lock().unwrap()[0] += 1;
        Ok(WorthQueryComputationPartitionPlan::keyed(
            [1, 2, 3, 4].map(Number),
            |item| PartitionItemId(item.0),
        ))
    }

    fn partition_key(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        account: &Root,
        item: &Number,
    ) -> Result<Parity, WorthQueryComputationInputDenial<u32>> {
        self.calls.lock().unwrap()[1] += 1;
        if matches!(self.status, StatusRead::ItemKeys) {
            reader.field(account, AccountStatus::reference())?;
        }
        Ok(Parity(item.0 % self.modulus))
    }

    fn gather(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        account: &Root,
        partition: WorthQueryComputationPartitionMembers<'_, Parity, Number>,
    ) -> Result<u64, WorthQueryComputationInputDenial<u32>> {
        self.calls.lock().unwrap()[2] += 1;
        reader.field(account, AccountLabel::reference())?;
        let parity = partition.key().0;
        let mut sum = partition.items().map(|(_, item)| item.0).sum::<u64>();
        if matches!(self.status, StatusRead::Gather(status) if status == parity) {
            reader.field(account, AccountStatus::reference())?;
            sum += *self.bump.lock().unwrap();
        }
        self.gathered.lock().unwrap().push(parity);
        Ok(sum)
    }

    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, Parity, u64>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<u64, WorthQueryManagedComputationDenial<u32>> {
        self.calls.lock().unwrap()[3] += 1;
        checkpoint.poll()?;
        if self.nested.load(std::sync::atomic::Ordering::Relaxed) {
            let map = worth_execution::ExecutionMap::<_, u64>::from_keyless_partitions(
                std::collections::BTreeMap::from([
                    (
                        worth_foundational::PartitionIdentity::new(1),
                        worth_execution::KeylessPartition {
                            value: 2_u64,
                            kernel_scratch_bytes: 0,
                            max_result_bytes: 0,
                        },
                    ),
                    (
                        worth_foundational::PartitionIdentity::new(2),
                        worth_execution::KeylessPartition {
                            value: 3_u64,
                            kernel_scratch_bytes: 0,
                            max_result_bytes: 0,
                        },
                    ),
                ]),
            )
            .unwrap();
            assert!(
                matches!(
                    map.run(
                        Some(
                            checkpoint
                                .native_lease_for_test()
                                .expect("the kernel lends its actual leased backing")
                        ),
                        |units, context| {
                            context.checkpoint(*units)?;
                            Ok::<_, worth_execution::MapKernelFailure<()>>(*units)
                        }
                    ),
                    worth_execution::MapOutcome::Complete { .. }
                ),
                "the nested public map computes inside its parent's meter"
            );
        }
        let work = self.work.lock().unwrap()[usize::from(partition.key().0 == 0)];
        match &*self.steps.lock().unwrap() {
            Some(steps) => {
                for step in &steps[usize::from(partition.key().0 == 0)] {
                    checkpoint.advance(*step)?;
                }
            }
            _ => {
                checkpoint.advance(work)?;
            }
        }
        Ok(*partition.gathered())
    }

    fn reducer(&self) -> WorthQueryDeterministicReducer<u64> {
        WorthQueryDeterministicReducer::canonical(|| 0, self.reducer)
    }

    fn complete(&self, reduced: u64) -> Result<u64, u32> {
        Ok(reduced)
    }
}
pub fn installed(status: StatusRead, reducer: fn(&u64, &u64) -> u64) -> Installed {
    installed_over(2, status, reducer)
}
pub fn installed_over(
    modulus: u64,
    status: StatusRead,
    reducer: fn(&u64, &u64) -> u64,
) -> Installed {
    let owner = Owner {
        modulus,
        status,
        reducer,
        bump: Mutex::new(0),
        work: Mutex::new([1, 1]),
        steps: Mutex::new(None),
        nested: Default::default(),
        gathered: Mutex::default(),
        calls: Mutex::default(),
    };
    Installed::new(owner, ComputationRetention::ProducerOperation)
}
pub fn sum(left: &u64, right: &u64) -> u64 {
    left + right
}
pub fn edition() -> InstalledProducerEdition {
    InstalledProducerEdition::for_test_for_test([7; 32])
}
pub struct Attempt {
    pub outcome: Outcome,
    pub bill: Option<ExecutionReport>,
    /// The work the projection's reader charged.
    pub work:
        worth_query_execution::facade::application_invariants::WorthQueryInvariantProjectionWork,
    pub runs: Vec<(Run, Option<ExecutionReport>)>,
    pub tree_runs:
        Vec<worth_query_execution::facade::application_contribution::WorthQueryPartitionedTreeRun>,
    pub gathered: Vec<u64>,
    pub sealed: Result<Option<SealedComputationRun>, ()>,
}
pub trait TestOwner<Tested: ApplicationManagedComputation<Schema, Feature> = Computation>:
    WorthQueryPartitionedComputationOwner<
    Schema,
    Feature,
    Tested,
    Operation = TouchAccountOperation,
    Item = Number,
    Gathered = u64,
    PartitionResult = u64,
    Output = u64,
    Stopped = u32,
>
{
    /// The parities its gathers met, in call order.
    fn gathered(&self) -> &Mutex<Vec<u64>>;
}
impl TestOwner for Owner {
    fn gathered(&self) -> &Mutex<Vec<u64>> {
        &self.gathered
    }
}
pub fn attempt<Tested, Owned>(
    world: &AuthorizationWorld,
    installed: &WorthQueryInstalledPartitionedComputation<Schema, Feature, Tested, Owned>,
    prior: Option<ComputationPrior>,
) -> Attempt
where
    Tested: ApplicationManagedComputation<Schema, Feature, Input = Input>,
    Owned: TestOwner<Tested>,
{
    attempt_in(world, installed, prior, &live_scope())
}
pub fn attempt_in<Tested, Owned>(
    world: &AuthorizationWorld,
    installed: &WorthQueryInstalledPartitionedComputation<Schema, Feature, Tested, Owned>,
    prior: Option<ComputationPrior>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> Attempt
where
    Tested: ApplicationManagedComputation<Schema, Feature, Input = Input>,
    Owned: TestOwner<Tested>,
{
    attempt_placed(
        world,
        installed,
        prior,
        request,
        RuntimeWorldExecutionPlacement::Serial(
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::test_placement::test_policy(
            std::num::NonZeroUsize::MIN, None,
        ),
    ),
    )
}
use crate::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::projected_attempt::attempt_placed;
pub fn prior_of(first: Attempt, moved: bool) -> ComputationPrior {
    let mut state = (*first
        .sealed
        .unwrap()
        .expect("a producer's run retains")
        .state_for_test());
    if moved {
        let (key, fact) = state
            .facts
            .facts()
            .find(|(_, _, readers)| {
                let partitioner = readers
                    .reads()
                    .any(|read| !matches!(read, ComputationRead::Partition(_)));
                partitioner || readers.partitions().len() == 1
            })
            .map(|(key, fact, _)| (key.clone(), fact.clone()))
            .expect("one owner call reads the status");
        let WorthQueryApplicationObservedFact::Field { entity_id, .. } = fact else {
            panic!("the status is a field's fact: {fact:?}");
        };
        state.facts.replace_fact(
            &key,
            WorthQueryApplicationObservedFact::SourceEntity { entity_id },
        );
    }
    ComputationPrior::new_for_test(edition(), Ok(std::sync::Arc::new(state)), None)
}
pub fn first_run(world: &AuthorizationWorld, installed: &Installed) -> Attempt {
    let first = attempt(
        world,
        installed,
        Some(ComputationPrior::new_for_test(edition(), Err(Cause::FirstRun), None)),
    );
    assert!(matches!(
        first.runs.as_slice(),
        [(Run::Full(Cause::FirstRun), Some(_))]
    ));
    assert_eq!(first.gathered, [0, 1]);
    first
}
pub fn sealed_run_for_lineage_test() -> SealedComputationRun {
    let world = installed_authorization_world(true);
    first_run(&world, &installed(StatusRead::Gather(1), sum))
        .sealed
        .unwrap()
        .unwrap()
}

#[path = "projected_attempt/mod.rs"]
pub mod projected_attempt;
