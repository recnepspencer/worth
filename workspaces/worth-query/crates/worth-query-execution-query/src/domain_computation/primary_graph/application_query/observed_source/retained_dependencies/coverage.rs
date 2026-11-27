//! Fresh descriptor affinity and native-only original query continuity custody.
use super::SourceFactExpansionControl;
use super::*;
use crate::domain_computation::primary_graph::output_lineage::invalidation::QueryCoverageStop;
use crate::domain_computation::primary_graph::output_lineage::invalidation::QueryFactCoverage;
use crate::domain_computation::primary_graph::output_reuse::preflight_stop;
use crate::domain_computation::primary_graph::output_reuse::retained_fact_denial;
use crate::domain_computation::primary_graph::output_reuse::{
    first_moved_fact_in, RetainedFactVerification,
};
use std::sync::Arc;
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_lineage::facade::ports::ObservedSourceInternalPort as _;
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;

pub struct RetainedQuerySource {
    pub source: WorthQueryObservedSource<()>,
    pub coverage: Option<Arc<QueryFactCoverage>>,
}
impl RetainedQuerySource {
    pub fn shared(&self) -> Self {
        Self {
            source: self.source.retyped(),
            coverage: self.coverage.as_ref().map(Arc::clone),
        }
    }
}

pub fn coverage_stop(stop: QueryCoverageStop) -> WorthQueryRequiredSetCurrentnessDenial {
    match stop {
        QueryCoverageStop::Allocation(cause) => storage(cause),
        QueryCoverageStop::Verification(stop) => crate::domain_computation::primary_graph::output_reuse::verification_denial(
            crate::domain_computation::primary_graph::invariant_projection::consumed_output::map_verification_stop(stop),
            "original query read registration",
        ).into(),
        QueryCoverageStop::Native(stop) => retained_fact_denial(
            preflight_stop(stop),
            "original query continuity unavailable",
        )
        .into(),
    }
}

/// Native resolution admits its actual branch metadata copies before allocation.
pub fn selected_basis(
    relational: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
) -> Result<
    Option<worth_relational::facade::runtime::PositionedRelationalSnapshot>,
    WorthQueryRequiredSetCurrentnessDenial,
> {
    Ok(relational.read_truth().positioned_snapshot(snapshot).ok())
}

impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    /// Runs once at the genuine original query image; later joins only share grants.
    pub fn retain_original_dependencies<'a, Query: 'a>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relational: &worth_relational::facade::runtime::RelationalRuntime,
        sources: impl Iterator<Item = &'a WorthQueryObservedSource<Query>>,
        count: usize,
    ) -> Result<WorthQueryRetainedQueryDependencies, WorthQueryRequiredSetCurrentnessDenial> {
        let mut array = ExecutionArrayBuilder::allocate(count, execution.allocation_policy())
            .map_err(storage)?;
        let snapshot = self.application_basis().snapshot_handle();
        let basis = crate::domain_computation::primary_graph::output_lineage::CertificationBasis::of(
            self.product().lifecycle_incarnation(),
            self.product().reference_generation(),
            snapshot,
        );
        let mut verification = RetainedFactVerification::new(execution, relational, basis, None);

        let owner = &self
            .application()
            .runtime
            .primary_provider
            .graph_ref()
            .source_owner_ref()
            .invalidation_owner;
        for source in sources {
            execution
                .allocation_policy()
                .check_live()
                .map_err(storage)?;
            let prepared = source.admit_fact_materialization(
                self.application()
                    .runtime
                    .primary_provider
                    .graph_ref()
                    .layout_ref(),
                &source.query_identifier(),
                SourceFactExpansionControl::in_request(*execution, None),
            )?;
            let facts = prepared.into_facts()?;
            // Selector addresses do not certify the original query's truth. Its
            // native facts must answer at their exact original selected image.
            let current = first_moved_fact_in(&mut verification, facts.iter())
                .map_err(|stop| {
                    WorthQueryRequiredSetCurrentnessDenial::from(retained_fact_denial(
                        stop,
                        "original query capture unavailable",
                    ))
                })?
                .is_none();
            let coverage = if current {
                match selected_basis(relational, snapshot)? {
                    Some(basis) => owner
                        .retain_query_fact_coverage(basis, &facts, execution.allocation_policy())
                        .map_err(coverage_stop)?,
                    None => None,
                }
            } else {
                None
            };
            array
                .push(RetainedQuerySource {
                    source: source.retyped(),
                    coverage,
                })
                .map_err(storage)?;
        }
        Ok(WorthQueryRetainedQueryDependencies {
            sources: array.seal().map_err(storage)?,
        })
    }
}
