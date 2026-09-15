use std::marker::PhantomData;

use crate::application_schema::{
    ApplicationInvariantExecutionPoint, ApplicationInvariantMarkerIdentity, ApplicationSchema,
};

use super::ApplicationFeature;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationProgramRuleDeclaration {
    identity: &'static str,
    major: u16,
    minor: u16,
    execution_point: ApplicationInvariantExecutionPoint,
    local_owner: Option<&'static str>,
}

impl ApplicationProgramRuleDeclaration {
    pub const fn identity(&self) -> &'static str {
        self.identity
    }
    pub const fn major(&self) -> u16 {
        self.major
    }
    pub const fn minor(&self) -> u16 {
        self.minor
    }
    pub const fn execution_point(&self) -> ApplicationInvariantExecutionPoint {
        self.execution_point
    }
    pub const fn local_owner(&self) -> Option<&'static str> {
        self.local_owner
    }
}

/// A rule scoped to concrete occurrences owned by one program feature.
pub struct ApplicationLocalRuleRef<Schema, Feature, Invariant> {
    marker: PhantomData<fn() -> (Schema, Feature, Invariant)>,
}

impl<Schema, Feature, Invariant> ApplicationLocalRuleRef<Schema, Feature, Invariant>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Invariant: ApplicationInvariantMarkerIdentity<Schema>,
{
    pub const fn new() -> Self {
        Self {
            marker: PhantomData,
        }
    }

    pub const fn declaration(
        self,
        execution_point: ApplicationInvariantExecutionPoint,
    ) -> ApplicationProgramRuleDeclaration {
        ApplicationProgramRuleDeclaration {
            identity: Invariant::IDENTIFIER,
            major: Invariant::MAJOR,
            minor: Invariant::MINOR,
            execution_point,
            local_owner: Some(Feature::IDENTITY),
        }
    }
}

impl<Schema, Feature, Invariant> Default for ApplicationLocalRuleRef<Schema, Feature, Invariant>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Invariant: ApplicationInvariantMarkerIdentity<Schema>,
{
    fn default() -> Self {
        Self::new()
    }
}

/// A composition-owned rule spanning exported feature meaning.
pub struct ApplicationSharedRuleRef<Schema, Invariant> {
    marker: PhantomData<fn() -> (Schema, Invariant)>,
}

impl<Schema, Invariant> ApplicationSharedRuleRef<Schema, Invariant>
where
    Schema: ApplicationSchema,
    Invariant: ApplicationInvariantMarkerIdentity<Schema>,
{
    pub const fn new() -> Self {
        Self {
            marker: PhantomData,
        }
    }

    pub const fn declaration(
        self,
        execution_point: ApplicationInvariantExecutionPoint,
    ) -> ApplicationProgramRuleDeclaration {
        ApplicationProgramRuleDeclaration {
            identity: Invariant::IDENTIFIER,
            major: Invariant::MAJOR,
            minor: Invariant::MINOR,
            execution_point,
            local_owner: None,
        }
    }
}

impl<Schema, Invariant> Default for ApplicationSharedRuleRef<Schema, Invariant>
where
    Schema: ApplicationSchema,
    Invariant: ApplicationInvariantMarkerIdentity<Schema>,
{
    fn default() -> Self {
        Self::new()
    }
}
