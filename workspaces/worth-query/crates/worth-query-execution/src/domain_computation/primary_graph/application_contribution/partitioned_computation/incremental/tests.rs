//! A producer's next run of a partitioned computation, through a real
//! admitted operation. The test hands each attempt the prior a producer would
//! and keeps the run seal retains; a moved fact is the retained fact standing
//! in for an older one.

use crate::domain_computation::primary_graph::output_lineage::custodied_state_for_test;
use std::sync::{Arc, Mutex};

use worth_execution::PartitionItemId;
use worth_foundational::facade::ExecutionReport;
use worth_runtime_world::facade::RuntimeWorldExecutionPlacement;

use worth_query_declaration::facade::application_program::ApplicationManagedComputation;

use super::super::attribution_tests::{Computation, Feature, Input, Number, Parity};
use super::super::{
    ComputationRetention, WorthQueryComputationInputDenial, WorthQueryComputationPartitionMembers,
    WorthQueryComputationPartitionPlan, WorthQueryComputationPartitionView,
    WorthQueryComputationReader, WorthQueryDeterministicReducer,
    WorthQueryInstalledPartitionedComputation, WorthQueryPartitionedComputationDenial,
    WorthQueryPartitionedComputationOwner,
};
use super::observed::{
    partitioned_computation_runs_on_this_thread_for_test as runs,
    partitioned_computation_tree_work_on_this_thread_for_test as tree_runs,
};
use super::{
    ComputationPrior, SealedComputationRun, WorthQueryPartitionedComputationFullCause as Cause,
    WorthQueryPartitionedComputationRun as Run,
};
use crate::domain_computation::primary_graph::application_attempt::{
    ComputationRead, WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::application_contribution::{
    InstalledProducerEdition, QueryRequestExecution,
};
use crate::domain_computation::primary_graph::tests::application_attempt::{
    authenticated_principal, resolved_account,
};
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, live_scope, Account, AccountLabel, AccountStatus,
    AuthorizationWorld, IdentityExecutionSchema as Schema, TouchAccountOperation,
};
use crate::domain_computation::primary_graph::{
    WorthQueryInvariantEntityIdentity, WorthQueryManagedComputationCheckpoint,
    WorthQueryManagedComputationDenial, WorthQueryManagedComputationExecution,
};

type Reader<'call, 'reader, 'runtime> =
    WorthQueryComputationReader<'call, 'reader, 'runtime, Schema, TouchAccountOperation>;
type Root = WorthQueryInvariantEntityIdentity<Schema, Account>;
type Installed = WorthQueryInstalledPartitionedComputation<Schema, Feature, Computation, Owner>;
type Outcome = Result<(u64, u64), WorthQueryPartitionedComputationDenial<u32>>;

/// Which owner call reads the account's status, besides every gather reading
/// its label.
#[derive(Clone, Copy)]
enum StatusRead {
    /// The gather of the partition of this parity.
    Gather(u64),
    ItemKeys,
}

/// Items 1 to 4, keyed by their remainder modulo `modulus`: odd and even by
/// default. What a moved fact would change is set by the test: what the
/// status partition gathers on top, and each kernel's work.
struct Owner {
    modulus: u64,
    status: StatusRead,
    reducer: fn(&u64, &u64) -> u64,
    bump: Mutex<u64>,
    /// The work of each kernel, odd parity first.
    work: Mutex<[usize; 2]>,
    gathered: Mutex<Vec<u64>>,
    calls: Mutex<[usize; 4]>,
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
        let work = self.work.lock().unwrap()[usize::from(partition.key().0 == 0)];
        checkpoint.advance(work)?;
        Ok(*partition.gathered())
    }

    fn reducer(&self) -> WorthQueryDeterministicReducer<u64> {
        WorthQueryDeterministicReducer::canonical(|| 0, self.reducer)
    }

    fn complete(&self, reduced: u64) -> Result<u64, u32> {
        Ok(reduced)
    }
}

fn installed(status: StatusRead, reducer: fn(&u64, &u64) -> u64) -> Installed {
    installed_over(2, status, reducer)
}

fn installed_over(modulus: u64, status: StatusRead, reducer: fn(&u64, &u64) -> u64) -> Installed {
    let owner = Owner {
        modulus,
        status,
        reducer,
        bump: Mutex::new(0),
        work: Mutex::new([1, 1]),
        gathered: Mutex::default(),
        calls: Mutex::default(),
    };
    Installed::new(owner, ComputationRetention::ProducerOperation)
}

fn sum(left: &u64, right: &u64) -> u64 {
    left + right
}

fn edition() -> InstalledProducerEdition {
    InstalledProducerEdition::for_test([7; 32])
}

/// One attempt's run: its outcome and charged work, how it ran, the
/// parities it gathered, and the run seal kept, or the seal's refusal.
struct Attempt {
    outcome: Outcome,
    /// The work the projection's reader charged.
    work: crate::domain_computation::primary_graph::invariant_projection::WorthQueryInvariantProjectionWork,
    runs: Vec<(Run, Option<ExecutionReport>)>,
    tree_runs: Vec<super::tree_report::WorthQueryPartitionedTreeRun>,
    gathered: Vec<u64>,
    sealed: Result<Option<SealedComputationRun>, ()>,
}

/// An owner of a test computation over the account, with the same items and
/// results.
trait TestOwner<Tested: ApplicationManagedComputation<Schema, Feature> = Computation>:
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

fn attempt<Tested, Owned>(
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

/// [`attempt`] under `request`.
fn attempt_in<Tested, Owned>(
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
        crate::domain_computation::primary_graph::application_contribution::request_execution::test_policy(
            std::num::NonZeroUsize::MIN, 1 << 30,
        ),
    ),
    )
}

mod projected_attempt;
use projected_attempt::attempt_placed;

/// The prior a producer hands its next run from what `first` retained, with
/// the status fact moved when `moved`.
fn prior_of(first: Attempt, moved: bool) -> ComputationPrior {
    let mut state = first
        .sealed
        .unwrap()
        .expect("a producer's run retains")
        .state;
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
    ComputationPrior::new(edition(), Ok(custodied_state_for_test(state)), None)
}

fn first_run(world: &AuthorizationWorld, installed: &Installed) -> Attempt {
    let first = attempt(
        world,
        installed,
        Some(ComputationPrior::new(edition(), Err(Cause::FirstRun), None)),
    );
    assert!(matches!(
        first.runs.as_slice(),
        [(Run::Full(Cause::FirstRun), Some(_))]
    ));
    assert_eq!(first.gathered, [0, 1]);
    first
}

mod carrying;
mod certified;
mod collision;
mod digests;
mod installation;
mod interruption;
mod request_memory;
mod reroute;
mod tree_memory;
mod tree_work;
mod unobservable;
mod wide;

pub(super) fn sealed_run_for_lineage_test() -> SealedComputationRun {
    let world = installed_authorization_world(true);
    first_run(&world, &installed(StatusRead::Gather(1), sum))
        .sealed
        .unwrap()
        .unwrap()
}
