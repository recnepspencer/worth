use std::any::TypeId;

use crate::application_schema::ApplicationSchema;

use super::{ApplicationFeature, ApplicationLocalityScope, ApplicationOutputPort};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationArtifactRetention {
    Retained,
    Disposable,
    Reconstructive,
}

impl ApplicationArtifactRetention {
    /// Names this retention posture in the durable canonical manifest record
    /// that the program revision digests.
    ///
    /// The token is decided here rather than derived from the Rust variant
    /// spelling, so renaming a variant is a compile-time-visible decision
    /// instead of a silent change of every program's content identity.
    pub const fn canonical_token(self) -> &'static str {
        match self {
            Self::Retained => "Retained",
            Self::Disposable => "Disposable",
            Self::Reconstructive => "Reconstructive",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationArtifactSuccession {
    PreserveWhenEquivalent,
    Replace,
    Recompute,
}

impl ApplicationArtifactSuccession {
    /// Names this succession posture in the durable canonical manifest record
    /// that the program revision digests.
    ///
    /// The token is decided here rather than derived from the Rust variant
    /// spelling, so renaming a variant is a compile-time-visible decision
    /// instead of a silent change of every program's content identity.
    pub const fn canonical_token(self) -> &'static str {
        match self {
            Self::PreserveWhenEquivalent => "PreserveWhenEquivalent",
            Self::Replace => "Replace",
            Self::Recompute => "Recompute",
        }
    }
}

/// Whether an application program has completed the cutover to governed derived outputs.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationDerivedArtifactGovernance {
    Compatible,
    Required,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationArtifactDependency {
    identity: &'static str,
}

impl ApplicationArtifactDependency {
    pub const fn new(identity: &'static str) -> Self {
        Self { identity }
    }

    pub const fn identity(self) -> &'static str {
        self.identity
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationArtifactResourceCeiling {
    maximum_work: usize,
    maximum_retained_bytes: usize,
}

impl ApplicationArtifactResourceCeiling {
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

/// One derived product owned by a feature in the canonical application program.
///
/// The declaration names dependency locality and lifecycle posture. Numerical
/// calculation and publication remain with their installed owners.
pub trait ApplicationDerivedArtifact<Schema, Feature>: Sized + 'static
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
{
    type Output: ApplicationOutputPort<Schema, Feature>;
    type Locality: ApplicationLocalityScope;

    const IDENTITY: &'static str;
    const RETENTION: ApplicationArtifactRetention;
    const SUCCESSION: ApplicationArtifactSuccession;
    const REQUIRED: bool;
    const PRODUCER_FAMILY: &'static str;
    const DEPENDENCIES: &'static [ApplicationArtifactDependency];
    const REUSE_RULE: &'static str;
    const RESOURCE_CEILING: ApplicationArtifactResourceCeiling;
    const STOPPED_OUTCOME: &'static str;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationDerivedArtifactDeclaration {
    identity: &'static str,
    artifact_type: TypeId,
    feature: &'static str,
    feature_type: TypeId,
    output: &'static str,
    output_type: TypeId,
    locality: super::ApplicationLocalityDeclaration,
    retention: ApplicationArtifactRetention,
    succession: ApplicationArtifactSuccession,
    required: bool,
    producer_family: &'static str,
    dependencies: &'static [ApplicationArtifactDependency],
    reuse_rule: &'static str,
    resource_ceiling: ApplicationArtifactResourceCeiling,
    stopped_outcome: &'static str,
}

impl ApplicationDerivedArtifactDeclaration {
    pub(crate) fn of<Schema, Feature, Artifact>() -> Self
    where
        Schema: ApplicationSchema,
        Feature: ApplicationFeature<Schema>,
        Artifact: ApplicationDerivedArtifact<Schema, Feature>,
    {
        Self {
            identity: Artifact::IDENTITY,
            artifact_type: TypeId::of::<Artifact>(),
            feature: Feature::IDENTITY,
            feature_type: TypeId::of::<Feature>(),
            output: Artifact::Output::IDENTITY,
            output_type: TypeId::of::<Artifact::Output>(),
            locality: super::ApplicationLocalityDeclaration::of::<Artifact::Locality>(),
            retention: Artifact::RETENTION,
            succession: Artifact::SUCCESSION,
            required: Artifact::REQUIRED,
            producer_family: Artifact::PRODUCER_FAMILY,
            dependencies: Artifact::DEPENDENCIES,
            reuse_rule: Artifact::REUSE_RULE,
            resource_ceiling: Artifact::RESOURCE_CEILING,
            stopped_outcome: Artifact::STOPPED_OUTCOME,
        }
    }

    pub const fn identity(&self) -> &'static str {
        self.identity
    }
    pub const fn artifact_type(&self) -> TypeId {
        self.artifact_type
    }
    pub const fn feature_type(&self) -> TypeId {
        self.feature_type
    }
    pub const fn feature(&self) -> &'static str {
        self.feature
    }
    pub const fn output(&self) -> &'static str {
        self.output
    }
    pub const fn output_type(&self) -> TypeId {
        self.output_type
    }
    pub const fn locality(&self) -> &super::ApplicationLocalityDeclaration {
        &self.locality
    }
    pub const fn retention(&self) -> ApplicationArtifactRetention {
        self.retention
    }
    pub const fn succession(&self) -> ApplicationArtifactSuccession {
        self.succession
    }
    pub const fn required(&self) -> bool {
        self.required
    }
    pub const fn producer_family(&self) -> &'static str {
        self.producer_family
    }
    pub const fn dependencies(&self) -> &'static [ApplicationArtifactDependency] {
        self.dependencies
    }
    pub const fn reuse_rule(&self) -> &'static str {
        self.reuse_rule
    }
    pub const fn resource_ceiling(&self) -> ApplicationArtifactResourceCeiling {
        self.resource_ceiling
    }
    pub const fn stopped_outcome(&self) -> &'static str {
        self.stopped_outcome
    }
}
