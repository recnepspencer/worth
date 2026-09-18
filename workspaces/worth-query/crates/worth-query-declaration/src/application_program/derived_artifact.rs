use std::any::TypeId;

use crate::application_schema::ApplicationSchema;

use super::{ApplicationFeature, ApplicationLocalityScope, ApplicationOutputPort};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationArtifactRetention {
    Retained,
    Disposable,
    Reconstructive,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationArtifactSuccession {
    PreserveWhenEquivalent,
    Replace,
    Recompute,
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationDerivedArtifactDeclaration {
    identity: &'static str,
    artifact_type: TypeId,
    output: &'static str,
    locality: super::ApplicationLocalityDeclaration,
    retention: ApplicationArtifactRetention,
    succession: ApplicationArtifactSuccession,
    required: bool,
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
            output: Artifact::Output::IDENTITY,
            locality: super::ApplicationLocalityDeclaration::of::<Artifact::Locality>(),
            retention: Artifact::RETENTION,
            succession: Artifact::SUCCESSION,
            required: Artifact::REQUIRED,
        }
    }

    pub const fn identity(&self) -> &'static str {
        self.identity
    }
    pub const fn artifact_type(&self) -> TypeId {
        self.artifact_type
    }
    pub const fn output(&self) -> &'static str {
        self.output
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
}
