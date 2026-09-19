use std::marker::PhantomData;

use crate::application_schema::{
    ApplicationInvariantExecutionPoint, ApplicationInvariantMarkerIdentity, ApplicationSchema,
};

use super::ApplicationFeature;

pub struct ApplicationRuleAt<Rule, ExecutionPoint> {
    marker: PhantomData<fn() -> (Rule, ExecutionPoint)>,
}

pub struct ApplicationRuleList<Rule, Tail> {
    marker: PhantomData<fn() -> (Rule, Tail)>,
}

pub struct ApplicationRuleLeaf;
pub struct ApplicationCommitBoundary;
pub struct ApplicationMutationSensitive;
pub struct ApplicationSnapshotPublication;

pub trait ApplicationRuleExecutionPoint: Sized + 'static {
    const VALUE: ApplicationInvariantExecutionPoint;
}

impl ApplicationRuleExecutionPoint for ApplicationCommitBoundary {
    const VALUE: ApplicationInvariantExecutionPoint =
        ApplicationInvariantExecutionPoint::CommitBoundary;
}

impl ApplicationRuleExecutionPoint for ApplicationMutationSensitive {
    const VALUE: ApplicationInvariantExecutionPoint =
        ApplicationInvariantExecutionPoint::MutationSensitive;
}

impl ApplicationRuleExecutionPoint for ApplicationSnapshotPublication {
    const VALUE: ApplicationInvariantExecutionPoint =
        ApplicationInvariantExecutionPoint::SnapshotPublication;
}

pub trait ApplicationRuleRefShape<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
{
    fn declaration(
        execution_point: ApplicationInvariantExecutionPoint,
    ) -> ApplicationProgramRuleDeclaration;
}

pub trait ApplicationRuleShape<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
{
    fn declaration() -> ApplicationProgramRuleDeclaration;
}

impl<Schema, Rule, ExecutionPoint> ApplicationRuleShape<Schema>
    for ApplicationRuleAt<Rule, ExecutionPoint>
where
    Schema: ApplicationSchema,
    Rule: ApplicationRuleRefShape<Schema>,
    ExecutionPoint: ApplicationRuleExecutionPoint,
{
    fn declaration() -> ApplicationProgramRuleDeclaration {
        Rule::declaration(ExecutionPoint::VALUE)
    }
}

pub trait ApplicationProgramRulesShape<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
{
    fn rules() -> Vec<ApplicationProgramRuleDeclaration>;
}

pub trait ApplicationRuleTailShape<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
{
    fn append_rules(rules: &mut Vec<ApplicationProgramRuleDeclaration>);
}

impl<Schema> ApplicationProgramRulesShape<Schema> for ApplicationRuleLeaf
where
    Schema: ApplicationSchema,
{
    fn rules() -> Vec<ApplicationProgramRuleDeclaration> {
        Vec::new()
    }
}

impl<Schema, Rule, Tail> ApplicationProgramRulesShape<Schema> for ApplicationRuleList<Rule, Tail>
where
    Schema: ApplicationSchema,
    Rule: ApplicationRuleShape<Schema>,
    Tail: ApplicationRuleTailShape<Schema>,
{
    fn rules() -> Vec<ApplicationProgramRuleDeclaration> {
        let mut rules = vec![Rule::declaration()];
        Tail::append_rules(&mut rules);
        rules
    }
}

impl<Schema> ApplicationRuleTailShape<Schema> for ApplicationRuleLeaf
where
    Schema: ApplicationSchema,
{
    fn append_rules(_: &mut Vec<ApplicationProgramRuleDeclaration>) {}
}

impl<Schema, Rule, Tail> ApplicationRuleTailShape<Schema> for ApplicationRuleList<Rule, Tail>
where
    Schema: ApplicationSchema,
    Rule: ApplicationRuleShape<Schema>,
    Tail: ApplicationRuleTailShape<Schema>,
{
    fn append_rules(rules: &mut Vec<ApplicationProgramRuleDeclaration>) {
        rules.push(Rule::declaration());
        Tail::append_rules(rules);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationProgramRuleDeclaration {
    composition_instance: &'static str,
    identity: &'static str,
    major: u16,
    minor: u16,
    execution_point: ApplicationInvariantExecutionPoint,
    local_owner: Option<&'static str>,
}

impl ApplicationProgramRuleDeclaration {
    pub const fn composition_instance(&self) -> &'static str {
        self.composition_instance
    }
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
            composition_instance:
                <super::ApplicationRootComposition as super::ApplicationCompositionInstance>::PATH,
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

impl<Schema, Feature, Invariant> ApplicationRuleRefShape<Schema>
    for ApplicationLocalRuleRef<Schema, Feature, Invariant>
where
    Schema: ApplicationSchema + 'static,
    Feature: ApplicationFeature<Schema>,
    Invariant: ApplicationInvariantMarkerIdentity<Schema>,
{
    fn declaration(
        execution_point: ApplicationInvariantExecutionPoint,
    ) -> ApplicationProgramRuleDeclaration {
        Self::new().declaration(execution_point)
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
            composition_instance:
                <super::ApplicationRootComposition as super::ApplicationCompositionInstance>::PATH,
            identity: Invariant::IDENTIFIER,
            major: Invariant::MAJOR,
            minor: Invariant::MINOR,
            execution_point,
            local_owner: None,
        }
    }
}

/// A feature-local rule installed at one explicit composition instance.
pub struct ApplicationLocalRuleInstanceRef<Schema, Instance, Feature, Invariant> {
    marker: PhantomData<fn() -> (Schema, Instance, Feature, Invariant)>,
}

impl<Schema, Instance, Feature, Invariant> ApplicationRuleRefShape<Schema>
    for ApplicationLocalRuleInstanceRef<Schema, Instance, Feature, Invariant>
where
    Schema: ApplicationSchema + 'static,
    Instance: super::ApplicationCompositionInstance,
    Feature: ApplicationFeature<Schema>,
    Invariant: ApplicationInvariantMarkerIdentity<Schema>,
{
    fn declaration(
        execution_point: ApplicationInvariantExecutionPoint,
    ) -> ApplicationProgramRuleDeclaration {
        ApplicationProgramRuleDeclaration {
            composition_instance: Instance::PATH,
            identity: Invariant::IDENTIFIER,
            major: Invariant::MAJOR,
            minor: Invariant::MINOR,
            execution_point,
            local_owner: Some(Feature::IDENTITY),
        }
    }
}

/// A composition-owned rule installed at one explicit composition instance.
pub struct ApplicationSharedRuleInstanceRef<Schema, Instance, Invariant> {
    marker: PhantomData<fn() -> (Schema, Instance, Invariant)>,
}

impl<Schema, Instance, Invariant> ApplicationRuleRefShape<Schema>
    for ApplicationSharedRuleInstanceRef<Schema, Instance, Invariant>
where
    Schema: ApplicationSchema + 'static,
    Instance: super::ApplicationCompositionInstance,
    Invariant: ApplicationInvariantMarkerIdentity<Schema>,
{
    fn declaration(
        execution_point: ApplicationInvariantExecutionPoint,
    ) -> ApplicationProgramRuleDeclaration {
        ApplicationProgramRuleDeclaration {
            composition_instance: Instance::PATH,
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

impl<Schema, Invariant> ApplicationRuleRefShape<Schema>
    for ApplicationSharedRuleRef<Schema, Invariant>
where
    Schema: ApplicationSchema + 'static,
    Invariant: ApplicationInvariantMarkerIdentity<Schema>,
{
    fn declaration(
        execution_point: ApplicationInvariantExecutionPoint,
    ) -> ApplicationProgramRuleDeclaration {
        Self::new().declaration(execution_point)
    }
}
