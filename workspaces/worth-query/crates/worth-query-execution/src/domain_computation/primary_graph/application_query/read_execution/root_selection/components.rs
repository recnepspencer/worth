//! Complete governed Native components become roots only after whole-cohort visibility.
use super::super::interruption::NativeReadInterruption;
use super::*;
use crate::domain_computation::primary_graph::{
    aggregate_observation::{AggregateObservation, AggregateQuery, AggregateSelection},
    application_query::{
        observed_source::{DeclaredComponentSelection, WorthQueryObservedRootSelection},
        read_execution::read_plan::ReadPlan,
    },
    component_selection::{
        acquisition::{
            self, CohortSourceCapture, ComponentAcquisitionReader, CouplingObservationDenial,
        },
        ComponentSelection, ComponentSelectionSubject,
    },
    root_path_selection::{
        RootPathOrientation, RootPathReadBasis, RootPathSelection, RootPathSelectionRequest,
    },
    schema_layout::ComponentSetLayout,
    source_facts::{SourceFacts, SourceFactsAllocation, SourceFactsBuilder},
};
use std::{collections::BTreeMap, sync::Arc};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryComponentMeaning, ApplicationQueryComponentSelection,
};
use worth_relational::facade::{
    runtime::RelationalPinnedRead, transactions::RelationalExecutionDenialCause as Cause,
};
mod sources;
pub(super) fn select(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalPinnedRead,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    plan: &ReadPlan<'_>,
    work: &mut RootSelectionWork<'_>,
) -> Result<BoundedRootSelection, WorthQueryApplicationReadExecutionDenial> {
    let meaning = plan
        .contract
        .component_selection()
        .expect("component mode was selected");
    if !plan.governance.is_disclosed(&meaning.slot_key()) {
        return Err(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::ProjectionUnavailable,
            meaning.output_name(),
        ));
    }
    let layout = graph
        .component_set(meaning.set())
        .filter(|layout| layout.member.as_ref() == meaning.member())
        .ok_or_else(|| path_union::traversal_denial(meaning.set()))?;
    let projection = runtime
        .project_snapshot(plan.snapshot)
        .ok_or_else(|| path_union::traversal_denial(meaning.set()))?;
    path_union::prepare_steps(execution, plan.contract.root_paths(), work, meaning.set())?;
    let interruption = NativeReadInterruption::new(work.interruption, meaning.set());
    let acquired = execution.run_native_with_checkpoint(|context| {
        let control = || {
            execution.consult().map_err(Cause::from)?;
            interruption.checkpoint()
        };
        if !layout
            .matches_native_definition(runtime, control)
            .map_err(|cause| super::super::denial::relational_read_denial(cause, meaning.set()))?
        {
            return Err(path_union::traversal_denial(meaning.set()));
        }
        let allocation = SourceFactsAllocation::open(execution)
            .map_err(|cause| super::super::denial::relational_read_denial(cause, meaning.set()))?;
        let mut reader = QueryComponentAcquisition {
            allocation: Arc::clone(&allocation),
            basis: RootPathReadBasis {
                execution,
                runtime,
                graph,
                projection,
            },
            plan,
            work,
            layout,
            interruption: &interruption,
        };
        let member = (meaning.selection() == ApplicationQueryComponentSelection::OfScopeMember)
            .then_some(plan.root);
        acquisition::acquire(
            &mut reader,
            execution,
            &allocation,
            plan.root,
            member,
            CohortSourceCapture::PerCohort,
            context,
        )
    });
    if let Some(stop) = interruption.denial() {
        return Err(stop);
    }
    let acquired = match acquired {
        Ok((answer, _)) => answer?,
        Err(stop) => {
            return Err(super::super::denial::relational_read_denial(
                stop.into(),
                meaning.set(),
            ))
        }
    };
    sources::complete(acquired, meaning, execution, graph, plan, work)
}
struct QueryComponentAcquisition<'frame, 'plan, 'work, 'execution, 'request> {
    allocation: Arc<SourceFactsAllocation>,
    basis: RootPathReadBasis<'frame, 'execution, 'request>,
    plan: &'frame ReadPlan<'plan>,
    work: &'frame mut RootSelectionWork<'work>,
    layout: &'frame ComponentSetLayout,
    interruption: &'frame NativeReadInterruption<'work, 'plan>,
}
impl ComponentAcquisitionReader for QueryComponentAcquisition<'_, '_, '_, '_, '_> {
    type Error = WorthQueryApplicationReadExecutionDenial;
    fn paths(
        &mut self,
        start: EntityId,
        orientation: RootPathOrientation,
        capture: bool,
    ) -> Result<RootPathSelection, Self::Error> {
        path_union::select_paths(
            &self.basis,
            self.plan,
            self.work,
            RootPathSelectionRequest {
                allocation: Some(&self.allocation),
                paths: self.plan.contract.root_paths(),
                start,
                orientation,
                capture_set: capture,
                subject: self.plan.name,
            },
        )
        .inspect_err(|error| self.interruption.remember(error))
    }
    fn aggregate(&mut self, member: EntityId) -> Result<Arc<AggregateObservation>, Self::Error> {
        crate::domain_computation::primary_graph::component_selection::storage::aggregate::<EntityId>(
            self.basis.execution, &self.allocation, self.plan.name,
        ).map_err(|cause| self.native_denial(cause))?;
        super::super::entry_observation::record_component_native_acquisition();
        super::aggregate::read(
            self.basis.execution,
            self.basis.runtime,
            self.work,
            AggregateQuery {
                allocation: Some(&self.allocation),
                subject: self.plan.name,
                entity_kind: self.layout.member_kind,
                selection: AggregateSelection::ComponentDefinition(self.layout.definition),
                member,
                maximum_work: None,
                control: None,
            },
            self.interruption,
        )
    }
    fn observe_cohort(
        &mut self,
        body: &AggregateObservation,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<SourceFacts, Self::Error> {
        acquisition::observe_coupling(self.basis.execution,self.basis.runtime,&self.allocation,self.layout,body,context).map_err(|stop|match stop {
            CouplingObservationDenial::Unavailable=>path_union::traversal_denial(self.plan.name),
            CouplingObservationDenial::Execution(cause)=>self.native_denial(cause),
            CouplingObservationDenial::Source(crate::domain_computation::primary_graph::source_facts::SourceFactReadDenial::Execution(cause))=>self.native_denial(cause),
            CouplingObservationDenial::Source(_)=>path_union::traversal_denial(self.plan.name),
        })
    }
    fn native_denial(&mut self, cause: Cause) -> Self::Error {
        super::super::denial::relational_read_denial(cause, self.plan.name)
    }
    fn outside_scope(&self) -> Self::Error {
        path_union::traversal_denial(self.plan.name)
    }
}
