use worth_query_installation::facade::ApplicationSchema;

use super::outcome::{
    metadata, WorthQueryOrdinaryReadBatch, WorthQueryOrdinaryReadProjection,
    WorthQueryOrdinaryReadVersion,
};
use {
    super::super::WorthQueryAdmittedApplicationOperation,
    super::super::WorthQueryApplicationInvariantProjectionAuthority,
    super::super::WorthQueryApplicationOperationInvariantProjectionReader,
    super::super::WorthQueryOperationProjectionDenial,
    crate::domain_computation::primary_graph::vocabulary::invariant_projection::WorthQueryInvariantEntityIdentity,
};

impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn read_admitted_operation<Operation, Input, Scope, Output>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        projection: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>,
            &WorthQueryInvariantEntityIdentity<Schema, Scope>,
        ) -> WorthQueryOrdinaryReadBatch<Output>,
    ) -> Result<WorthQueryOrdinaryReadProjection<Output>, WorthQueryOperationProjectionDenial> {
        let completed = self.project_admitted_operation(
            phase,
            admission,
            |reader, root| {
                let version =
                    WorthQueryOrdinaryReadVersion::from_provider_version(reader.version().0);
                (projection(reader, root), version)
            },
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )?;
        let ((batch, version), snapshot, work) = completed.into_parts();
        drop(snapshot);
        let (output, result_count, truncated) = batch.into_parts();
        Ok(WorthQueryOrdinaryReadProjection::new(
            output,
            metadata(version, work, result_count, truncated),
        ))
    }
}
