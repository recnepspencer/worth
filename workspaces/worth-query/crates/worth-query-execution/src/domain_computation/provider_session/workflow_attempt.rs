use std::sync::{Arc, Mutex, Weak};

use worth_query_admission::facade::resource_admission::{
    WorthQueryAdmittedExecutionResourcePlan, WorthQueryAdmittedWorkflowResourcePlan,
};
use worth_query_admission::integration::{
    WorthQueryCapacityReservedWorkflowResourcePlan, WorthQueryExecutionCapacityReleaseReceipt,
};

use super::{
    WorthQueryExecutionAttemptIdentity, WorthQueryExecutionProviderSession,
    WorthQueryExecutionResourceAttemptEvidence,
};
pub struct WorthQueryWorkflowExecutionResourceAttempt {
    pub(in crate::domain_computation::provider_session) reserved:
        WorthQueryCapacityReservedWorkflowResourcePlan,
    pub(in crate::domain_computation::provider_session) attempt_identity:
        WorthQueryExecutionAttemptIdentity,
    pub(in crate::domain_computation::provider_session) provider_session:
        WorthQueryExecutionProviderSession,
    pub(in crate::domain_computation::provider_session) evidence:
        WorthQueryExecutionResourceAttemptEvidence,
    pub(in crate::domain_computation::provider_session) artifact_run:
        Mutex<WorthQueryWorkflowArtifactRunState>,
}

pub(in crate::domain_computation::provider_session) struct WorthQueryWorkflowArtifactRunState {
    next_generation: u64,
    active_registry:
        Option<Weak<crate::domain_computation::artifact_owner::WorthQueryWorkflowArtifactRegistry>>,
}

impl WorthQueryWorkflowExecutionResourceAttempt {
    pub(crate) fn start(
        mut reserved: WorthQueryCapacityReservedWorkflowResourcePlan,
        binding_authority: &crate::domain_computation::operation_binding::WorthQueryExecutionBoundOperationAuthority,
    ) -> Self {
        let attempt_identity = WorthQueryExecutionAttemptIdentity::mint();
        let provider_session =
            WorthQueryExecutionProviderSession::mint(&attempt_identity, binding_authority);
        reserved.resources_mut().record_provider_session_mint();
        let evidence = WorthQueryExecutionResourceAttemptEvidence::capture(
            reserved.resources().operation(),
            &provider_session,
        );
        Self {
            reserved,
            attempt_identity,
            provider_session,
            evidence,
            artifact_run: Mutex::new(WorthQueryWorkflowArtifactRunState {
                next_generation: 1,
                active_registry: None,
            }),
        }
    }

    pub fn resources(&self) -> &WorthQueryAdmittedWorkflowResourcePlan {
        self.reserved.resources()
    }

    pub fn operation_resources(&self) -> &WorthQueryAdmittedExecutionResourcePlan {
        self.reserved.resources().operation()
    }

    pub(in crate::domain_computation) fn provider_session_for_managed_run(
        &self,
        _owner: &crate::domain_computation::managed_run::WorthQueryWorkflowRunTransitionPermit,
    ) -> &WorthQueryExecutionProviderSession {
        &self.provider_session
    }

    pub fn evidence(&self) -> &WorthQueryExecutionResourceAttemptEvidence {
        &self.evidence
    }

    pub fn attempt_identity(&self) -> &WorthQueryExecutionAttemptIdentity {
        &self.attempt_identity
    }

    pub(crate) fn retained_capacity_reservation_count(&self) -> usize {
        self.reserved.reservation_count()
    }

    #[doc(hidden)]
    pub fn begin_managed_workflow_artifacts(
        &self,
    ) -> Result<
        crate::domain_computation::artifact_owner::WorthQueryWorkflowArtifactAuthority,
        crate::domain_computation::artifact_owner::WorthQueryArtifactDenial,
    > {
        self.bind_workflow_artifacts_owned()
    }

    pub(in crate::domain_computation) fn bind_workflow_artifacts_for_managed_run(
        &self,
        _owner: &crate::domain_computation::managed_run::WorthQueryWorkflowRunTransitionPermit,
    ) -> Result<
        crate::domain_computation::artifact_owner::WorthQueryWorkflowArtifactAuthority,
        crate::domain_computation::artifact_owner::WorthQueryArtifactDenial,
    > {
        self.bind_workflow_artifacts_owned()
    }

    fn bind_workflow_artifacts_owned(
        &self,
    ) -> Result<
        crate::domain_computation::artifact_owner::WorthQueryWorkflowArtifactAuthority,
        crate::domain_computation::artifact_owner::WorthQueryArtifactDenial,
    > {
        let mut run = self
            .artifact_run
            .lock()
            .expect("workflow artifact run lock must remain available");
        if run
            .active_registry
            .as_ref()
            .and_then(Weak::upgrade)
            .is_some_and(|registry| !registry.is_closed())
        {
            return Err(
                crate::domain_computation::artifact_owner::WorthQueryArtifactDenial::new(
                    crate::domain_computation::artifact_owner::WorthQueryArtifactDenialKind::ActiveWorkflowRun,
                    None,
                    "execution resource attempt already owns an active workflow artifact run",
                ),
            );
        }
        let run_generation = run.next_generation;
        run.next_generation = run.next_generation.checked_add(1).ok_or_else(|| {
            crate::domain_computation::artifact_owner::WorthQueryArtifactDenial::new(
                crate::domain_computation::artifact_owner::WorthQueryArtifactDenialKind::StaleLifecycleGeneration,
                None,
                "workflow artifact run generation is exhausted",
            )
        })?;
        let authority =
            crate::domain_computation::artifact_owner::WorthQueryWorkflowArtifactAuthority::mint(
                self.provider_session.retain_binding_authority(),
                self.provider_session.identity(),
                run_generation,
            )?;
        run.active_registry = Some(Arc::downgrade(&authority.registry()));
        Ok(authority)
    }

    pub(in crate::domain_computation) fn stage_resources_and_evidence_for_managed_run(
        &self,
        stage_identity: &str,
        _owner: &crate::domain_computation::managed_run::WorthQueryWorkflowRunTransitionPermit,
    ) -> Option<(
        Arc<WorthQueryAdmittedExecutionResourcePlan>,
        WorthQueryExecutionResourceAttemptEvidence,
    )> {
        self.stage_resources_and_evidence_owned(stage_identity)
    }

    fn stage_resources_and_evidence_owned(
        &self,
        stage_identity: &str,
    ) -> Option<(
        Arc<WorthQueryAdmittedExecutionResourcePlan>,
        WorthQueryExecutionResourceAttemptEvidence,
    )> {
        let resources = self.reserved.resources().shared_stage(stage_identity)?;
        let evidence =
            WorthQueryExecutionResourceAttemptEvidence::capture(&resources, &self.provider_session);
        Some((resources, evidence))
    }

    pub(crate) fn binding_authority(
        &self,
    ) -> &crate::domain_computation::operation_binding::WorthQueryExecutionBoundOperationAuthority
    {
        self.provider_session.binding_authority()
    }

    pub(crate) fn release(self) -> WorthQueryWorkflowExecutionAttemptReleaseReceipt {
        let provider_session_identity = self.provider_session.identity().to_owned();
        drop(self.provider_session);
        WorthQueryWorkflowExecutionAttemptReleaseReceipt {
            provider_session_identity,
            capacity: self.reserved.release(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryWorkflowExecutionAttemptReleaseReceipt {
    provider_session_identity: String,
    capacity: WorthQueryExecutionCapacityReleaseReceipt,
}

impl WorthQueryWorkflowExecutionAttemptReleaseReceipt {
    pub fn provider_session_identity(&self) -> &str {
        &self.provider_session_identity
    }

    pub fn capacity(&self) -> &WorthQueryExecutionCapacityReleaseReceipt {
        &self.capacity
    }
}
