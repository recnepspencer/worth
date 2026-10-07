#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthServerOperationScope {
    Workspace {
        tenant_id: String,
        workspace_id: String,
    },
    WorkspaceBranch {
        tenant_id: String,
        workspace_id: String,
        branch_label: String,
    },
    ProductDraft {
        tenant_id: String,
        workspace_id: String,
        product_session_identity: String,
        draft_scope: String,
    },
    DurableProductAuthority {
        tenant_id: String,
        workspace_id: String,
        authority_scope: String,
    },
    SyncLease {
        tenant_id: String,
        workspace_id: String,
        branch_label: String,
        lease_target: String,
    },
}

impl WorthServerOperationScope {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        use crate::product_adapter::execution_pipeline::read_batch_accounting::string;
        match self {
            Self::Workspace {
                tenant_id,
                workspace_id,
            } => string(tenant_id).saturating_add(string(workspace_id)),
            Self::WorkspaceBranch {
                tenant_id,
                workspace_id,
                branch_label,
            } => string(tenant_id)
                .saturating_add(string(workspace_id))
                .saturating_add(string(branch_label)),
            _ => u64::MAX,
        }
    }

    pub(crate) fn workspace(tenant_id: impl Into<String>, workspace_id: impl Into<String>) -> Self {
        Self::Workspace {
            tenant_id: tenant_id.into(),
            workspace_id: workspace_id.into(),
        }
    }

    pub(crate) fn workspace_branch(
        tenant_id: impl Into<String>,
        workspace_id: impl Into<String>,
        branch_label: impl Into<String>,
    ) -> Self {
        Self::WorkspaceBranch {
            tenant_id: tenant_id.into(),
            workspace_id: workspace_id.into(),
            branch_label: branch_label.into(),
        }
    }

    pub(crate) fn product_draft(
        tenant_id: impl Into<String>,
        workspace_id: impl Into<String>,
        product_session_identity: impl Into<String>,
        draft_scope: impl Into<String>,
    ) -> Self {
        Self::ProductDraft {
            tenant_id: tenant_id.into(),
            workspace_id: workspace_id.into(),
            product_session_identity: product_session_identity.into(),
            draft_scope: draft_scope.into(),
        }
    }

    pub(crate) fn sync_lease(
        tenant_id: impl Into<String>,
        workspace_id: impl Into<String>,
        branch_label: impl Into<String>,
        lease_target: impl Into<String>,
    ) -> Self {
        Self::SyncLease {
            tenant_id: tenant_id.into(),
            workspace_id: workspace_id.into(),
            branch_label: branch_label.into(),
            lease_target: lease_target.into(),
        }
    }

    pub(crate) fn durable_product_authority(
        tenant_id: impl Into<String>,
        workspace_id: impl Into<String>,
        authority_scope: impl Into<String>,
    ) -> Self {
        Self::DurableProductAuthority {
            tenant_id: tenant_id.into(),
            workspace_id: workspace_id.into(),
            authority_scope: authority_scope.into(),
        }
    }

    pub(crate) fn canonical_digest(&self) -> String {
        let digest = match self {
            Self::Workspace {
                tenant_id,
                workspace_id,
            } => crate::canonical_digest::WorthServerCanonicalDigestBuilder::new(
                "worth-server-operation-scope-v2",
            )
            .field("kind", "workspace")
            .field("tenant", tenant_id)
            .field("workspace", workspace_id),
            Self::WorkspaceBranch {
                tenant_id,
                workspace_id,
                branch_label,
            } => crate::canonical_digest::WorthServerCanonicalDigestBuilder::new(
                "worth-server-operation-scope-v2",
            )
            .field("kind", "workspace-branch")
            .field("tenant", tenant_id)
            .field("workspace", workspace_id)
            .field("branch", branch_label),
            Self::ProductDraft {
                tenant_id,
                workspace_id,
                product_session_identity,
                draft_scope,
            } => crate::canonical_digest::WorthServerCanonicalDigestBuilder::new(
                "worth-server-operation-scope-v2",
            )
            .field("kind", "product-draft")
            .field("tenant", tenant_id)
            .field("workspace", workspace_id)
            .field("session", product_session_identity)
            .field("draft", draft_scope),
            Self::DurableProductAuthority {
                tenant_id,
                workspace_id,
                authority_scope,
            } => crate::canonical_digest::WorthServerCanonicalDigestBuilder::new(
                "worth-server-operation-scope-v2",
            )
            .field("kind", "durable-product-authority")
            .field("tenant", tenant_id)
            .field("workspace", workspace_id)
            .field("scope", authority_scope),
            Self::SyncLease {
                tenant_id,
                workspace_id,
                branch_label,
                lease_target,
            } => crate::canonical_digest::WorthServerCanonicalDigestBuilder::new(
                "worth-server-operation-scope-v2",
            )
            .field("kind", "sync-lease")
            .field("tenant", tenant_id)
            .field("workspace", workspace_id)
            .field("branch", branch_label)
            .field("target", lease_target),
        };
        digest.finish()
    }

    pub(crate) fn breadth(&self) -> usize {
        match self {
            Self::Workspace { .. } => 2,
            Self::WorkspaceBranch { .. } => 3,
            Self::ProductDraft { .. } | Self::SyncLease { .. } => 4,
            Self::DurableProductAuthority { .. } => 3,
        }
    }
}
