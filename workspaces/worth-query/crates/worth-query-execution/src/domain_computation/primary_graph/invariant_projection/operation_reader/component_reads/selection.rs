//! Actual operation admission around the shared physical whole-cohort collector.
use super::*;
use crate::domain_computation::primary_graph::{
    component_selection::acquisition::{
        self, CohortSourceCapture, ComponentAcquisitionReader, CouplingObservationDenial,
    },
    root_path_selection::{RootPathOrientation, RootPathSelection},
};
impl<Schema: ApplicationSchema, Operation: ApplicationOperationMarkerIdentity<Schema>>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    pub(super) fn collect_component_selection<Entity>(
        &mut self,
        scope: &WorthQueryOperationComponentReadScope,
        layout: &ComponentSetLayout,
        root: EntityId,
        member: Option<EntityId>,
        pin: &RelationalPinnedRead,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
        subject: &str,
        allocation: &Arc<
            crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation,
        >,
    ) -> Result<(Vec<Arc<AggregateObservation>>, Vec<SourceFacts>), Denial> {
        let execution = *self.reader.execution;
        let mut reader = OperationComponentAcquisition::<Schema, Operation, Entity> {
            reader: self,
            allocation: Arc::clone(allocation),
            scope,
            layout,
            pin,
            subject,
            _entity: std::marker::PhantomData,
        };
        let answer = acquisition::acquire(
            &mut reader,
            &execution,
            allocation,
            root,
            member,
            CohortSourceCapture::SelectionOnly,
            context,
        )?;
        answer
            .into_operation_parts(&execution)
            .map_err(|cause| reader.native_denial(cause))
    }
}
struct OperationComponentAcquisition<'frame, 'reader, 'runtime, Schema, Operation, Entity> {
    allocation: Arc<crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation>,
    reader: &'frame mut WorthQueryApplicationOperationInvariantProjectionReader<
        'reader,
        'runtime,
        Schema,
        Operation,
    >,
    scope: &'frame WorthQueryOperationComponentReadScope,
    layout: &'frame ComponentSetLayout,
    pin: &'frame RelationalPinnedRead,
    subject: &'frame str,
    _entity: std::marker::PhantomData<Entity>,
}
impl<Schema: ApplicationSchema, Operation: ApplicationOperationMarkerIdentity<Schema>, Entity>
    ComponentAcquisitionReader
    for OperationComponentAcquisition<'_, '_, '_, Schema, Operation, Entity>
{
    type Error = Denial;
    fn paths(
        &mut self,
        start: EntityId,
        orientation: RootPathOrientation,
        capture: bool,
    ) -> Result<RootPathSelection, Denial> {
        self.reader.select_component_paths(
            self.scope,
            start,
            orientation,
            capture,
            self.pin,
            self.subject,
            &self.allocation,
        )
    }
    fn aggregate(&mut self, member: EntityId) -> Result<Arc<AggregateObservation>, Denial> {
        crate::domain_computation::primary_graph::component_selection::storage::aggregate::<EntityId>(
            self.reader.reader.execution, &self.allocation, self.subject,
        ).map_err(|cause| self.native_denial(cause))?;
        self.reader.reader.read_aggregate_identity(
            self.layout.member_kind,
            AggregateSelection::ComponentDefinition(self.layout.definition),
            member,
            self.subject,
            Some(&self.allocation),
        )
    }
    fn observe_cohort(
        &mut self,
        body: &AggregateObservation,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<SourceFacts, Denial> {
        self.reader.reader.admit_component_members::<Entity>(
            self.layout,
            body,
            context,
            self.subject,
        )?;
        acquisition::observe_coupling(
            self.reader.reader.execution,
            self.pin,
            &self.allocation,
            self.layout,
            body,
            context,
        )
        .map_err(|stop| match stop {
            CouplingObservationDenial::Unavailable => {
                Denial::new(Kind::EndpointUnavailable, self.subject)
            }
            CouplingObservationDenial::Execution(cause) => self.native_denial(cause),
            CouplingObservationDenial::Source(cause) => {
                self.reader.component_source_denial(cause, self.subject)
            }
        })
    }
    fn native_denial(
        &mut self,
        cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
    ) -> Denial {
        self.reader.reader.walk_stop(cause, self.subject)
    }
    fn outside_scope(&self) -> Denial {
        Denial::new(Kind::ComponentOutsideScope, self.subject)
    }
}
