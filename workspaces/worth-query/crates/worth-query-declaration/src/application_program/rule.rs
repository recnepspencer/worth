use std::marker::PhantomData;

use crate::application_schema::{
    ApplicationInvariantExecutionPoint, ApplicationInvariantMarkerIdentity, ApplicationSchema,
};

use super::ApplicationFeature;

pub trait ApplicationProgramExecutionPoint: Sized + 'static {
    const POINT: ApplicationInvariantExecutionPoint;
}

pub struct AtCommitBoundary;
pub struct AtMutationSensitive;
pub struct AtSnapshotPublication;

impl ApplicationProgramExecutionPoint for AtCommitBoundary {
    const POINT: ApplicationInvariantExecutionPoint =
        ApplicationInvariantExecutionPoint::CommitBoundary;
}
impl ApplicationProgramExecutionPoint for AtMutationSensitive {
    const POINT: ApplicationInvariantExecutionPoint =
        ApplicationInvariantExecutionPoint::MutationSensitive;
}
impl ApplicationProgramExecutionPoint for AtSnapshotPublication {
    const POINT: ApplicationInvariantExecutionPoint =
        ApplicationInvariantExecutionPoint::SnapshotPublication;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationProgramRuleDeclaration {
    identity: &'static str,
    major: u16,
    minor: u16,
    execution_point: ApplicationInvariantExecutionPoint,
    local_owner: Option<&'static str>,
    posture: ApplicationProgramRulePosture,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationProgramRulePosture {
    Available,
    Unavailable,
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
    pub const fn posture(&self) -> ApplicationProgramRulePosture {
        self.posture
    }

    const fn unavailable(mut self) -> Self {
        self.posture = ApplicationProgramRulePosture::Unavailable;
        self
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
            posture: ApplicationProgramRulePosture::Available,
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
            posture: ApplicationProgramRulePosture::Available,
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

pub struct ApplicationProgramLocalRule<Feature, Invariant, Point>(
    PhantomData<fn() -> (Feature, Invariant, Point)>,
);
pub struct ApplicationProgramSharedRule<Invariant, Point>(PhantomData<fn() -> (Invariant, Point)>);
pub struct ApplicationProgramUnavailableLocalRule<Feature, Invariant, Point>(
    PhantomData<fn() -> (Feature, Invariant, Point)>,
);
pub struct ApplicationProgramUnavailableSharedRule<Invariant, Point>(
    PhantomData<fn() -> (Invariant, Point)>,
);

pub trait ApplicationProgramRuleNode<Schema>
where
    Schema: ApplicationSchema,
{
    fn declaration() -> ApplicationProgramRuleDeclaration;
}

impl<Schema, Feature, Invariant, Point> ApplicationProgramRuleNode<Schema>
    for ApplicationProgramLocalRule<Feature, Invariant, Point>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Invariant: ApplicationInvariantMarkerIdentity<Schema>,
    Point: ApplicationProgramExecutionPoint,
{
    fn declaration() -> ApplicationProgramRuleDeclaration {
        ApplicationLocalRuleRef::<Schema, Feature, Invariant>::new().declaration(Point::POINT)
    }
}

impl<Schema, Invariant, Point> ApplicationProgramRuleNode<Schema>
    for ApplicationProgramSharedRule<Invariant, Point>
where
    Schema: ApplicationSchema,
    Invariant: ApplicationInvariantMarkerIdentity<Schema>,
    Point: ApplicationProgramExecutionPoint,
{
    fn declaration() -> ApplicationProgramRuleDeclaration {
        ApplicationSharedRuleRef::<Schema, Invariant>::new().declaration(Point::POINT)
    }
}

impl<Schema, Feature, Invariant, Point> ApplicationProgramRuleNode<Schema>
    for ApplicationProgramUnavailableLocalRule<Feature, Invariant, Point>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Invariant: ApplicationInvariantMarkerIdentity<Schema>,
    Point: ApplicationProgramExecutionPoint,
{
    fn declaration() -> ApplicationProgramRuleDeclaration {
        ApplicationLocalRuleRef::<Schema, Feature, Invariant>::new()
            .declaration(Point::POINT)
            .unavailable()
    }
}

impl<Schema, Invariant, Point> ApplicationProgramRuleNode<Schema>
    for ApplicationProgramUnavailableSharedRule<Invariant, Point>
where
    Schema: ApplicationSchema,
    Invariant: ApplicationInvariantMarkerIdentity<Schema>,
    Point: ApplicationProgramExecutionPoint,
{
    fn declaration() -> ApplicationProgramRuleDeclaration {
        ApplicationSharedRuleRef::<Schema, Invariant>::new()
            .declaration(Point::POINT)
            .unavailable()
    }
}

pub trait ApplicationProgramRuleSet<Schema>
where
    Schema: ApplicationSchema,
{
    fn declarations() -> Vec<ApplicationProgramRuleDeclaration>;
}

impl<Schema> ApplicationProgramRuleSet<Schema> for ()
where
    Schema: ApplicationSchema,
{
    fn declarations() -> Vec<ApplicationProgramRuleDeclaration> {
        Vec::new()
    }
}

macro_rules! impl_rule_sets {
    ($($rule:ident),+) => {
        impl<Schema, $($rule),+> ApplicationProgramRuleSet<Schema> for ($($rule,)+)
        where
            Schema: ApplicationSchema,
            $($rule: ApplicationProgramRuleNode<Schema>,)+
        {
            fn declarations() -> Vec<ApplicationProgramRuleDeclaration> {
                vec![$($rule::declaration()),+]
            }
        }
    };
}

impl_rule_sets!(A);
impl_rule_sets!(A, B);
impl_rule_sets!(A, B, C);
impl_rule_sets!(A, B, C, D);
impl_rule_sets!(A, B, C, D, E);
impl_rule_sets!(A, B, C, D, E, F);
impl_rule_sets!(A, B, C, D, E, F, G);
impl_rule_sets!(A, B, C, D, E, F, G, H);
impl_rule_sets!(A, B, C, D, E, F, G, H, I);
impl_rule_sets!(A, B, C, D, E, F, G, H, I, J);
impl_rule_sets!(A, B, C, D, E, F, G, H, I, J, K);
impl_rule_sets!(A, B, C, D, E, F, G, H, I, J, K, L);
