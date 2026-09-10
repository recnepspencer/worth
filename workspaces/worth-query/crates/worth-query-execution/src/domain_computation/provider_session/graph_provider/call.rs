use std::sync::Arc;

use worth_query_installation::facade::WorthQueryExecutionResourceEnvelope;

use super::call_identity::WorthQueryGraphCallAuthorityIdentity;
use super::{
    WorthQueryBoundGraphExecutionReceipt, WorthQueryExecutionGraphReadStreamEvidence,
    WorthQueryGraphCallBindingDenial, WorthQueryGraphProviderCallKind,
    WorthQueryGraphProviderFailure, WorthQueryGraphProviderReceipt,
    WorthQueryGraphReceiptAdmissionDenial, WorthQueryProviderWorkReport,
};
use crate::domain_computation::domain_evidence_binding::WorthQueryBoundExecutionSnapshotIdentity;
use crate::domain_computation::provider_session::{
    WorthQueryClosedExecutionAttemptIdentity, WorthQueryExecutionProviderSession,
    WorthQueryExecutionProviderSessionIdentity, WorthQueryExecutionResourceAttemptEvidence,
};
use crate::execution_digest::hash_parts;

mod readmission;

pub(crate) use readmission::WorthQueryGraphProviderCallReadmissionPlan;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphProviderCallRequest {
    kind: WorthQueryGraphProviderCallKind,
    scope_identity: Arc<str>,
    stage_identity: Option<Arc<str>>,
    snapshot_identity: Option<Arc<str>>,
}

impl WorthQueryGraphProviderCallRequest {
    pub fn direct(
        kind: WorthQueryGraphProviderCallKind,
        scope_identity: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            kind,
            scope_identity: scope_identity.into(),
            stage_identity: None,
            snapshot_identity: None,
        }
    }

    pub fn workflow_stage(
        kind: WorthQueryGraphProviderCallKind,
        scope_identity: impl Into<Arc<str>>,
        stage_identity: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            kind,
            scope_identity: scope_identity.into(),
            stage_identity: Some(stage_identity.into()),
            snapshot_identity: None,
        }
    }

    pub(in crate::domain_computation) fn with_managed_execution_snapshot(
        mut self,
        snapshot_identity: impl Into<Arc<str>>,
    ) -> Self {
        self.snapshot_identity = Some(snapshot_identity.into());
        self
    }

    pub(in crate::domain_computation::provider_session) fn kind(
        &self,
    ) -> WorthQueryGraphProviderCallKind {
        self.kind
    }

    pub(in crate::domain_computation::provider_session) fn execution_snapshot_identity(
        &self,
    ) -> Option<&str> {
        self.snapshot_identity.as_deref()
    }

    pub(in crate::domain_computation::provider_session) fn stage_identity(&self) -> Option<&str> {
        self.stage_identity.as_deref()
    }

    pub(in crate::domain_computation::provider_session) fn into_spec(
        self,
        binding: &crate::domain_computation::operation_binding::WorthQueryExecutionBoundOperationAuthority,
        graph: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
    ) -> WorthQueryGraphProviderCallSpec {
        let scope = WorthQueryGraphCallScope {
            scope_identity: self.scope_identity,
            operation_identity: Arc::from(binding.operation_identity()),
            binding_identity: Arc::from(binding.binding_identity()),
            stage_identity: self.stage_identity,
        };
        WorthQueryGraphProviderCallSpec {
            kind: self.kind,
            scope,
            read_binding: WorthQueryGraphCallReadBinding {
                graph_role: Arc::from(graph.role()),
                canonical_query_digest: Arc::from(binding.canonical_query_digest()),
                basis_identity: Arc::from(binding.basis_identity()),
                snapshot_identity: WorthQueryBoundExecutionSnapshotIdentity::capture(
                    self.snapshot_identity
                        .expect("provider session validates managed execution basis"),
                ),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::provider_session) struct WorthQueryGraphCallScope {
    pub(super) scope_identity: Arc<str>,
    pub(super) operation_identity: Arc<str>,
    pub(super) binding_identity: Arc<str>,
    pub(super) stage_identity: Option<Arc<str>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::provider_session) struct WorthQueryGraphCallReadBinding {
    pub(super) graph_role: Arc<str>,
    pub(super) canonical_query_digest: Arc<str>,
    pub(super) basis_identity: Arc<str>,
    pub(super) snapshot_identity: WorthQueryBoundExecutionSnapshotIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::provider_session) struct WorthQueryGraphProviderCallSpec {
    pub(super) kind: WorthQueryGraphProviderCallKind,
    pub(super) scope: WorthQueryGraphCallScope,
    pub(super) read_binding: WorthQueryGraphCallReadBinding,
}

#[derive(Clone, Debug)]
pub struct WorthQueryGraphProviderCall {
    authority_identity: WorthQueryGraphCallAuthorityIdentity,
    call_identity: Arc<str>,
    provider_session_identity: WorthQueryExecutionProviderSessionIdentity,
    attempt_identity: WorthQueryClosedExecutionAttemptIdentity,
    spec: WorthQueryGraphProviderCallSpec,
    execution_resources: WorthQueryExecutionResourceAttemptEvidence,
    resource_envelope: Arc<WorthQueryExecutionResourceEnvelope>,
}

impl WorthQueryGraphProviderCall {
    pub(in crate::domain_computation::provider_session) fn mint(
        session: &WorthQueryExecutionProviderSession,
        spec: WorthQueryGraphProviderCallSpec,
        execution_resources: &WorthQueryExecutionResourceAttemptEvidence,
        resource_envelope: Arc<WorthQueryExecutionResourceEnvelope>,
    ) -> Result<Self, WorthQueryGraphCallBindingDenial> {
        if spec.kind == WorthQueryGraphProviderCallKind::CommitAdmission {
            return Err(WorthQueryGraphCallBindingDenial::CommitKindRequiresCommitCall);
        }
        if execution_resources.provider_session_identity() != session.identity()
            || execution_resources.provider_session_attempt_identity() != session.attempt_identity()
        {
            return Err(WorthQueryGraphCallBindingDenial::ForeignResourceAttempt);
        }
        Ok(Self::mint_validated(
            session,
            spec,
            execution_resources,
            resource_envelope,
        ))
    }

    fn mint_validated(
        session: &WorthQueryExecutionProviderSession,
        spec: WorthQueryGraphProviderCallSpec,
        execution_resources: &WorthQueryExecutionResourceAttemptEvidence,
        resource_envelope: Arc<WorthQueryExecutionResourceEnvelope>,
    ) -> Self {
        let authority_identity = WorthQueryGraphCallAuthorityIdentity::mint();
        let call_identity = Arc::<str>::from(hash_parts(&[
            "worth_query_graph_provider_call_v2".into(),
            format!("authority:{}", authority_identity.as_u64()),
            format!("session:{}", session.identity()),
            format!("operation:{}", spec.scope.operation_identity),
            format!("binding:{}", spec.scope.binding_identity),
            format!("role:{}", spec.read_binding.graph_role),
            format!("query:{}", spec.read_binding.canonical_query_digest),
            format!("basis:{}", spec.read_binding.basis_identity),
            format!("snapshot:{}", spec.read_binding.snapshot_identity.as_str()),
            format!("kind:{}", spec.kind.as_str()),
            format!("scope:{}", spec.scope.scope_identity),
            format!(
                "stage:{}",
                spec.scope.stage_identity.as_deref().unwrap_or("direct")
            ),
            format!("resources:{}", execution_resources.identity()),
        ]));
        Self {
            authority_identity,
            call_identity,
            provider_session_identity: session.closed_identity(),
            attempt_identity: session.closed_attempt_identity(),
            spec,
            execution_resources: execution_resources.clone(),
            resource_envelope,
        }
    }

    pub fn operation_identity(&self) -> &str {
        &self.spec.scope.operation_identity
    }

    pub fn kind(&self) -> WorthQueryGraphProviderCallKind {
        self.spec.kind
    }

    pub fn scope_identity(&self) -> &str {
        &self.spec.scope.scope_identity
    }

    pub fn graph_role(&self) -> &str {
        &self.spec.read_binding.graph_role
    }

    pub fn binding_identity(&self) -> &str {
        &self.spec.scope.binding_identity
    }

    pub fn canonical_query_digest(&self) -> &str {
        &self.spec.read_binding.canonical_query_digest
    }

    pub fn basis_identity(&self) -> &str {
        &self.spec.read_binding.basis_identity
    }

    pub fn snapshot_identity(&self) -> &str {
        self.spec.read_binding.snapshot_identity.as_str()
    }

    pub fn execution_resources(&self) -> &WorthQueryExecutionResourceAttemptEvidence {
        &self.execution_resources
    }

    pub fn resource_envelope(&self) -> &WorthQueryExecutionResourceEnvelope {
        &self.resource_envelope
    }

    pub(crate) fn stage_identity(&self) -> Option<&str> {
        self.spec.scope.stage_identity.as_deref()
    }

    pub(crate) fn completed(
        &self,
        provider_receipt: impl Into<Arc<str>>,
        work_report: WorthQueryProviderWorkReport,
    ) -> WorthQueryGraphProviderReceipt {
        WorthQueryGraphProviderReceipt::completed(
            self.authority_identity,
            provider_receipt,
            work_report,
        )
    }

    pub(crate) fn streamed(
        &self,
        provider_receipt: impl Into<Arc<str>>,
        stream: WorthQueryExecutionGraphReadStreamEvidence,
        work_report: WorthQueryProviderWorkReport,
    ) -> Result<WorthQueryGraphProviderReceipt, WorthQueryGraphProviderFailure> {
        if self.kind() != WorthQueryGraphProviderCallKind::Project {
            return Err(WorthQueryGraphProviderFailure::new(
                WorthQueryGraphReceiptAdmissionDenial::UnexpectedProjection.detail(),
            ));
        }
        Ok(WorthQueryGraphProviderReceipt::streamed(
            self.authority_identity,
            provider_receipt,
            Arc::new(stream),
            work_report,
        ))
    }

    pub fn admit_receipt(
        &self,
        receipt: WorthQueryGraphProviderReceipt,
    ) -> Result<WorthQueryBoundGraphExecutionReceipt, WorthQueryGraphReceiptAdmissionDenial> {
        receipt.admit_graph_call(self)
    }

    pub(super) fn authority_identity(&self) -> WorthQueryGraphCallAuthorityIdentity {
        self.authority_identity
    }

    pub(crate) fn call_identity(&self) -> &str {
        &self.call_identity
    }

    pub(super) fn call_identity_arc(&self) -> Arc<str> {
        Arc::clone(&self.call_identity)
    }

    pub(super) fn provider_session_identity_arc(&self) -> Arc<str> {
        self.provider_session_identity.description()
    }

    pub(super) fn canonical_query_digest_arc(&self) -> Arc<str> {
        Arc::clone(&self.spec.read_binding.canonical_query_digest)
    }

    pub(super) fn basis_identity_arc(&self) -> Arc<str> {
        Arc::clone(&self.spec.read_binding.basis_identity)
    }

    pub(super) fn snapshot_identity_arc(&self) -> Arc<str> {
        self.spec.read_binding.snapshot_identity.description()
    }

    pub(super) fn provider_session_identity(&self) -> &str {
        self.provider_session_identity.as_str()
    }

    pub(super) fn closed_provider_session_identity(
        &self,
    ) -> WorthQueryExecutionProviderSessionIdentity {
        self.provider_session_identity.clone()
    }

    pub(super) fn closed_attempt_identity(&self) -> WorthQueryClosedExecutionAttemptIdentity {
        self.attempt_identity.clone()
    }

    pub(super) fn spec(&self) -> &WorthQueryGraphProviderCallSpec {
        &self.spec
    }
}
