use worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity;
use worth_query_installation::facade::{ApplicationSchema, OperationReads};
use worth_relational::facade::runtime::ProjectionAspectScope;
use worth_relational::facade::storage::RecordLifecycleState;

use super::super::WorthQueryCurrentOutputDenial;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOperationInvariantProjectionReader, WorthQueryInvariantEntityIdentity,
};

impl<'reader, 'runtime, Schema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'reader, 'runtime, Schema, Operation>
where
    Schema: ApplicationSchema,
{
    pub(super) fn current_source_is_live<Producer>(
        &mut self,
        producer: &WorthQueryInvariantEntityIdentity<Schema, Producer>,
    ) -> Result<bool, WorthQueryCurrentOutputDenial>
    where
        Producer: ApplicationEntityMarkerIdentity<Schema> + OperationReads<Operation>,
    {
        self.require_current_output_budget(1, Producer::IDENTIFIER)?;
        self.reader.work_budget.consume(1);
        let live = self
            .reader
            .runtime
            .read_truth()
            .project_snapshot(self.reader.snapshot)
            .and_then(|view| {
                view.entity_record_with_projection_scope(
                    producer.entity_id(),
                    ProjectionAspectScope::empty(),
                    |record| Some((record.kind_id(), record.lifecycle())),
                )
            })
            .is_some_and(|(kind, lifecycle)| {
                lifecycle == RecordLifecycleState::Live
                    && self.reader.layout.entity_name(kind) == Some(Producer::IDENTIFIER)
            });
        Ok(live)
    }
}
