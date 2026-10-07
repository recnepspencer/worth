use super::*;

pub(super) struct CertificationSnapshotIdentity;

impl WorthQueryRuntimeSnapshotIdentityAdapter for CertificationSnapshotIdentity {
    fn current_snapshot_identity(
        &self,
    ) -> Result<
        WorthQuerySnapshotIdentity,
        worth_query_execution::facade::primary_graph::WorthQueryHandleDenial,
    > {
        Ok(certification_snapshot_identity(
            "certification-runtime-current-snapshot",
        ))
    }
}
