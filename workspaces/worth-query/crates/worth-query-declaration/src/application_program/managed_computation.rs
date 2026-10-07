use std::any::TypeId;

use serde::{Serialize, Serializer};
use worth_foundational::facade::DeterminismContract;

use crate::application_schema::ApplicationSchema;

use super::{ApplicationDerivedArtifact, ApplicationFeature};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationComputationExecution {
    Deterministic,
    DeterministicPartitioned,
}

impl ApplicationComputationExecution {
    /// Names this execution posture in the durable canonical manifest record
    /// that the program revision digests.
    ///
    /// The token is decided here rather than derived from the Rust variant
    /// spelling, so renaming a variant is a compile-time-visible decision
    /// instead of a silent change of every program's content identity.
    pub const fn canonical_token(self) -> &'static str {
        match self {
            Self::Deterministic => "Deterministic",
            Self::DeterministicPartitioned => "DeterministicPartitioned",
        }
    }
}

/// Names a determinism contract in the durable canonical manifest record that
/// the program revision digests. The token is decided here for the same reason
/// as [`ApplicationComputationExecution::canonical_token`].
pub(super) fn determinism_canonical_token(determinism: DeterminismContract) -> String {
    match determinism {
        DeterminismContract::CanonicalBitwise => "CanonicalBitwise".to_owned(),
        DeterminismContract::ContractEquivalent(equivalence) => {
            format!("ContractEquivalent({})", equivalence.value())
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationComputationResourceCeiling {
    maximum_work: usize,
    maximum_retained_bytes: usize,
}

impl ApplicationComputationResourceCeiling {
    pub const fn new(maximum_work: usize, maximum_retained_bytes: usize) -> Self {
        Self {
            maximum_work,
            maximum_retained_bytes,
        }
    }

    pub const fn maximum_work(self) -> usize {
        self.maximum_work
    }

    pub const fn maximum_retained_bytes(self) -> usize {
        self.maximum_retained_bytes
    }
}

/// What a computation reads its input through.
///
/// The input value has a canonical encoding: the prefix-free canonical
/// encoding of its `Serialize` form that partition keys have. Its digest is
/// what makes two input values the same input, so a partitioned computation
/// keeps its last run's partitions only for the same input.
pub trait ApplicationComputationInput: 'static {
    type Value: Serialize + Send + Sync + 'static;
    const IDENTITY: &'static str;
}

/// A value with a canonical encoding under a declared identity: a computation
/// partition key, or an item a partitioned computation partitions.
///
/// Two values are the same value exactly when their canonical encodings are
/// the same bytes: the declaration's prefix-free canonical encoding of the
/// value's `Serialize` form, the encoding that already identifies structured
/// operation inputs. A key carries no ordering of its own. Computation
/// partitions are ordered by partition identity, which
/// `application_computation_partition_identity` derives from that encoding;
/// `application_computation_item_digest` derives an item's digest from it.
pub trait ApplicationComputationPartition: Serialize + Send + Sync + 'static {
    const IDENTITY: &'static str;
}

/// The one computation partition of every `Deterministic` computation.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ApplicationSingleComputationPartition;

impl Serialize for ApplicationSingleComputationPartition {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_unit_struct("ApplicationSingleComputationPartition")
    }
}

impl ApplicationComputationPartition for ApplicationSingleComputationPartition {
    const IDENTITY: &'static str = "worth.query.single-computation-partition.v1";
}

pub trait ApplicationComputationReuse: 'static {
    const IDENTITY: &'static str;
}

pub trait ApplicationComputationStopped: 'static {
    const IDENTITY: &'static str;
}

pub trait ApplicationManagedComputation<Schema, Feature>: Sized + 'static
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
{
    type Input: ApplicationComputationInput;
    type Output: ApplicationDerivedArtifact<Schema, Feature>;
    type Partition: ApplicationComputationPartition;
    type Reuse: ApplicationComputationReuse;
    type Stopped: ApplicationComputationStopped;

    const IDENTITY: &'static str;
    const EXECUTION: ApplicationComputationExecution;
    const DETERMINISM: DeterminismContract = DeterminismContract::CanonicalBitwise;
    const RESOURCES: ApplicationComputationResourceCeiling;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationManagedComputationDeclaration {
    identity: &'static str,
    computation_type: TypeId,
    feature_type: TypeId,
    input: &'static str,
    output_artifact: &'static str,
    output_artifact_type: TypeId,
    partition: &'static str,
    partition_type: TypeId,
    reuse: &'static str,
    stopped: &'static str,
    execution: ApplicationComputationExecution,
    determinism: DeterminismContract,
    resources: ApplicationComputationResourceCeiling,
}

impl ApplicationManagedComputationDeclaration {
    pub(crate) fn of<Schema, Feature, Computation>() -> Self
    where
        Schema: ApplicationSchema,
        Feature: ApplicationFeature<Schema>,
        Computation: ApplicationManagedComputation<Schema, Feature>,
    {
        Self {
            identity: Computation::IDENTITY,
            computation_type: TypeId::of::<Computation>(),
            feature_type: TypeId::of::<Feature>(),
            input: Computation::Input::IDENTITY,
            output_artifact: Computation::Output::IDENTITY,
            output_artifact_type: TypeId::of::<Computation::Output>(),
            partition: Computation::Partition::IDENTITY,
            partition_type: TypeId::of::<Computation::Partition>(),
            reuse: Computation::Reuse::IDENTITY,
            stopped: Computation::Stopped::IDENTITY,
            execution: Computation::EXECUTION,
            determinism: Computation::DETERMINISM,
            resources: Computation::RESOURCES,
        }
    }

    pub const fn identity(&self) -> &'static str {
        self.identity
    }
    pub const fn computation_type(&self) -> TypeId {
        self.computation_type
    }
    pub const fn feature_type(&self) -> TypeId {
        self.feature_type
    }
    pub const fn input(&self) -> &'static str {
        self.input
    }
    pub const fn output_artifact(&self) -> &'static str {
        self.output_artifact
    }
    pub const fn output_artifact_type(&self) -> TypeId {
        self.output_artifact_type
    }
    pub const fn partition(&self) -> &'static str {
        self.partition
    }
    pub const fn reuse(&self) -> &'static str {
        self.reuse
    }
    pub const fn stopped(&self) -> &'static str {
        self.stopped
    }
    pub const fn execution(&self) -> ApplicationComputationExecution {
        self.execution
    }
    pub const fn determinism(&self) -> DeterminismContract {
        self.determinism
    }
    /// A `Deterministic` computation declares exactly the platform's single
    /// computation partition, and a `DeterministicPartitioned` one never does.
    pub(super) fn partition_matches_execution(&self) -> bool {
        let single = self.partition_type == TypeId::of::<ApplicationSingleComputationPartition>();
        match self.execution {
            ApplicationComputationExecution::Deterministic => single,
            ApplicationComputationExecution::DeterministicPartitioned => !single,
        }
    }
    pub const fn resources(&self) -> ApplicationComputationResourceCeiling {
        self.resources
    }
}
