//! Fresh descriptor affinity and native-only original query continuity custody.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::retained_decision_facts::StorageControl,
    output_lineage::invalidation::{
        InvalidationEditAdmission, QueryCoverageStop, QueryFactCoverage,
    },
    output_reuse::{first_moved_fact, preflight_stop, retained_fact_denial},
};
use std::sync::Arc;

pub(super) struct RetainedQuerySource {
    pub(super) source: WorthQueryObservedSource<()>,
    pub(super) coverage: Option<Arc<QueryFactCoverage>>,
}
impl RetainedQuerySource {
    pub(super) fn shared(&self) -> Self {
        Self {
            source: self.source.retyped(),
            coverage: self.coverage.as_ref().map(Arc::clone),
        }
    }
}

pub(super) fn coverage_stop(stop: QueryCoverageStop) -> WorthQueryRequiredSetCurrentnessDenial {
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
pub(super) fn selected_basis(
    relational: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    admission: &mut InvalidationEditAdmission,
) -> Result<
    Option<worth_relational::facade::runtime::PositionedRelationalSnapshot>,
    WorthQueryRequiredSetCurrentnessDenial,
> {
    use worth_relational::facade::{
        mvcc::CompanionPreflightStop, runtime::RelationalSnapshotPositionAdmissionStop as Stop,
    };
    match relational
        .read_truth()
        .positioned_snapshot_admitted(snapshot, |work, bytes| {
            admission.charge_external_work(work)?;
            admission.admit_read_scratch(bytes)
        }) {
        Ok(basis) => Ok(Some(basis)),
        Err(Stop::Position(_)) => Ok(None),
        Err(Stop::Admission(stop)) => Err(coverage_stop(QueryCoverageStop::Native(stop))),
        Err(Stop::AccountingOverflow) => Err(coverage_stop(QueryCoverageStop::Native(
            CompanionPreflightStop::WorkCounterOverflow,
        ))),
    }
}

impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    /// Runs once at the genuine original query image; later joins only share grants.
    pub(super) fn retain_original_dependencies<'a, Query: 'a>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relational: &worth_relational::facade::runtime::RelationalRuntime,
        sources: impl Iterator<Item = &'a WorthQueryObservedSource<Query>>,
        count: usize,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<WorthQueryRetainedQueryDependencies, WorthQueryRequiredSetCurrentnessDenial> {
        let mut array = ExecutionArrayBuilder::allocate(count, execution.allocation_policy())
            .map_err(storage)?;
        let snapshot = self.application_basis().snapshot_handle();
        let owner = &self
            .application()
            .primary_provider
            .graph
            .source_owner
            .invalidation_owner;
        for source in sources {
            execution
                .allocation_policy()
                .check_live()
                .map_err(storage)?;
            let prepared = source.admit_fact_materialization(
                &self.application().primary_provider.graph.layout,
                admission,
                &source.query_identifier,
            )?;
            let facts =
                prepared.into_facts(StorageControl::new(execution.allocation_policy(), None))?;
            // Selector addresses do not certify the original query's truth. Its
            // native facts must answer at their exact original selected image.
            let current =
                first_moved_fact(execution, relational, snapshot, facts.iter(), admission)
                    .map_err(|stop| {
                        WorthQueryRequiredSetCurrentnessDenial::from(retained_fact_denial(
                            stop,
                            "original query capture unavailable",
                        ))
                    })?
                    .is_none();
            let coverage = if current {
                match selected_basis(relational, snapshot, admission)? {
                    Some(basis) => owner
                        .retain_query_fact_coverage(
                            basis,
                            &facts,
                            execution.allocation_policy(),
                            admission,
                        )
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
