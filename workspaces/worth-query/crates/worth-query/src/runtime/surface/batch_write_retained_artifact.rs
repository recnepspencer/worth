use super::{
    WorthQueryBatchWriteReceipt, WorthQueryDerivedArtifactBinding,
    WorthQueryDerivedMaterializationTarget,
};
use crate::runtime::{
    WorthQueryBatchWriteReceiptInspection, WorthQueryInspection, WorthQueryRuntimeError,
    WorthQueryWorkspace,
};

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryBatchWriteRetainedArtifact {
    receipt: WorthQueryBatchWriteReceipt,
    inspection: WorthQueryBatchWriteReceiptInspection,
    retained_artifact: WorthQueryDerivedArtifactBinding,
}

impl WorthQueryBatchWriteRetainedArtifact {
    pub(in crate::runtime) fn build(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        workspace: &mut WorthQueryWorkspace,
        receipt: &WorthQueryBatchWriteReceipt,
        artifact_name: impl Into<String>,
        targets: impl IntoIterator<Item = WorthQueryDerivedMaterializationTarget>,
    ) -> Result<Self, WorthQueryRuntimeError> {
        let inspection = match workspace.inspect(execution, receipt)? {
            WorthQueryInspection::BatchWriteReceipt(inspection) => inspection,
            other => {
                return Err(WorthQueryRuntimeError::Workspace(
                    crate::memory_workspace::WorthQueryWorkspaceError::new(format!(
                        "expected batch-write receipt inspection, got `{other:?}`",
                    )),
                ));
            }
        };
        let retained_artifact =
            workspace.materialize_derived_artifact_binding(execution, artifact_name, targets)?;
        Ok(Self {
            receipt: receipt.clone(),
            inspection,
            retained_artifact,
        })
    }

    pub fn receipt(&self) -> &WorthQueryBatchWriteReceipt {
        &self.receipt
    }

    pub fn inspection(&self) -> &WorthQueryBatchWriteReceiptInspection {
        &self.inspection
    }

    pub fn retained_artifact(&self) -> &WorthQueryDerivedArtifactBinding {
        &self.retained_artifact
    }
}
