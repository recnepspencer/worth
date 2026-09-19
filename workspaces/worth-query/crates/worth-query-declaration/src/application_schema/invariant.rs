use std::marker::PhantomData;
use std::num::NonZeroU64;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationInvariantExecutionPoint {
    CommitBoundary,
    MutationSensitive,
    SnapshotPublication,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationInvariantEnforcement {
    BlockCommit,
    BlockPublication,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationInvariantGroup {
    StorageCoherence,
    VersionVisibility,
    AdjacencyIntegrity,
    IdentityCoherence,
    SchemaCompliance,
    LineageIntegrity,
    PublicationCoherence,
    RelationIntegrity,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationInvariantCostPosture {
    Touched,
    Partition,
    Global,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationInvariantScopeTarget {
    Entity(String),
    Relation(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationInvariantOperationalContract {
    enforcement: ApplicationInvariantEnforcement,
    required_groups: Vec<ApplicationInvariantGroup>,
    read_closure: Vec<ApplicationInvariantScopeTarget>,
    applicability: Vec<ApplicationInvariantScopeTarget>,
    provider: String,
    cost_posture: ApplicationInvariantCostPosture,
}

impl ApplicationInvariantOperationalContract {
    pub fn new(
        enforcement: ApplicationInvariantEnforcement,
        required_groups: impl IntoIterator<Item = ApplicationInvariantGroup>,
        read_closure: impl IntoIterator<Item = ApplicationInvariantScopeTarget>,
        applicability: impl IntoIterator<Item = ApplicationInvariantScopeTarget>,
        provider: impl Into<String>,
        cost_posture: ApplicationInvariantCostPosture,
    ) -> Self {
        let mut required_groups = required_groups.into_iter().collect::<Vec<_>>();
        required_groups.sort();
        required_groups.dedup();
        let mut read_closure = read_closure.into_iter().collect::<Vec<_>>();
        read_closure.sort();
        read_closure.dedup();
        let mut applicability = applicability.into_iter().collect::<Vec<_>>();
        applicability.sort();
        applicability.dedup();
        Self {
            enforcement,
            required_groups,
            read_closure,
            applicability,
            provider: provider.into(),
            cost_posture,
        }
    }
    pub const fn enforcement(&self) -> ApplicationInvariantEnforcement {
        self.enforcement
    }
    pub fn required_groups(&self) -> &[ApplicationInvariantGroup] {
        &self.required_groups
    }
    pub fn read_closure(&self) -> &[ApplicationInvariantScopeTarget] {
        &self.read_closure
    }
    pub fn applicability(&self) -> &[ApplicationInvariantScopeTarget] {
        &self.applicability
    }
    pub fn provider(&self) -> &str {
        &self.provider
    }
    pub const fn cost_posture(&self) -> ApplicationInvariantCostPosture {
        self.cost_posture
    }
}

pub trait ApplicationInvariantMarkerIdentity<Schema>: Sized + 'static {
    const IDENTIFIER: &'static str;
    const MAJOR: u16;
    const MINOR: u16;

    fn reference() -> ApplicationInvariantRef<Schema, Self> {
        ApplicationInvariantRef(PhantomData)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ApplicationInvariantRef<Schema, Invariant>(PhantomData<fn() -> (Schema, Invariant)>);

impl<Schema, Invariant> Copy for ApplicationInvariantRef<Schema, Invariant> {}
impl<Schema, Invariant> Clone for ApplicationInvariantRef<Schema, Invariant> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Schema, Invariant> ApplicationInvariantRef<Schema, Invariant>
where
    Invariant: ApplicationInvariantMarkerIdentity<Schema>,
{
    pub const fn identifier(self) -> &'static str {
        Invariant::IDENTIFIER
    }
    pub const fn major(self) -> u16 {
        Invariant::MAJOR
    }
    pub const fn minor(self) -> u16 {
        Invariant::MINOR
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RelationalInvariantWorkBudget(NonZeroU64);

impl RelationalInvariantWorkBudget {
    pub const fn new(maximum_work_units: NonZeroU64) -> Self {
        Self(maximum_work_units)
    }
    pub const fn maximum_work_units(self) -> NonZeroU64 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationInvariantDefinition<Schema, Invariant> {
    reference: ApplicationInvariantRef<Schema, Invariant>,
    execution_point: ApplicationInvariantExecutionPoint,
    work_budget: RelationalInvariantWorkBudget,
    operational: ApplicationInvariantOperationalContract,
}

impl<Schema, Invariant> ApplicationInvariantDefinition<Schema, Invariant>
where
    Invariant: ApplicationInvariantMarkerIdentity<Schema>,
{
    pub const fn new(
        reference: ApplicationInvariantRef<Schema, Invariant>,
        execution_point: ApplicationInvariantExecutionPoint,
        work_budget: RelationalInvariantWorkBudget,
        operational: ApplicationInvariantOperationalContract,
    ) -> Self {
        Self {
            reference,
            execution_point,
            work_budget,
            operational,
        }
    }

    pub const fn reference(&self) -> ApplicationInvariantRef<Schema, Invariant> {
        self.reference
    }
    pub const fn execution_point(&self) -> ApplicationInvariantExecutionPoint {
        self.execution_point
    }
    pub const fn work_budget(&self) -> RelationalInvariantWorkBudget {
        self.work_budget
    }
    pub fn operational(&self) -> &ApplicationInvariantOperationalContract {
        &self.operational
    }
}
