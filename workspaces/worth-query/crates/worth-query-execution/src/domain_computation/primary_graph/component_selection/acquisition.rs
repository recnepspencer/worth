//! One whole-cohort algorithm; each caller supplies its genuine admission boundary.
use super::*;
use crate::domain_computation::primary_graph::root_path_selection::{
    RootPathOrientation, RootPathSelection,
};
use std::collections::{BTreeMap, BTreeSet};
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
}
impl AcquiredComponents {
    /// Operation commitments retain their original acquisition order for facts.
    pub fn into_operation_parts(self) -> (Vec<Arc<AggregateObservation>>, Vec<SourceFacts>) {
        let mut aggregates = Vec::new();
        let mut facts = Vec::new();
        let is_census = self.census.is_some();
        facts.extend(self.census);
        for cohort in self.cohorts {
            aggregates.push(cohort.aggregate);
            if !is_census {
                facts.extend(cohort.visibility);
            }
            facts.push(cohort.coupling);
        }
        aggregates.sort_by_key(|body| body.native().identity());
        (aggregates, facts)
    }
}
pub(in crate::domain_computation::primary_graph) fn acquire<R: ComponentAcquisitionReader>(
    reader: &mut R,
    root: EntityId,
    member: Option<EntityId>,
    capture: CohortSourceCapture,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<AcquiredComponents, R::Error> {
    let mut census = None;
    let mut witnesses = BTreeMap::new();
    let universe = if let Some(member) = member {
        BTreeSet::from([member])
    } else {
        let selected = reader.paths(root, RootPathOrientation::ForwardCensus, true)?;
        let universe = selected.roots.keys().copied().collect();
        if capture == CohortSourceCapture::PerCohort {
            witnesses = selected.roots;
        }
        census = Some(
            selected
                .set_source
                .expect("census captures its full negative universe")
                .finish()
                .map_err(|cause| reader.native_denial(cause))?,
        );
        universe
    };
    let mut seen = BTreeSet::new();
    let mut cohorts = Vec::new();
    for anchor in &universe {
        AggregateObservation::checkpoint(context).map_err(|cause| reader.native_denial(cause))?;
        if seen.contains(anchor) {
            continue;
        }
        let aggregate = reader.aggregate(*anchor)?;
        let mut visibility = Vec::new();
        for candidate in aggregate.native().members() {
            AggregateObservation::checkpoint(context)
                .map_err(|cause| reader.native_denial(cause))?;
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
            } else {
                if !universe.contains(&candidate) {
                    return Err(reader.outside_scope());
                }
                if capture == CohortSourceCapture::SelectionOnly {
                    seen.insert(candidate);
                    continue;
                }
                witnesses
                    .remove(&candidate)
                    .expect("the complete forward universe retains each actual member witness")
            };
            // Census row witnesses are retained for Query's independent cohort granule.
            visibility.push(
                source
                    .finish()
                    .map_err(|cause| reader.native_denial(cause))?,
            );
            seen.insert(candidate);
        }
        let coupling = reader.observe_cohort(&aggregate, context)?;
        cohorts.push(AcquiredCohort {
            aggregate,
            visibility,
            coupling,
        });
    }
    Ok(AcquiredComponents { census, cohorts })
}
