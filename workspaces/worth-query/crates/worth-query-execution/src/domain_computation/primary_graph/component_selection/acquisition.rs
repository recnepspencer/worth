//! One whole-cohort algorithm; each caller supplies its genuine admission boundary.
use super::*;
use crate::domain_computation::primary_graph::root_path_selection::{
    RootPathOrientation, RootPathSelection,
};
use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
use std::collections::BTreeMap;
use worth_execution::ExecutionRequest;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
mod coupling;
pub(in crate::domain_computation::primary_graph) use coupling::{
    observe_coupling, CouplingObservationDenial,
};

pub(in crate::domain_computation::primary_graph) trait ComponentAcquisitionReader {
    type Error;
    fn paths(
        &mut self,
        start: EntityId,
        orientation: RootPathOrientation,
        capture: bool,
    ) -> Result<RootPathSelection, Self::Error>;
    fn aggregate(&mut self, member: EntityId) -> Result<Arc<AggregateObservation>, Self::Error>;
    fn observe_cohort(
        &mut self,
        body: &AggregateObservation,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<SourceFacts, Self::Error>;
    fn native_denial(&mut self, cause: Cause) -> Self::Error;
    fn outside_scope(&self) -> Self::Error;
}
#[derive(Clone, Copy, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum CohortSourceCapture {
    SelectionOnly,
    PerCohort,
}
pub(in crate::domain_computation::primary_graph) struct AcquiredCohort {
    pub aggregate: Arc<AggregateObservation>,
    pub visibility: Vec<SourceFacts>,
    pub coupling: SourceFacts,
}
pub(in crate::domain_computation::primary_graph) struct AcquiredComponents {
    pub census: Option<SourceFacts>,
    pub cohorts: Vec<AcquiredCohort>,
    pub allocation: Arc<SourceFactsAllocation>,
}
impl AcquiredComponents {
    /// Operation commitments retain their original acquisition order for facts.
    pub fn into_operation_parts(
        self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(Vec<Arc<AggregateObservation>>, Vec<SourceFacts>), Cause> {
        let is_census = self.census.is_some();
        let count = self
            .cohorts
            .iter()
            .try_fold(usize::from(is_census), |count, cohort| {
                count.checked_add(1)?.checked_add(if is_census {
                    0
                } else {
                    cohort.visibility.len()
                })
            })
            .ok_or(Cause::ChargedBytesOverflow)?;
        let mut aggregates =
            super::storage::vector(execution, &self.allocation, self.cohorts.len())?;
        let mut facts = super::storage::vector(execution, &self.allocation, count)?;
        facts.extend(self.census);
        for cohort in self.cohorts {
            aggregates.push(cohort.aggregate);
            if !is_census {
                facts.extend(cohort.visibility);
            }
            facts.push(cohort.coupling);
        }
        // Native identity is unique; no stable-sort scratch is required.
        aggregates.sort_unstable_by_key(|body| body.native().identity());
        Ok((aggregates, facts))
    }
}

pub(in crate::domain_computation::primary_graph) fn acquire<R: ComponentAcquisitionReader>(
    reader: &mut R,
    execution: &ExecutionRequest<'_, '_>,
    allocation: &Arc<SourceFactsAllocation>,
    root: EntityId,
    member: Option<EntityId>,
    capture: CohortSourceCapture,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<AcquiredComponents, R::Error> {
    let mut census = None;
    let mut witnesses = BTreeMap::new();
    if member.is_none() {
        let selected = reader.paths(root, RootPathOrientation::ForwardCensus, true)?;
        witnesses = selected.roots;
        census = Some(
            selected
                .set_source
                .expect("census captures its full negative universe"),
        );
    }
    let maximum_cohorts = member.map_or(witnesses.len(), |_| 1);
    let mut cohorts = super::storage::vector(execution, allocation, maximum_cohorts)
        .map_err(|cause| reader.native_denial(cause))?;
    let mut single = member;
    loop {
        let (anchor, mut anchor_source) = if let Some(member) = single.take() {
            (member, None)
        } else if let Some((anchor, source)) = witnesses.pop_first() {
            (anchor, Some(source))
        } else {
            break;
        };
        AggregateObservation::checkpoint(context).map_err(|cause| reader.native_denial(cause))?;
        let aggregate = reader.aggregate(anchor)?;
        let capacity = if member.is_some() || capture == CohortSourceCapture::PerCohort {
            aggregate.native().len()
        } else {
            0
        }
        .checked_add(1)
        .ok_or_else(|| reader.native_denial(Cause::ChargedBytesOverflow))?;
        // The final coupling fact will use the already admitted spare slot.
        let mut visibility = super::storage::vector(execution, allocation, capacity)
            .map_err(|cause| reader.native_denial(cause))?;
        let mut previous = None;
        for candidate in aggregate.native().members() {
            AggregateObservation::checkpoint(context)
                .map_err(|cause| reader.native_denial(cause))?;
            if previous.is_some_and(|previous| previous >= candidate) {
                return Err(reader.outside_scope());
            }
            previous = Some(candidate);
            let source = if member.is_some() {
                let mut selected = reader.paths(
                    candidate,
                    RootPathOrientation::ReverseWitness {
                        required_root: root,
                    },
                    false,
                )?;
                selected
                    .roots
                    .remove(&root)
                    .ok_or_else(|| reader.outside_scope())?
            } else if candidate == anchor {
                anchor_source.take().ok_or_else(|| reader.outside_scope())?
            } else {
                witnesses
                    .remove(&candidate)
                    .ok_or_else(|| reader.outside_scope())?
            };
            if member.is_some() || capture == CohortSourceCapture::PerCohort {
                visibility.push(source);
            }
        }
        if anchor_source.is_some() {
            return Err(reader.outside_scope());
        }
        let coupling = reader.observe_cohort(&aggregate, context)?;
        cohorts.push(AcquiredCohort {
            aggregate,
            visibility,
            coupling,
        });
    }
    Ok(AcquiredComponents {
        census,
        cohorts,
        allocation: Arc::clone(allocation),
    })
}
