use std::any::TypeId;

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

pub trait ApplicationComputationInput: 'static {
    type Value: Send + Sync + 'static;
    const IDENTITY: &'static str;
}

pub trait ApplicationComputationPartition: 'static {
    const IDENTITY: &'static str;
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
    const ORDERING: &'static str;
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
    reuse: &'static str,
    stopped: &'static str,
    execution: ApplicationComputationExecution,
    ordering: &'static str,
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
            reuse: Computation::Reuse::IDENTITY,
            stopped: Computation::Stopped::IDENTITY,
            execution: Computation::EXECUTION,
            ordering: Computation::ORDERING,
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
    pub const fn ordering(&self) -> &'static str {
        self.ordering
    }
    pub const fn resources(&self) -> ApplicationComputationResourceCeiling {
        self.resources
    }
}
