//! Bank-owned projection denial taxonomy and Query crossing.

use bank_domain::proposals::BankProposalDenial;
use worth_query_host::facade::primary_graph::{
    WorthQueryEntityResolutionDenial, WorthQueryInvariantAggregateDenial,
    WorthQueryInvariantAggregateDenialKind as QueryAggregate,
    WorthQueryInvariantDecisionPlanDenial, WorthQueryInvariantProjectionTraversalDenial,
};

use crate::{
    BankEntityResolutionDenial, BankInvariantDecisionPlanDenial,
    BankInvariantProjectionTraversalDenial,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankInvariantAggregateDenialKind {
    ReadExecution(
        Result<
            worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind,
            worth_query_host::facade::primary_graph::WorthQueryProviderSessionControlStopKind,
        >,
    ),
    RelationNotInstalled,
    FieldNotInstalled,
    ForeignIdentity,
    WorkBudgetExceeded,
    InvalidScalar,
    ArithmeticOverflow,
    SourceCountOverflow,
    AmbiguousSourceRelation,
}

impl BankInvariantAggregateDenialKind {
    const fn from_query(kind: QueryAggregate) -> Self {
        match kind {
            QueryAggregate::ReadExecution(cause) => Self::ReadExecution(cause),
            QueryAggregate::RelationNotInstalled => Self::RelationNotInstalled,
            QueryAggregate::FieldNotInstalled => Self::FieldNotInstalled,
            QueryAggregate::ForeignIdentity => Self::ForeignIdentity,
            QueryAggregate::WorkBudgetExceeded => Self::WorkBudgetExceeded,
            QueryAggregate::InvalidScalar => Self::InvalidScalar,
            QueryAggregate::ArithmeticOverflow => Self::ArithmeticOverflow,
            QueryAggregate::SourceCountOverflow => Self::SourceCountOverflow,
            QueryAggregate::AmbiguousSourceRelation => Self::AmbiguousSourceRelation,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BankProjectionDenial {
    InvalidSnapshotVersion,
    MissingField(&'static str),
    MissingRelation(&'static str),
    AmbiguousRelation(&'static str),
    EntityResolution(BankEntityResolutionDenial),
    Traversal(BankInvariantProjectionTraversalDenial),
    Aggregate(BankInvariantAggregateDenialKind),
    DecisionPlan(BankInvariantDecisionPlanDenial),
    AccountingRevisionMismatch,
    InvalidDomainState(BankProposalDenial),
}

pub(crate) fn missing_field<T>(
    value: Option<T>,
    field: &'static str,
) -> Result<T, BankProjectionDenial> {
    value.ok_or(BankProjectionDenial::MissingField(field))
}

impl std::fmt::Display for BankProjectionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "bank invariant projection denied: {self:?}")
    }
}

impl std::error::Error for BankProjectionDenial {}

impl From<WorthQueryEntityResolutionDenial> for BankProjectionDenial {
    fn from(denial: WorthQueryEntityResolutionDenial) -> Self {
        Self::EntityResolution(BankEntityResolutionDenial::from_query(denial.kind()))
    }
}

impl From<WorthQueryInvariantProjectionTraversalDenial> for BankProjectionDenial {
    fn from(denial: WorthQueryInvariantProjectionTraversalDenial) -> Self {
        Self::Traversal(BankInvariantProjectionTraversalDenial::from_query(
            denial.kind(),
        ))
    }
}

impl From<WorthQueryInvariantDecisionPlanDenial> for BankProjectionDenial {
    fn from(denial: WorthQueryInvariantDecisionPlanDenial) -> Self {
        Self::DecisionPlan(BankInvariantDecisionPlanDenial::from_query(denial.kind()))
    }
}

impl From<WorthQueryInvariantAggregateDenial> for BankProjectionDenial {
    fn from(denial: WorthQueryInvariantAggregateDenial) -> Self {
        Self::Aggregate(BankInvariantAggregateDenialKind::from_query(denial.kind()))
    }
}
