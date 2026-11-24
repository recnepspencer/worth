//! Dispatching target-entity matcher combining typed equality predicates and
//! an optional Foundation `where_filter` over the target entity.
use super::super::{NarrowingRead, RootPathSourceBuilder, RootSelectionWork};
use super::target_equality::TargetEqualityMatcher;
use super::target_filter::TargetFilterMatcher;
use crate::domain_computation::primary_graph::application_query::read_execution::{
    read_execution_denial, WorthQueryApplicationReadExecutionDenial,
    WorthQueryApplicationReadExecutionDenialKind,
};
use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
use worth_query_installation::facade::WorthQueryInstalledRelatedPredicate;
use worth_relational::facade::identity::{EntityId, KindId};

pub(super) enum TargetMatcher<'plan> {
    AnyLiveTarget,
    Filtered {
        subject: &'plan str,
        target_entity: &'plan str,
        target_kind: KindId,
        equality: Option<TargetEqualityMatcher<'plan>>,
        filter: Option<TargetFilterMatcher<'plan>>,
    },
}

impl<'plan> TargetMatcher<'plan> {
    pub fn new<ReadIdentity>(
        read: &NarrowingRead<'_, 'plan, '_, '_, ReadIdentity>,
        predicate: &'plan WorthQueryInstalledRelatedPredicate,
        target_kind: KindId,
        allocation: &SourceFactsAllocation,
        work: &mut RootSelectionWork<'_>,
    ) -> Result<Self, WorthQueryApplicationReadExecutionDenial> {
        if predicate.target_predicates().is_empty() && predicate.target_where_filter().is_none() {
            return Ok(Self::AnyLiveTarget);
        }
        let subject = predicate.relation();
        let equality = if predicate.target_predicates().is_empty() {
            None
        } else {
            Some(TargetEqualityMatcher::new(
                read,
                predicate.target_predicates(),
                target_kind,
                subject,
                allocation,
                work,
            )?)
        };
        let filter = match predicate.target_where_filter() {
            None => None,
            Some(filter) => Some(TargetFilterMatcher::new(
                read,
                predicate.target_entity(),
                filter,
                subject,
                allocation,
                work,
            )?),
        };
        Ok(Self::Filtered {
            subject,
            target_entity: predicate.target_entity(),
            target_kind,
            equality,
            filter,
        })
    }

    pub const fn is_unfiltered(&self) -> bool {
        matches!(self, Self::AnyLiveTarget)
    }

    /// Evaluates one target entity, observing its entity identity and every
    /// inspected target field into `evidence` before reading.
    pub fn matches<ReadIdentity>(
        &self,
        read: &NarrowingRead<'_, '_, '_, '_, ReadIdentity>,
        target_id: EntityId,
        evidence: &mut RootPathSourceBuilder,
        work: &mut RootSelectionWork<'_>,
    ) -> Result<bool, WorthQueryApplicationReadExecutionDenial> {
        match self {
            Self::AnyLiveTarget => {
                evidence.observe_entity(read.execution, target_id)?;
                Ok(true)
            }
            Self::Filtered {
                subject,
                target_entity,
                target_kind,
                equality,
                filter,
            } => {
                evidence.observe_entity(read.execution, target_id)?;
                if let Some(eq) = equality
                    && !eq.matches(
                        read,
                        target_id,
                        target_entity,
                        *target_kind,
                        subject,
                        evidence,
                        work,
                    )?
                {
                    return Ok(false);
                }
                if let Some(flt) = filter {
                    return flt.matches(read, target_id, subject, evidence, work);
                }
                Ok(true)
            }
        }
    }
}

pub(super) fn projection_unavailable(subject: &str) -> WorthQueryApplicationReadExecutionDenial {
    read_execution_denial(
        WorthQueryApplicationReadExecutionDenialKind::ProjectionUnavailable,
        subject,
    )
}
