use std::sync::{Arc, Mutex};

use super::registry_evidence::WorthQueryArtifactLifecycleSnapshotGate;
use super::{
    WorthQueryArtifactDenial, WorthQueryArtifactDenialKind, WorthQueryArtifactLifecycleRecord,
    WorthQueryArtifactOccurrenceScope, WorthQueryArtifactSemanticProjection,
    WorthQueryErasedArtifactProviderResource, WorthQueryPreparedArtifactResource,
};

pub(crate) struct WorthQueryRuntimeArtifactOwner {
    binding: WorthQueryRuntimeArtifactBinding,
    semantic_projection: WorthQueryArtifactSemanticProjection,
    retained_bytes: usize,
    created_thread: std::thread::ThreadId,
    provider_access_session_identity: String,
    resource: Mutex<Option<Box<dyn WorthQueryErasedArtifactProviderResource>>>,
    occurrence_scope: Option<WorthQueryArtifactOccurrenceScope>,
    pub(super) lifecycle: Arc<WorthQueryArtifactLifecycleRecord>,
    pub(super) snapshot_gate: Arc<WorthQueryArtifactLifecycleSnapshotGate>,
}

pub(crate) struct WorthQueryRuntimeArtifactBinding {
    pub(super) contract:
        Arc<worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>,
    pub(super) domain_authority:
        Arc<crate::domain_computation::vocabulary::operation_binding::installed_domain_authority::WorthQueryInstalledDomainExecutionAuthority>,
    pub(super) operation_identity: String,
    pub(super) binding_identity: String,
    pub(super) run_identity: String,
    pub(super) producing_stage: String,
    pub(super) basis_identity: String,
    pub(super) provenance_identity: String,
    pub(super) dependency_identity: String,
    pub(super) owner_identity: String,
    pub(super) occurrence_identity: String,
}

impl WorthQueryRuntimeArtifactOwner {
    pub(super) fn register(
        binding: WorthQueryRuntimeArtifactBinding,
        prepared: WorthQueryPreparedArtifactResource,
        occurrence_scope: Option<WorthQueryArtifactOccurrenceScope>,
        snapshot_gate: Arc<WorthQueryArtifactLifecycleSnapshotGate>,
    ) -> Arc<Self> {
        let (semantic_projection, retained_bytes, resource) = prepared.into_owner_parts();
        let provider_access_session_identity =
            crate::domain_computation::artifact_identity::hash_artifact_identity_parts(&[
                "worth_query_artifact_provider_access_session_v1".into(),
                format!("owner:{}", binding.owner_identity),
                format!(
                    "provider:{}",
                    binding
                        .contract
                        .contract()
                        .ownership()
                        .provider_family()
                        .unwrap_or("unbound")
                ),
            ]);
        let owner = Arc::new(Self {
            binding,
            semantic_projection,
            retained_bytes,
            created_thread: std::thread::current().id(),
            provider_access_session_identity,
            resource: Mutex::new(Some(resource)),
            occurrence_scope,
            lifecycle: Arc::new(WorthQueryArtifactLifecycleRecord::new(retained_bytes)),
            snapshot_gate,
        });
        if let Some(scope) = &owner.occurrence_scope {
            scope.record_produced(retained_bytes);
        }
        owner
    }

    pub(super) fn binding(&self) -> &WorthQueryRuntimeArtifactBinding {
        &self.binding
    }

    pub(super) fn semantic_projection(&self) -> &WorthQueryArtifactSemanticProjection {
        &self.semantic_projection
    }

    pub(super) const fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }

    pub(super) fn created_thread(&self) -> std::thread::ThreadId {
        self.created_thread
    }

    pub(super) fn provider_access_session_identity(&self) -> &str {
        &self.provider_access_session_identity
    }

    pub(super) fn lifecycle_record(&self) -> Arc<WorthQueryArtifactLifecycleRecord> {
        Arc::clone(&self.lifecycle)
    }

    pub(super) fn with_native_access_provider<T>(
        &self,
        access: impl FnOnce(&dyn super::WorthQueryArtifactNativeAccessProvider) -> T,
    ) -> Option<T> {
        let resource = self
            .resource
            .lock()
            .expect("artifact provider resource lock must remain available");
        let provider = resource.as_ref()?.native_access_provider()?;
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| access(provider)));
        drop(resource);
        match outcome {
            Ok(value) => Some(value),
            Err(panic) => std::panic::resume_unwind(panic),
        }
    }

    pub(super) fn dispose_provider_if_required(&self, should_dispose: bool) {
        if !should_dispose {
            return;
        }
        let resource = self
            .resource
            .lock()
            .expect("artifact provider resource lock must remain available")
            .take()
            .expect("live artifact owner retains exactly one provider resource");
        let release = resource.release();
        self.record_occurrence_disposal();
        self.record_provider_release(release);
    }

    pub(super) fn denial(
        &self,
        kind: WorthQueryArtifactDenialKind,
        detail: &'static str,
    ) -> WorthQueryArtifactDenial {
        WorthQueryArtifactDenial::new(
            kind,
            Some(self.binding.contract.contract().family().as_str()),
            detail,
        )
    }

    fn record_occurrence_disposal(&self) {
        if let Some(scope) = &self.occurrence_scope {
            scope.record_disposed(self.retained_bytes);
        }
    }

    pub(super) fn record_provider_release(
        &self,
        release: super::WorthQueryArtifactProviderReleaseEvidence,
    ) {
        let _snapshot = self.snapshot_gate.lifecycle_mutation();
        self.lifecycle.record_provider_release(release);
    }
}

impl Drop for WorthQueryRuntimeArtifactOwner {
    fn drop(&mut self) {
        let Some(resource) = self
            .resource
            .get_mut()
            .expect("artifact provider resource lock must remain available")
            .take()
        else {
            return;
        };
        let release = resource.release();
        self.record_occurrence_disposal();
        let _snapshot = self.snapshot_gate.lifecycle_mutation();
        self.lifecycle.record_provider_release(release);
    }
}
