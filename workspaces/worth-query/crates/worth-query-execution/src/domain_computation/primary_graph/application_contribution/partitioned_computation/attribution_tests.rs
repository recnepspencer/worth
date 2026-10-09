//! Which owner calls the sealed read set keeps for each fact, for an owner
//! reading through the reader of a real admitted operation.

use std::sync::Mutex;

use serde::Serialize;
use worth_execution::PartitionItemId;
use worth_foundational::facade::PartitionIdentity;
use worth_query_declaration::facade::application_program::{
    ApplicationArtifactDependency, ApplicationArtifactResourceCeiling,
    ApplicationArtifactRetention, ApplicationArtifactSuccession, ApplicationComputationExecution,
    ApplicationComputationInput, ApplicationComputationPartition,
    ApplicationComputationResourceCeiling, ApplicationComputationReuse,
    ApplicationComputationStopped, ApplicationDerivedArtifact, ApplicationFeature,
    ApplicationFeatureInputLeaf, ApplicationLocalityGranule, ApplicationLocalityScope,
    ApplicationManagedComputation, ApplicationOutputPort,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationStructuredValueBinding, ApplicationValueValidationDenial,
};

use super::{
    WorthQueryComputationInputDenial, WorthQueryComputationPartitionMembers,
    WorthQueryComputationPartitionPlan, WorthQueryComputationPartitionView,
    WorthQueryComputationReader, WorthQueryDeterministicReducer,
    WorthQueryInstalledPartitionedComputation, WorthQueryPartitionedComputationOwner,
};
use crate::domain_computation::primary_graph::application_attempt::ComputationFactReaders;
use crate::domain_computation::primary_graph::application_contribution::QueryRequestExecution;
use crate::domain_computation::primary_graph::tests::application_attempt::{
    authenticated_principal, resolved_account,
};
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, live_scope, Account, AccountLabel, AccountStatus,
    IdentityExecutionSchema as Schema, TouchAccountOperation,
};
use crate::domain_computation::primary_graph::{
    WorthQueryInvariantEntityIdentity, WorthQueryManagedComputationCheckpoint,
    WorthQueryManagedComputationDenial,
};

pub(super) struct Feature;
impl ApplicationFeature<Schema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.tests.attribution-feature.v1";
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Value(u64);
worth_query_declaration::worth_query_portable_type!(
    Value => "worth.query.tests.attribution-value.v1"
);
pub(super) struct ValueBinding;
impl ApplicationStructuredValueBinding for ValueBinding {
    type Value = Value;
    const IDENTITY_NAME: &'static str = "worth.query.tests.attribution-value.v1";
    fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}
pub(super) struct Output;
impl ApplicationOutputPort<Schema, Feature> for Output {
    type Value = ValueBinding;
    const IDENTITY: &'static str = "worth.query.tests.attribution-output.v1";
}
pub(super) struct Locality;
impl ApplicationLocalityScope for Locality {
    const IDENTITY: &'static str = "worth.query.tests.attribution-locality.v1";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Partition;
}
pub(super) struct Artifact;
impl ApplicationDerivedArtifact<Schema, Feature> for Artifact {
    type Output = Output;
    type Locality = Locality;
    const IDENTITY: &'static str = "worth.query.tests.attribution-artifact.v1";
    const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Disposable;
    const SUCCESSION: ApplicationArtifactSuccession = ApplicationArtifactSuccession::Recompute;
    const REQUIRED: bool = true;
    const PRODUCER_FAMILY: &'static str = "worth.query.tests.attribution-producer.v1";
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] = &[];
    const REUSE_RULE: &'static str = "worth.query.tests.attribution-reuse.v1";
    const RESOURCE_CEILING: ApplicationArtifactResourceCeiling =
        ApplicationArtifactResourceCeiling::new(4, 64);
    const STOPPED_OUTCOME: &'static str = "worth.query.tests.attribution-stopped.v1";
}
pub(super) struct Input;
impl ApplicationComputationInput for Input {
    type Value = WorthQueryInvariantEntityIdentity<Schema, Account>;
    const IDENTITY: &'static str = "worth.query.tests.attribution-input.v1";
}
pub(super) struct Reuse;
impl ApplicationComputationReuse for Reuse {
    const IDENTITY: &'static str = "worth.query.tests.attribution-evidence.v1";
}
pub(super) struct Stopped;
impl ApplicationComputationStopped for Stopped {
    const IDENTITY: &'static str = "worth.query.tests.attribution-owner-stopped.v1";
}
/// One test item: a number, which its digest encodes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub(super) struct Number(pub(super) u64);
impl ApplicationComputationPartition for Number {
    const IDENTITY: &'static str = "worth.query.tests.attribution-number.v1";
}
impl worth_execution::ChargedBytes for Number {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
/// Odd and even items are the two partitions.
#[derive(Serialize)]
pub(super) struct Parity(pub(super) u64);
impl ApplicationComputationPartition for Parity {
    const IDENTITY: &'static str = "worth.query.tests.attribution-parity.v1";
}
pub(super) struct Computation;
impl ApplicationManagedComputation<Schema, Feature> for Computation {
    type Input = Input;
    type Output = Artifact;
    type Partition = Parity;
    type Reuse = Reuse;
    type Stopped = Stopped;
    const IDENTITY: &'static str = "worth.query.tests.attribution-computation.v1";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::DeterministicPartitioned;
    const RESOURCES: ApplicationComputationResourceCeiling =
        ApplicationComputationResourceCeiling::new(4096, 4096);
}

/// Which owner call reads the account's status. Every `gather` reads its
/// label.
#[derive(Clone, Copy)]
enum StatusReadBy {
    Membership,
    ItemKeys,
}

/// Items 1, 2 and 3 of one account. The partitions `gather` was called for
/// are kept for the test to name them.
struct Owner {
    status: StatusReadBy,
    gathered: Mutex<Vec<PartitionIdentity>>,
}

type Reader<'call, 'reader, 'runtime> =
    WorthQueryComputationReader<'call, 'reader, 'runtime, Schema, TouchAccountOperation>;
type Root = WorthQueryInvariantEntityIdentity<Schema, Account>;

impl WorthQueryPartitionedComputationOwner<Schema, Feature, Computation> for Owner {
    type Operation = TouchAccountOperation;
    type Item = Number;
    type Gathered = u64;
    type PartitionResult = u64;
    type Output = u64;
    type Stopped = u32;

    fn partitions(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        account: &Root,
    ) -> Result<WorthQueryComputationPartitionPlan<Number>, WorthQueryComputationInputDenial<u32>>
    {
        if matches!(self.status, StatusReadBy::Membership) {
            reader.field(account, AccountStatus::reference())?;
        }
        Ok(WorthQueryComputationPartitionPlan::keyed(
            [3, 1, 2].map(Number),
            |item| PartitionItemId(item.0),
        ))
    }

    fn partition_key(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        account: &Root,
        item: &Number,
    ) -> Result<Parity, WorthQueryComputationInputDenial<u32>> {
        if matches!(self.status, StatusReadBy::ItemKeys) {
            reader.field(account, AccountStatus::reference())?;
        }
        Ok(Parity(item.0 % 2))
    }

    fn gather(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        account: &Root,
        partition: WorthQueryComputationPartitionMembers<'_, Parity, Number>,
    ) -> Result<u64, WorthQueryComputationInputDenial<u32>> {
        reader.field(account, AccountLabel::reference())?;
        self.gathered.lock().unwrap().push(partition.identity());
        Ok(partition.items().map(|(_, item)| item.0).sum())
    }

    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, Parity, u64>,
        _: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<u64, WorthQueryManagedComputationDenial<u32>> {
        Ok(*partition.gathered())
    }

    fn reducer(&self) -> WorthQueryDeterministicReducer<u64> {
        WorthQueryDeterministicReducer::canonical(|| 0, |left, right| left + right)
    }

    fn complete(&self, reduced: u64) -> Result<u64, u32> {
        Ok(reduced)
    }
}

/// The calls the sealed read set keeps for the label and the status fact, in
/// fact order, with the partitions `gather` ran for in ascending order.
///
/// `handler_reads_status` has the handler read the status itself after the
/// computation's `runs` runs.
fn routed(
    status: StatusReadBy,
    runs: usize,
    handler_reads_status: bool,
) -> (Option<Vec<ComputationFactReaders>>, Vec<PartitionIdentity>) {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            Default::default(),
            &request,
        )
        .unwrap();
    let installed =
        WorthQueryInstalledPartitionedComputation::<Schema, Feature, Computation, Owner>::new(
            Owner {
                status,
                gathered: Mutex::default(),
            },
            super::ComputationRetention::Unretained,
        );
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, root| {
            let execution = QueryRequestExecution::open(
            worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Serial(
                crate::domain_computation::primary_graph::application_contribution::request_execution::test_policy(
                    std::num::NonZeroUsize::MIN, 1 << 30,
                ),
            ), &request);
            for _ in 0..runs {
                if installed.prepare_through(reader, &execution, root).is_err() {
                    panic!("the owner's reads are declared reads of the operation");
                }
            }
            if handler_reads_status {
                reader
                    .require_decision_field(root, AccountStatus::reference())
                    .unwrap();
            }
        }, worth_execution::ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
        .into_parts();
    let read_set = world
        .application
        .begin_projected_application_read_attempt(
            admission,
            projection,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .complete_projected_dependencies(
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let mut gathered = installed.owner.gathered.lock().unwrap().clone();
    gathered.sort_unstable();
    gathered.dedup();
    (
        read_set.computation_facts().map(|facts| {
            facts
                .facts()
                .map(|(_, _, readers)| readers.clone())
                .collect()
        }),
        gathered,
    )
}

#[test]
fn a_fact_two_gathers_read_is_kept_for_both_partitions() {
    let (readers, partitions) = routed(StatusReadBy::ItemKeys, 1, false);

    assert_eq!(partitions.len(), 2, "odd and even items are two partitions");
    // The facts are in key order: the label, then the status.
    assert_eq!(
        readers.expect("one computation ran"),
        [
            ComputationFactReaders::read_by(false, [], partitions),
            ComputationFactReaders::read_by(false, [1, 2, 3], []),
        ]
    );
}

#[test]
fn a_fact_the_membership_read_is_a_membership_fact() {
    let (readers, partitions) = routed(StatusReadBy::Membership, 1, false);

    assert_eq!(
        readers.expect("one computation ran"),
        [
            ComputationFactReaders::read_by(false, [], partitions),
            ComputationFactReaders::read_by(true, [], []),
        ]
    );
}

#[test]
fn a_fact_the_handler_also_read_stays_the_item_keys() {
    // The handler reads its own facts again on every attempt, so it is not a
    // reader the computation keeps: the status is still the item keys' fact.
    let (readers, partitions) = routed(StatusReadBy::ItemKeys, 1, true);

    assert_eq!(
        readers.expect("one computation ran"),
        [
            ComputationFactReaders::read_by(false, [], partitions),
            ComputationFactReaders::read_by(false, [1, 2, 3], []),
        ]
    );
}

#[test]
fn an_attempt_that_ran_two_computations_keeps_no_computation_facts() {
    // Partition identities name the partitions of one computation only.
    assert!(routed(StatusReadBy::ItemKeys, 2, false).0.is_none());
}
