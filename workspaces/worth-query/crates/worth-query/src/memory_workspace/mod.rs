use crate::view_shape_live::ViewShapePatchEnvelope;
use worth_foundational::facade::{CanonicalFieldPath, FieldKey};
use worth_query_execution::facade::integration::WorthQueryRelationalSourceOwner;
use worth_relational::facade::identity::{EntityId, KindId};
use worth_runtime_bridge::facade::BridgeMutationAuthorityBundle;

mod atomic_batch;
mod commit_batch;
mod commit_snapshot_closeout;
mod entity_native_replacement;
#[cfg(test)]
mod entity_native_replacement_tests;
mod entity_row;
mod identities;
mod mutation_authority_admission;
mod native_patch;
mod native_value_access;
mod runtime_identity;
pub(crate) use runtime_identity::{
    snapshot_identity_from_admitted_basis, snapshot_identity_from_branch,
};
mod seed_rows;
#[cfg(test)]
mod tests;
mod transaction_denial;
mod truth_identity_admission;
mod workspace;
mod workspace_schema;

pub(crate) use atomic_batch::WorthQueryMemoryBatchMutation;
pub(crate) use entity_native_replacement::{
    WorthQueryEntityNativeReplacement, WorthQueryEntityNativeReplacementValue,
};
pub use entity_row::WorthQueryEntity;
pub use identities::{
    WorthQueryCommitIdentity, WorthQueryEntityIdentity, WorthQuerySnapshotIdentity,
};
pub(crate) use native_value_access::{aspect_relative_scalar, normalized_native_storage_path};
#[cfg(test)]
pub(crate) use truth_identity_admission::admit_external_commit_label;
pub(crate) use truth_identity_admission::{
    admit_authored_entity_label, admit_external_snapshot_label,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorthQueryAspect {
    touch: crate::runtime::WorthQueryAspectTouch,
    native_field_path: CanonicalFieldPath,
}

impl WorthQueryAspect {
    pub fn new(
        touch: crate::runtime::WorthQueryAspectTouch,
        native_field_path: CanonicalFieldPath,
    ) -> Result<Self, WorthQueryWorkspaceError> {
        ensure_touch_matches_native_field_path(&touch, &native_field_path)?;
        Ok(Self {
            touch,
            native_field_path,
        })
    }

    pub(crate) fn from_native_field_path(
        touch: crate::runtime::WorthQueryAspectTouch,
        native_field_path: CanonicalFieldPath,
    ) -> Self {
        Self {
            touch,
            native_field_path,
        }
    }

    pub fn aspect_touch(&self) -> &crate::runtime::WorthQueryAspectTouch {
        &self.touch
    }

    pub fn native_field_path(&self) -> &CanonicalFieldPath {
        &self.native_field_path
    }
}

fn ensure_touch_matches_native_field_path(
    touch: &crate::runtime::WorthQueryAspectTouch,
    native_field_path: &CanonicalFieldPath,
) -> Result<(), WorthQueryWorkspaceError> {
    let expected_aspect_root =
        FieldKey::new(touch.native_aspect_key().as_str()).ok_or_else(|| {
            WorthQueryWorkspaceError::new(format!(
                "aspect `{}` cannot anchor a memory workspace native field path",
                touch.native_aspect_key().as_str()
            ))
        })?;
    let native_fields = native_field_path.fields();
    if native_fields.first() != Some(&expected_aspect_root) {
        return Err(WorthQueryWorkspaceError::new(format!(
            "memory workspace aspect `{}` must use native field path rooted at `{}`",
            touch.admitted_touch_digest_part(),
            touch.native_aspect_key().as_str()
        )));
    }
    let Some(touch_field_path) = touch.native_field_path() else {
        return Ok(());
    };
    let expected = std::iter::once(expected_aspect_root)
        .chain(touch_field_path.fields().iter().cloned())
        .collect::<Vec<_>>();
    if native_fields != expected.as_slice() {
        return Err(WorthQueryWorkspaceError::new(format!(
            "memory workspace aspect `{}` must use matching native field path `{}`",
            touch.admitted_touch_digest_part(),
            expected
                .iter()
                .map(|field| field.as_str())
                .collect::<Vec<_>>()
                .join(".")
        )));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorthQueryMutationKind {
    Created,
    Updated,
    Deleted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorthQueryMutationDelta {
    pub(crate) collection_identity: crate::runtime::WorthQueryMutationTargetCollectionIdentity,
    pub(crate) entity_identity: WorthQueryEntityIdentity,
    pub(crate) kind: WorthQueryMutationKind,
    pub(crate) touched_aspects: Vec<crate::runtime::WorthQueryAspectTouch>,
}

impl WorthQueryMutationDelta {
    pub fn from_touched_aspects(
        collection: impl Into<String>,
        entity_identity: WorthQueryEntityIdentity,
        kind: WorthQueryMutationKind,
        touched_aspects: Vec<crate::runtime::WorthQueryAspectTouch>,
    ) -> Self {
        let collection = collection.into();
        Self {
            collection_identity: crate::runtime::WorthQueryMutationTargetCollectionIdentity::new(
                "mutation-delta-collection",
                collection,
            ),
            entity_identity,
            kind,
            touched_aspects,
        }
    }

    pub(crate) fn from_collection_identity(
        collection_identity: crate::runtime::WorthQueryMutationTargetCollectionIdentity,
        entity_identity: WorthQueryEntityIdentity,
        kind: WorthQueryMutationKind,
        touched_aspects: Vec<crate::runtime::WorthQueryAspectTouch>,
    ) -> Self {
        Self {
            collection_identity,
            entity_identity,
            kind,
            touched_aspects,
        }
    }

    pub fn collection(&self) -> &str {
        self.collection_identity.as_str()
    }

    pub fn target_collection_identity(
        &self,
    ) -> &crate::runtime::WorthQueryMutationTargetCollectionIdentity {
        &self.collection_identity
    }

    pub fn entity_identity(&self) -> &WorthQueryEntityIdentity {
        &self.entity_identity
    }

    pub fn kind(&self) -> &WorthQueryMutationKind {
        &self.kind
    }

    pub fn admitted_touched_aspects(&self) -> &[crate::runtime::WorthQueryAspectTouch] {
        &self.touched_aspects
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorthQueryMutationReceipt {
    pub(crate) commit_identity: WorthQueryCommitIdentity,
    pub(crate) snapshot_identity: WorthQuerySnapshotIdentity,
    pub(crate) deltas: Vec<WorthQueryMutationDelta>,
    pub(crate) bridge_authority: Option<BridgeMutationAuthorityBundle>,
}

impl WorthQueryMutationReceipt {
    pub(crate) fn has_current_mutation_authority(&self) -> bool {
        self.bridge_authority.is_some()
            && self.commit_identity.has_current_authority()
            && self.snapshot_identity.has_current_authority()
            && self
                .deltas
                .iter()
                .all(|delta| delta.entity_identity.has_current_authority())
    }

    pub(crate) fn admit_runtime_write_authority(mut self) -> Self {
        let Some(authority) = self.bridge_authority.as_ref() else {
            return self;
        };
        if !mutation_authority_admission::bridge_authority_admits_receipt(
            &self.commit_identity,
            &self.snapshot_identity,
            &self.deltas,
            authority,
        ) {
            return self;
        }
        self.commit_identity = self.commit_identity.admit_runtime_write_authority();
        self.snapshot_identity = self.snapshot_identity.admit_runtime_backend_authority();
        for delta in &mut self.deltas {
            delta.entity_identity = delta
                .entity_identity
                .clone()
                .admit_runtime_write_authority();
        }
        self
    }

    pub fn from_authoritative_parts(
        commit_identity: WorthQueryCommitIdentity,
        snapshot_identity: WorthQuerySnapshotIdentity,
        deltas: Vec<WorthQueryMutationDelta>,
    ) -> Self {
        Self {
            commit_identity,
            snapshot_identity,
            deltas,
            bridge_authority: None,
        }
    }

    pub fn from_bridge_authoritative_parts(
        commit_identity: WorthQueryCommitIdentity,
        snapshot_identity: WorthQuerySnapshotIdentity,
        deltas: Vec<WorthQueryMutationDelta>,
        bridge_authority: BridgeMutationAuthorityBundle,
    ) -> Self {
        Self {
            commit_identity,
            snapshot_identity,
            deltas,
            bridge_authority: Some(bridge_authority),
        }
    }

    pub fn commit_identity(&self) -> &WorthQueryCommitIdentity {
        &self.commit_identity
    }

    pub fn snapshot_identity(&self) -> &WorthQuerySnapshotIdentity {
        &self.snapshot_identity
    }

    pub fn deltas(&self) -> &[WorthQueryMutationDelta] {
        &self.deltas
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorthQueryLiveViewHandle {
    name: String,
}

impl WorthQueryLiveViewHandle {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorthQueryLivePatch {
    pub(crate) view_name: String,
    pub(crate) commit_identity: WorthQueryCommitIdentity,
    pub(crate) entity_identity: WorthQueryEntityIdentity,
    pub(crate) mutation_kind: WorthQueryMutationKind,
    pub(crate) touched_aspects: Vec<crate::runtime::WorthQueryAspectTouch>,
    pub(crate) envelope: ViewShapePatchEnvelope,
}

impl WorthQueryLivePatch {
    pub fn admitted_touched_aspects(&self) -> &[crate::runtime::WorthQueryAspectTouch] {
        &self.touched_aspects
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorthQueryWorkspaceError {
    kind: WorthQueryWorkspaceErrorKind,
    message: String,
}

impl WorthQueryWorkspaceError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            kind: WorthQueryWorkspaceErrorKind::Unclassified,
            message: message.into(),
        }
    }

    pub fn with_kind(kind: WorthQueryWorkspaceErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> WorthQueryWorkspaceErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for WorthQueryWorkspaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for WorthQueryWorkspaceError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorthQueryWorkspaceErrorKind {
    Unclassified,
    UnsupportedCollection,
    UnsupportedWriteFamily,
    EmptySchema,
    BatchAtomicityUnsupported,
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
    TransactionOverlayCapacityExhausted {
        maximum_bytes: u64,
        required_bytes: u64,
    },
    TransactionFootprintCapacityExhausted {
        maximum_loci: usize,
        required_loci: usize,
    },
    SavepointCapacityExhausted {
        maximum_savepoints: usize,
    },
    SavepointFootprintCapacityExhausted {
        maximum_loci: usize,
        required_loci: usize,
    },
    SavepointIdentityExhausted,
    CandidateCapacityExhausted {
        maximum_candidates: usize,
    },
    PublishedSnapshotCapacityExhausted {
        maximum_handles: usize,
    },
    CandidateIdentityExhausted,
    PreparedRootBudgetExhausted {
        maximum_bytes: u64,
        required_bytes: u64,
    },
    PatchPositionReservationContended,
    ProposalIdentityExhausted,
    RelationalBasisUnavailable,
}

pub struct WorthQueryMemoryWorkspace {
    runtime: WorthQueryRelationalSourceOwner,
    kind_id: KindId,
    kind_name: String,
    aspects: Vec<WorthQueryAspect>,
    aspect_contracts:
        crate::runtime::native_aspect_contracts::WorthQueryNativeAspectContractRegistry,
    next_client_key: u64,
}
