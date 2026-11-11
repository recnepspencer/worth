use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use std::cmp::Ordering;
use std::collections::BTreeMap;

use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::runtime::surface::live_artifact_binding::WorthQueryLiveArtifactBinding;
use crate::runtime::WorthQueryLiveView;
#[cfg(test)]
use crate::runtime::{record_forbidden_fallback_seam_invocation, WorthQueryForbiddenFallbackSeam};
use crate::runtime::{WorthQueryRuntimeError, WorthQueryRuntimeLiveSubscriptionInstallation};

use super::WorthQueryLiveReadResult;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLiveArtifactTarget {
    view_name: String,
    installation: Option<WorthQueryRuntimeLiveSubscriptionInstallation>,
}

impl Ord for WorthQueryLiveArtifactTarget {
    fn cmp(&self, other: &Self) -> Ordering {
        self.view_name.cmp(&other.view_name)
    }
}

impl PartialOrd for WorthQueryLiveArtifactTarget {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl WorthQueryLiveArtifactTarget {
    pub(crate) fn view_name(&self) -> &str {
        &self.view_name
    }

    pub fn terminal_view_name_projection(&self) -> &str {
        self.view_name()
    }

    pub(crate) fn from_view_name(view_name: impl Into<String>) -> Self {
        Self {
            view_name: view_name.into(),
            installation: None,
        }
    }

    pub fn from_source_adapter_declared_view_name(view_name: impl Into<String>) -> Self {
        Self::from_view_name(view_name)
    }

    pub(in crate::runtime) fn from_subscription_installation(
        installation: &WorthQueryRuntimeLiveSubscriptionInstallation,
    ) -> Self {
        Self {
            view_name: installation.view_name().to_string(),
            installation: Some(installation.clone()),
        }
    }

    pub(in crate::runtime) fn subscription_installation(
        &self,
    ) -> Option<&WorthQueryRuntimeLiveSubscriptionInstallation> {
        self.installation.as_ref()
    }

    #[cfg(test)]
    pub(crate) fn test_only(view_name: impl Into<String>) -> Self {
        Self::from_view_name(view_name)
    }
}

impl<T> From<&WorthQueryLiveView<T>> for WorthQueryLiveArtifactTarget {
    fn from(value: &WorthQueryLiveView<T>) -> Self {
        Self {
            view_name: value.name().to_string(),
            installation: Some(value.subscription_installation().clone()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryLiveArtifactBundle {
    snapshot_identity: WorthQuerySnapshotIdentity,
    snapshot_evidence_identity: WorthQueryEvidenceIdentity,
    bundle_digest: String,
    reads: BTreeMap<WorthQueryLiveArtifactTarget, WorthQueryLiveReadResult>,
}

impl WorthQueryLiveArtifactBundle {
    pub(in crate::runtime) fn new(
        snapshot_identity: WorthQuerySnapshotIdentity,
        reads: BTreeMap<WorthQueryLiveArtifactTarget, WorthQueryLiveReadResult>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let snapshot_evidence_identity = snapshot_identity.evidence_identity();
        let bundle_digest =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LiveArtifactBundle)
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("snapshot_identity"),
                    &snapshot_evidence_identity,
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("read_result"),
                    reads.iter().map(|(target, result)| {
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
            reads,
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
        self.reads.len()
    }

    pub fn targets(&self) -> impl Iterator<Item = WorthQueryLiveArtifactTarget> + '_ {
        self.reads.keys().cloned()
    }

    pub fn terminal_target_view_names_projection(&self) -> impl Iterator<Item = &str> {
        self.reads
            .keys()
            .map(WorthQueryLiveArtifactTarget::terminal_view_name_projection)
    }

    pub fn includes_target(&self, target: &WorthQueryLiveArtifactTarget) -> bool {
        self.reads.contains_key(target)
    }

    pub fn read<T>(
        &self,
        view: &WorthQueryLiveView<T>,
    ) -> Result<&WorthQueryLiveReadResult, WorthQueryRuntimeError> {
        let target = WorthQueryLiveArtifactTarget::from(view);
        self.read_for_target(&target)
    }

    pub(crate) fn read_for_target(
        &self,
        target: &WorthQueryLiveArtifactTarget,
    ) -> Result<&WorthQueryLiveReadResult, WorthQueryRuntimeError> {
        self.reads.get(target).ok_or_else(|| {
            WorthQueryRuntimeError::ReadCompositionDenied(
                crate::runtime::WorthQueryReadDenial::new(
                    crate::runtime::WorthQueryReadDenialKind::ExecutionDenied,
                    format!(
                        "live artifact bundle did not retain requested live view `{}`",
                        target.terminal_view_name_projection()
                    ),
                ),
            )
        })
    }

    pub fn bind_live_artifact(
        self,
        artifact_name: impl Into<String>,
        required_targets: impl IntoIterator<Item = WorthQueryLiveArtifactTarget>,
    ) -> Result<WorthQueryLiveArtifactBinding, WorthQueryRuntimeError> {
        #[cfg(test)]
        record_forbidden_fallback_seam_invocation(
            WorthQueryForbiddenFallbackSeam::BindLiveArtifact,
        );
        WorthQueryLiveArtifactBinding::bind(self, artifact_name, required_targets)
    }

    #[cfg(test)]
    pub(crate) fn test_only(
        snapshot_identity: WorthQuerySnapshotIdentity,
        reads: BTreeMap<WorthQueryLiveArtifactTarget, WorthQueryLiveReadResult>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::new(snapshot_identity, reads)
    }
}
