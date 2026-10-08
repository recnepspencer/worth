use worth_query_installation::facade::ApplicationSchema;

use super::super::{
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationInvariantProjectionAuthority,
    WorthQueryApplicationOperationInvariantProjectionReader, WorthQueryInvariantEntityIdentity,
    WorthQueryOperationProjectionDenial,
};
use super::outcome::{
    metadata, WorthQueryOrdinaryReadBatch, WorthQueryOrdinaryReadProjection,
    WorthQueryOrdinaryReadVersion,
};

impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn read_admitted_operation<Operation, Input, Scope, Output>(
        &self,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        projection: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>,
            &WorthQueryInvariantEntityIdentity<Schema, Scope>,
        ) -> WorthQueryOrdinaryReadBatch<Output>,
    ) -> Result<WorthQueryOrdinaryReadProjection<Output>, WorthQueryOperationProjectionDenial> {
        let completed = self.project_admitted_operation(
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
