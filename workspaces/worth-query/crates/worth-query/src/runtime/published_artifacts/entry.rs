use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use std::sync::Arc;

use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::runtime::async_result_identity::runtime_async_causality_identity;
use crate::runtime::computed::WorthQueryDerivedViewRuntime;
use crate::runtime::evidence_identities::{
    shared_read_bind_retained_artifact_label_identity, shared_read_republishing_causality_identity,
    shared_read_unpublished_causality_identity,
};
use crate::runtime::{
    WorthQueryDerivedArtifactBinding, WorthQueryDerivedMaterializationBundle,
    WorthQueryDerivedMaterializationReceipt, WorthQueryDerivedMaterializationResult,
    WorthQueryDerivedMaterializationTarget, WorthQueryRuntimeAsyncResultState,
    WorthQueryRuntimeAsyncResultStateKind, WorthQueryRuntimeError,
};

#[derive(Clone, Debug, PartialEq)]
pub(in crate::runtime) struct WorthQueryPublishedArtifactEntry {
    target: WorthQueryDerivedMaterializationTarget,
    published_binding: Option<Arc<WorthQueryDerivedArtifactBinding>>,
    async_result_state: Option<WorthQueryRuntimeAsyncResultState>,
}

impl WorthQueryPublishedArtifactEntry {
    pub(in crate::runtime) fn published(
        target: WorthQueryDerivedMaterializationTarget,
        binding: WorthQueryDerivedArtifactBinding,
        async_result_state: Option<WorthQueryRuntimeAsyncResultState>,
    ) -> Self {
        Self {
            target,
            published_binding: Some(Arc::new(binding)),
            async_result_state,
        }
    }

    pub(in crate::runtime) fn unpublished(
        target: WorthQueryDerivedMaterializationTarget,
        async_result_state: WorthQueryRuntimeAsyncResultState,
    ) -> Self {
        Self {
            target,
            published_binding: None,
            async_result_state: Some(async_result_state),
        }
    }

    pub(in crate::runtime) fn from_runtime_view(
        snapshot_identity: &WorthQuerySnapshotIdentity,
        view_name: &str,
        runtime_view: &WorthQueryDerivedViewRuntime,
    ) -> Result<Self, WorthQueryRuntimeError> {
        let target = WorthQueryDerivedMaterializationTarget::new(view_name);
        let evidence =
            crate::runtime::WorthQueryComputedInspectionEvidence::from_runtime(runtime_view)?;
        let receipt = WorthQueryDerivedMaterializationReceipt::from_evidence(
            &evidence,
            snapshot_identity.clone(),
        );
        let materialization = WorthQueryDerivedMaterializationResult::from_retained_rows(
            runtime_view.materialization.retained_rows().to_vec(),
            receipt,
        );
        let async_result_state = async_result_state_for_shared_read_entry(
            snapshot_identity,
            view_name,
            evidence.pending_patch_count(),
            evidence.pending_refresh_fallback_count(),
        )?;
        if runtime_view.materialization.is_published() {
            return Ok(Self::published(
                target.clone(),
                bind_shared_read_artifact(snapshot_identity, &target, materialization)?,
                async_result_state,
            ));
        }
        Ok(Self::unpublished(
            target,
            unpublished_async_result_state(snapshot_identity, view_name)?,
        ))
    }

    pub(in crate::runtime) fn published_binding(
        &self,
    ) -> Option<Arc<WorthQueryDerivedArtifactBinding>> {
        self.published_binding.clone()
    }

    pub(in crate::runtime) fn target(&self) -> &WorthQueryDerivedMaterializationTarget {
        &self.target
    }

    pub(in crate::runtime) fn async_result_state(
        &self,
    ) -> Option<WorthQueryRuntimeAsyncResultState> {
        self.async_result_state.clone()
    }
}

fn bind_shared_read_artifact(
    snapshot_identity: &WorthQuerySnapshotIdentity,
    target: &WorthQueryDerivedMaterializationTarget,
    materialization: WorthQueryDerivedMaterializationResult,
) -> Result<WorthQueryDerivedArtifactBinding, WorthQueryRuntimeError> {
    let bundle = WorthQueryDerivedMaterializationBundle::new(
        snapshot_identity.clone(),
        std::collections::BTreeMap::from([(target.clone(), materialization)]),
    )?;
    bundle.bind_retained_artifact_identity(
        shared_read_bind_retained_artifact_label_identity(
            target.terminal_view_name_projection(),
            &snapshot_identity.evidence_identity(),
        )?,
        [target.clone()],
    )
}

fn async_result_state_for_shared_read_entry(
    snapshot_identity: &WorthQuerySnapshotIdentity,
    view_name: &str,
    pending_patch_count: usize,
    pending_refresh_fallback_count: usize,
) -> Result<Option<WorthQueryRuntimeAsyncResultState>, IdentityDerivationDenial> {
    if pending_patch_count == 0 {
        return Ok(None);
    }

    let kind = if pending_refresh_fallback_count > 0 {
        WorthQueryRuntimeAsyncResultStateKind::Revalidating
    } else {
        WorthQueryRuntimeAsyncResultStateKind::Stale
    };

    Ok(Some(WorthQueryRuntimeAsyncResultState::new(
        kind,
        &runtime_async_causality_identity(&shared_read_republishing_causality_identity(
            view_name,
            kind,
            &snapshot_identity.evidence_identity(),
        )?),
        &snapshot_identity.evidence_identity(),
        &shared_read_generation_identity(snapshot_identity),
    )))
}

fn unpublished_async_result_state(
    snapshot_identity: &WorthQuerySnapshotIdentity,
    view_name: &str,
) -> Result<WorthQueryRuntimeAsyncResultState, IdentityDerivationDenial> {
    Ok(WorthQueryRuntimeAsyncResultState::new(
        WorthQueryRuntimeAsyncResultStateKind::Pending,
        &runtime_async_causality_identity(&shared_read_unpublished_causality_identity(
            view_name,
            &snapshot_identity.evidence_identity(),
        )?),
        &snapshot_identity.evidence_identity(),
        &shared_read_generation_identity(snapshot_identity),
    ))
}

fn shared_read_generation_identity(
    snapshot_identity: &WorthQuerySnapshotIdentity,
) -> crate::evidence_identity::WorthQueryEvidenceIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::SharedReadGeneration)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("snapshot_identity"),
                &snapshot_identity.evidence_identity(),
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}
