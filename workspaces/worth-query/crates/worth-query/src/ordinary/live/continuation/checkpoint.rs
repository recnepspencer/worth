use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use std::sync::Arc;

use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::ordinary::read::WorthQueryReadProjectionBinding;
use crate::runtime::{
    WorthQueryLiveView, WorthQueryManagedLiveWorkspaceCapability, WorthQueryUnrefinedLiveShape,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedLiveContinuationDurability {
    RuntimeBound,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryManagedLiveCheckpointReceipt {
    resource_name: String,
    installation_identity: WorthQueryEvidenceIdentity,
    basis_binding_identity: WorthQueryEvidenceIdentity,
    durability: WorthQueryManagedLiveContinuationDurability,
    pending_delivery_batch_count: usize,
    last_delivery_sequence: Option<u64>,
    continuation_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryManagedLiveCheckpointReceipt {
    pub(super) fn new(
        resource_name: &str,
        installation_identity: &WorthQueryEvidenceIdentity,
        basis_binding_identity: &WorthQueryEvidenceIdentity,
        pending_delivery_batch_count: usize,
        last_delivery_sequence: Option<u64>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let continuation_identity = continuation_identity(
            resource_name,
            installation_identity,
            basis_binding_identity,
            pending_delivery_batch_count,
            last_delivery_sequence,
        )?;
        Ok(Self {
            resource_name: resource_name.to_string(),
            installation_identity: installation_identity.clone(),
            basis_binding_identity: basis_binding_identity.clone(),
            durability: WorthQueryManagedLiveContinuationDurability::RuntimeBound,
            pending_delivery_batch_count,
            last_delivery_sequence,
            continuation_identity,
        })
    }

    pub fn resource_name(&self) -> &str {
        &self.resource_name
    }

    pub fn installation_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.installation_identity
    }

    pub fn basis_binding_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.basis_binding_identity
    }

    pub fn last_delivery_sequence(&self) -> Option<u64> {
        self.last_delivery_sequence
    }

    pub fn pending_delivery_batch_count(&self) -> usize {
        self.pending_delivery_batch_count
    }

    pub fn durability(&self) -> WorthQueryManagedLiveContinuationDurability {
        self.durability
    }

    pub fn continuation_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.continuation_identity
    }
}

#[derive(Debug)]
#[must_use = "a managed live continuation owns an active Query resource until resumed or dropped"]
pub struct WorthQueryManagedLiveContinuation {
    view: Option<WorthQueryLiveView<WorthQueryUnrefinedLiveShape>>,
    workspace_capability: Arc<WorthQueryManagedLiveWorkspaceCapability>,
    checkpoint: WorthQueryManagedLiveCheckpointReceipt,
    projection_binding: Option<WorthQueryReadProjectionBinding>,
}

impl WorthQueryManagedLiveContinuation {
    pub(super) fn new(
        view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape>,
        workspace_capability: Arc<WorthQueryManagedLiveWorkspaceCapability>,
        checkpoint: WorthQueryManagedLiveCheckpointReceipt,
        projection_binding: WorthQueryReadProjectionBinding,
    ) -> Self {
        Self {
            view: Some(view),
            workspace_capability,
            checkpoint,
            projection_binding: Some(projection_binding),
        }
    }

    pub fn checkpoint(&self) -> &WorthQueryManagedLiveCheckpointReceipt {
        &self.checkpoint
    }

    pub(super) fn view(&self) -> &WorthQueryLiveView<WorthQueryUnrefinedLiveShape> {
        self.view
            .as_ref()
            .expect("active continuation must retain its managed live view")
    }

    pub(super) fn workspace_capability(&self) -> &Arc<WorthQueryManagedLiveWorkspaceCapability> {
        &self.workspace_capability
    }

    pub(super) fn into_resource_parts(
        mut self,
    ) -> (
        WorthQueryLiveView<WorthQueryUnrefinedLiveShape>,
        Arc<WorthQueryManagedLiveWorkspaceCapability>,
        WorthQueryReadProjectionBinding,
    ) {
        let view = self
            .view
            .take()
            .expect("resumed continuation must retain its managed live view");
        let projection_binding = self
            .projection_binding
            .take()
            .expect("resumed continuation must retain its projection binding");
        (
            view,
            Arc::clone(&self.workspace_capability),
            projection_binding,
        )
    }
}

impl Drop for WorthQueryManagedLiveContinuation {
    fn drop(&mut self) {
        if let Some(view) = self.view.take() {
            self.workspace_capability.abandon(view);
        }
    }
}

fn continuation_identity(
    resource_name: &str,
    installation_identity: &WorthQueryEvidenceIdentity,
    basis_binding_identity: &WorthQueryEvidenceIdentity,
    pending_delivery_batch_count: usize,
    last_delivery_sequence: Option<u64>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_managed_live_continuation_v1",
            )
            .field_value(WorthQueryEvidenceTag::new("resource_name"), resource_name)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("installation"),
                installation_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("basis_binding"),
                basis_binding_identity,
            )
            .field_shape(WorthQueryEvidenceTag::new("durability"), "runtime_bound")
            .field_usize(
                WorthQueryEvidenceTag::new("pending_delivery_batch_count"),
                pending_delivery_batch_count,
            )
            .field_value(
                WorthQueryEvidenceTag::new("last_delivery_sequence"),
                last_delivery_sequence
                    .map(|sequence| sequence.to_string())
                    .unwrap_or_else(|| "none".to_string()),
            )
            .seal()?,
    )
}
