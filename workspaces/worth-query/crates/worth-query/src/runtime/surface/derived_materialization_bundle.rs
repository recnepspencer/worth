use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use std::collections::BTreeMap;

use super::derived_artifact_binding::WorthQueryDerivedArtifactBinding;
use super::derived_materialization_result::WorthQueryDerivedMaterializationResult;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::runtime::computed::WorthQueryDerivedViewHandle;
use crate::runtime::WorthQueryRuntimeError;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryDerivedMaterializationTarget {
    view_name: String,
}

impl WorthQueryDerivedMaterializationTarget {
    pub(in crate::runtime) fn new(view_name: impl Into<String>) -> Self {
        Self {
            view_name: view_name.into(),
        }
    }

    pub(crate) fn view_name(&self) -> &str {
        &self.view_name
    }

    pub fn terminal_view_name_projection(&self) -> &str {
        self.view_name()
    }

    #[cfg(test)]
    pub(crate) fn test_only(view_name: impl Into<String>) -> Self {
        Self::new(view_name)
    }
}

impl<T> From<&WorthQueryDerivedViewHandle<T>> for WorthQueryDerivedMaterializationTarget {
    fn from(value: &WorthQueryDerivedViewHandle<T>) -> Self {
        Self::new(value.name())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryDerivedMaterializationBundle {
    snapshot_identity: WorthQuerySnapshotIdentity,
    snapshot_evidence_identity: WorthQueryEvidenceIdentity,
    bundle_digest: String,
    materializations:
        BTreeMap<WorthQueryDerivedMaterializationTarget, WorthQueryDerivedMaterializationResult>,
}

impl WorthQueryDerivedMaterializationBundle {
    pub(in crate::runtime) fn new(
        snapshot_identity: WorthQuerySnapshotIdentity,
        materializations: BTreeMap<
            WorthQueryDerivedMaterializationTarget,
            WorthQueryDerivedMaterializationResult,
        >,
    ) -> Result<Self, IdentityDerivationDenial> {
        let snapshot_evidence_identity = snapshot_identity.evidence_identity()?;
        let bundle_digest = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::DerivedMaterializationBundle,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("snapshot_identity"),
            &snapshot_evidence_identity,
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("materialization_result"),
            materializations.iter().map(|(target, result)| {
                format!(
                    "{}:{}",
                    target.terminal_view_name_projection(),
                    result.receipt().result_digest()
                )
            }),
        )
        .seal()?
        .terminal_projection_for_reporting()
        .to_string();
        Ok(Self {
            snapshot_identity,
            snapshot_evidence_identity,
            bundle_digest,
            materializations,
        })
    }

    pub fn snapshot_identity(&self) -> &WorthQuerySnapshotIdentity {
        &self.snapshot_identity
    }

    pub fn snapshot_evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.snapshot_evidence_identity
    }

    pub fn bundle_digest(&self) -> &str {
        &self.bundle_digest
    }

    pub fn target_count(&self) -> usize {
        self.materializations.len()
    }

    pub fn targets(&self) -> impl Iterator<Item = WorthQueryDerivedMaterializationTarget> + '_ {
        self.materializations.keys().cloned()
    }

    pub fn terminal_target_view_names_projection(&self) -> impl Iterator<Item = &str> {
        self.materializations
            .keys()
            .map(WorthQueryDerivedMaterializationTarget::terminal_view_name_projection)
    }

    pub fn includes_target(&self, target: &WorthQueryDerivedMaterializationTarget) -> bool {
        self.materializations.contains_key(target)
    }

    pub fn materialization<T>(
        &self,
        view: &WorthQueryDerivedViewHandle<T>,
    ) -> Result<&WorthQueryDerivedMaterializationResult, WorthQueryRuntimeError> {
        let target = WorthQueryDerivedMaterializationTarget::from(view);
        self.materialization_for_target(&target)
    }

    pub(crate) fn materialization_for_target(
        &self,
        target: &WorthQueryDerivedMaterializationTarget,
    ) -> Result<&WorthQueryDerivedMaterializationResult, WorthQueryRuntimeError> {
        self.materializations
            .get(target)
            .ok_or_else(|| WorthQueryRuntimeError::RetainedRowDecode {
                view_name: target.terminal_view_name_projection().to_string(),
                stage: "derived-materialization-bundle",
                message: "bundle did not retain the requested derived surface".to_string(),
            })
    }

    pub fn bind_retained_artifact(
        self,
        artifact_name: impl Into<String>,
        required_targets: impl IntoIterator<Item = WorthQueryDerivedMaterializationTarget>,
    ) -> Result<WorthQueryDerivedArtifactBinding, WorthQueryRuntimeError> {
        WorthQueryDerivedArtifactBinding::bind(self, artifact_name, required_targets)
    }

    pub fn bind_retained_artifact_identity(
        self,
        artifact_identity: WorthQueryEvidenceIdentity,
        required_targets: impl IntoIterator<Item = WorthQueryDerivedMaterializationTarget>,
    ) -> Result<WorthQueryDerivedArtifactBinding, WorthQueryRuntimeError> {
        WorthQueryDerivedArtifactBinding::bind_with_identity(
            self,
            artifact_identity,
            required_targets,
        )
    }

    #[cfg(test)]
    pub(crate) fn test_only(
        snapshot_identity: WorthQuerySnapshotIdentity,
        materializations: BTreeMap<
            WorthQueryDerivedMaterializationTarget,
            WorthQueryDerivedMaterializationResult,
        >,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(snapshot_identity, materializations)?)
    }
}
