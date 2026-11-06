use std::cell::RefCell;
use std::path::Path;
use worth_foundational::CanonicalDigestId;

mod construction;
mod process_crash;

pub use crate::operational_recovery_yieldpoint::OperationalRecoveryYieldpoint;
use crate::OperationalRecoveryProcessCrashConfig;
use worth_store_authority::PrimaryServingAuthority;
use worth_store_offline_verifier::{
    ForensicAcquisitionCounters, ForensicAcquisitionDenial, ForensicAcquisitionProgress,
    ForensicAcquisitionSession, ForensicBundle, OperationalTruthReport,
    ReplicaTargetVerificationBudget,
};
use worth_store_operations::{
    CompletedReplicaBootstrap, CurrentReplicaPromotion, DurablyFencedReplicaPromotion,
    ExecutedReplicaBootstrap, ExecutedReplicaPromotion, ExecutionReadyReplicaBootstrap,
    ExecutionReadyReplicaPromotion, FencedReplicaPromotion, OperationalControlStorePort,
    OperationalOperationId, OperationalTransitionId, PostVerifiedReplicaBootstrap,
    PostVerifiedReplicaPromotion, PublishedReplicaPromotion, ReplicaBootstrapExecutionDenial,
    ReplicaBootstrapFinalizationDenial, ReplicaBootstrapPersistenceDenial,
    ReplicaPromotionExecutionDenial, ReplicaPromotionFencePersistenceDenial,
    ReplicaPromotionFencingDenial, ReplicaPromotionFinalizationDenial,
    ReplicaPromotionPublicationPort, TransferredReplicaBootstrap,
};
use worth_store_replication::ReplicaBootstrapExecutionPort;

#[derive(Debug)]
pub enum DrivenOperationalTransition<T> {
    InterruptedBefore,
    Completed(T),
    InterruptedAfter(T),
}

#[derive(Debug)]
pub struct OperationalRecoveryProductionDriver {
    state: RefCell<OperationalRecoveryDriverState>,
}

#[derive(Debug)]
struct OperationalRecoveryDriverState {
    pause_at: Option<OperationalRecoveryYieldpoint>,
    process_crash: Option<OperationalRecoveryProcessCrashConfig>,
    reached: Vec<OperationalRecoveryYieldpoint>,
    operation_identities: Vec<String>,
    control_artifact_identities: Vec<CanonicalDigestId>,
    inspection_evidence_identity: Option<CanonicalDigestId>,
    truth_evidence_identity: Option<CanonicalDigestId>,
    latest_control_observation:
        Option<worth_store_operations::OperationalControlSessionObservation>,
}

impl OperationalRecoveryProductionDriver {
    /// Binds a durable production transition to this scenario invocation.
    /// The driver accepts only the owner-produced control artifact and derives
    /// its identity itself; callers cannot supply a phase label or digest.
    pub(super) fn observe_durable_control_transition(
        &self,
        record: &worth_store_operations::OperationalControlRecord,
    ) {
        self.observe_operation(record.operation_id());
        let identity = CanonicalDigestId::new(record.stable_fingerprint());
        let mut state = self.state.borrow_mut();
        if !state.control_artifact_identities.contains(&identity) {
            state.control_artifact_identities.push(identity);
            state.control_artifact_identities.sort_unstable();
        }
    }

    /// Binds the completed bounded media walk and canonical semantic truth to
    /// the invocation without accepting caller-authored evidence bytes.
    pub fn observe_completed_truth_composition(
        &self,
        operation: &OperationalOperationId,
        truth: &OperationalTruthReport,
    ) {
        self.observe_operation(operation);
        let mut state = self.state.borrow_mut();
        state.inspection_evidence_identity =
            Some(CanonicalDigestId::new(truth.source_inspection_identity()));
        state.truth_evidence_identity =
            Some(CanonicalDigestId::new(truth.truth_evidence_identity()));
    }

    pub fn forensic_acquire_next(
        &self,
        operation: &OperationalOperationId,
        session: &mut ForensicAcquisitionSession,
    ) -> Result<DrivenOperationalTransition<ForensicAcquisitionProgress>, ForensicAcquisitionDenial>
    {
        self.observe_operation(operation);
        if self.before(OperationalRecoveryYieldpoint::BeforeForensicSourceAcquisition) {
            return Ok(DrivenOperationalTransition::InterruptedBefore);
        }
        let progress = session.acquire_next()?;
        Ok(self.after(
            OperationalRecoveryYieldpoint::AfterForensicSourceRecord,
            progress,
        ))
    }

    pub fn forensic_finish(
        &self,
        session: ForensicAcquisitionSession,
        completed_at_tick: u64,
    ) -> Result<
        DrivenOperationalTransition<(ForensicBundle, ForensicAcquisitionCounters)>,
        ForensicAcquisitionDenial,
    > {
        if self.before(OperationalRecoveryYieldpoint::BeforeForensicFinalization) {
            return Ok(DrivenOperationalTransition::InterruptedBefore);
        }
        let finished = session.finish(completed_at_tick)?;
        Ok(self.after(
            OperationalRecoveryYieldpoint::AfterForensicFinalization,
            finished,
        ))
    }

    pub fn bootstrap_transfer<'control>(
        &self,
        ready: ExecutionReadyReplicaBootstrap<'control>,
        port: &mut impl ReplicaBootstrapExecutionPort,
    ) -> Result<
        DrivenOperationalTransition<TransferredReplicaBootstrap<'control>>,
        ReplicaBootstrapExecutionDenial,
    > {
        self.observe_operation(ready.operation_id());
        if self.before(OperationalRecoveryYieldpoint::BeforeBootstrapTransfer) {
            return Ok(DrivenOperationalTransition::InterruptedBefore);
        }
        let transferred = ready.transfer(port)?;
        Ok(self.after(
            OperationalRecoveryYieldpoint::AfterBootstrapTransfer,
            transferred,
        ))
    }

    pub fn persist_bootstrap_transfer(
        &self,
        transferred: &TransferredReplicaBootstrap<'_>,
        transition: OperationalTransitionId,
    ) -> Result<
        DrivenOperationalTransition<ExecutedReplicaBootstrap>,
        ReplicaBootstrapPersistenceDenial,
    > {
        if self.before(OperationalRecoveryYieldpoint::BeforeBootstrapControlRecord) {
            return Ok(DrivenOperationalTransition::InterruptedBefore);
        }
        let executed = transferred.persist(transition)?;
        Ok(self.after(
            OperationalRecoveryYieldpoint::AfterBootstrapControlRecord,
            executed,
        ))
    }

    pub fn post_verify_bootstrap(
        &self,
        executed: ExecutedReplicaBootstrap,
        target_root: &Path,
        budget: ReplicaTargetVerificationBudget,
    ) -> Result<
        DrivenOperationalTransition<PostVerifiedReplicaBootstrap>,
        ReplicaBootstrapFinalizationDenial,
    > {
        if self.before(OperationalRecoveryYieldpoint::BeforeBootstrapPostVerification) {
            return Ok(DrivenOperationalTransition::InterruptedBefore);
        }
        let verified = executed.post_verify(target_root, budget)?;
        Ok(self.after(
            OperationalRecoveryYieldpoint::AfterBootstrapPostVerification,
            verified,
        ))
    }

    pub fn complete_bootstrap(
        &self,
        verified: PostVerifiedReplicaBootstrap,
        control: &dyn OperationalControlStorePort,
        transition: OperationalTransitionId,
    ) -> Result<
        DrivenOperationalTransition<CompletedReplicaBootstrap>,
        ReplicaBootstrapFinalizationDenial,
    > {
        if self.before(OperationalRecoveryYieldpoint::BeforeBootstrapCompletion) {
            return Ok(DrivenOperationalTransition::InterruptedBefore);
        }
        let completed = verified.complete_with_certification_control_store(control, transition)?;
        Ok(self.after(
            OperationalRecoveryYieldpoint::AfterBootstrapCompletion,
            completed,
        ))
    }

    pub fn promotion_fence<'control>(
        &self,
        ready: ExecutionReadyReplicaPromotion<'control>,
        authority: &PrimaryServingAuthority<'_>,
    ) -> Result<
        DrivenOperationalTransition<FencedReplicaPromotion<'control>>,
        ReplicaPromotionFencingDenial,
    > {
        self.observe_operation(ready.operation_id());
        if self.before(OperationalRecoveryYieldpoint::BeforePromotionExternalFence) {
            return Ok(DrivenOperationalTransition::InterruptedBefore);
        }
        let fenced = ready.fence(authority)?;
        Ok(self.after(
            OperationalRecoveryYieldpoint::AfterPromotionExternalFence,
            fenced,
        ))
    }

    pub fn persist_promotion_fence<'control>(
        &self,
        fenced: &FencedReplicaPromotion<'control>,
        transition: OperationalTransitionId,
    ) -> Result<
        DrivenOperationalTransition<DurablyFencedReplicaPromotion<'control>>,
        ReplicaPromotionFencePersistenceDenial,
    > {
        if self.before(OperationalRecoveryYieldpoint::BeforePromotionFenceRecord) {
            return Ok(DrivenOperationalTransition::InterruptedBefore);
        }
        let durable = fenced.persist_fence(transition)?;
        Ok(self.after(
            OperationalRecoveryYieldpoint::AfterPromotionFenceRecord,
            durable,
        ))
    }

    pub fn record_promotion(
        &self,
        durable: &DurablyFencedReplicaPromotion<'_>,
        transition: OperationalTransitionId,
    ) -> Result<
        DrivenOperationalTransition<ExecutedReplicaPromotion>,
        ReplicaPromotionExecutionDenial,
    > {
        if self.before(OperationalRecoveryYieldpoint::BeforePromotionRecord) {
            return Ok(DrivenOperationalTransition::InterruptedBefore);
        }
        let executed = durable.promote(transition)?;
        Ok(self.after(
            OperationalRecoveryYieldpoint::AfterPromotionRecord,
            executed,
        ))
    }

    pub fn post_verify_promotion(
        &self,
        executed: ExecutedReplicaPromotion,
        target_root: &Path,
        budget: ReplicaTargetVerificationBudget,
    ) -> Result<
        DrivenOperationalTransition<PostVerifiedReplicaPromotion>,
        ReplicaPromotionFinalizationDenial,
    > {
        if self.before(OperationalRecoveryYieldpoint::BeforePromotionPostVerification) {
            return Ok(DrivenOperationalTransition::InterruptedBefore);
        }
        let verified = executed.post_verify(target_root, budget)?;
        Ok(self.after(
            OperationalRecoveryYieldpoint::AfterPromotionPostVerification,
            verified,
        ))
    }

    pub fn publish_promotion(
        &self,
        verified: PostVerifiedReplicaPromotion,
        control: &dyn OperationalControlStorePort,
        transition: OperationalTransitionId,
        port: &mut impl ReplicaPromotionPublicationPort,
    ) -> Result<
        DrivenOperationalTransition<PublishedReplicaPromotion>,
        ReplicaPromotionFinalizationDenial,
    > {
        if self.before(OperationalRecoveryYieldpoint::BeforePromotionPublication) {
            return Ok(DrivenOperationalTransition::InterruptedBefore);
        }
        let published =
            verified.publish_with_certification_control_store(control, transition, port)?;
        Ok(self.after(
            OperationalRecoveryYieldpoint::AfterPromotionPublication,
            published,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn readmit_promotion(
        &self,
        published: PublishedReplicaPromotion,
        control: &dyn OperationalControlStorePort,
        transition: OperationalTransitionId,
        serving: &PrimaryServingAuthority<'_>,
        now_tick: u64,
        requested_until_tick: u64,
    ) -> Result<
        DrivenOperationalTransition<CurrentReplicaPromotion>,
        ReplicaPromotionFinalizationDenial,
    > {
        if self.before(OperationalRecoveryYieldpoint::BeforePromotionReadmission) {
            return Ok(DrivenOperationalTransition::InterruptedBefore);
        }
        let current = published.readmit_with_certification_control_store(
            control,
            transition,
            serving,
            now_tick,
            requested_until_tick,
        )?;
        Ok(self.after(
            OperationalRecoveryYieldpoint::AfterPromotionReadmission,
            current,
        ))
    }

    pub(super) fn before(&self, point: OperationalRecoveryYieldpoint) -> bool {
        let paused = self.record_yieldpoint(point);
        self.crash_at_latest_control_if_scheduled(point);
        paused
    }

    pub(super) fn after<T>(
        &self,
        point: OperationalRecoveryYieldpoint,
        value: T,
    ) -> DrivenOperationalTransition<T> {
        let paused = self.record_yieldpoint(point);
        self.crash_at_latest_control_if_scheduled(point);
        if paused {
            DrivenOperationalTransition::InterruptedAfter(value)
        } else {
            DrivenOperationalTransition::Completed(value)
        }
    }

    pub(super) fn record_control_yieldpoint(&self, point: OperationalRecoveryYieldpoint) -> bool {
        self.record_yieldpoint(point)
    }

    fn record_yieldpoint(&self, point: OperationalRecoveryYieldpoint) -> bool {
        let mut state = self.state.borrow_mut();
        state.reached.push(point);
        take_pause(&mut state, point)
    }

    pub(super) fn observe_control_session(
        &self,
        observation: worth_store_operations::OperationalControlSessionObservation,
    ) {
        self.state.borrow_mut().latest_control_observation = Some(observation);
    }

    pub(super) fn observe_operation(&self, operation: &OperationalOperationId) {
        let identity = operation.as_str();
        let mut state = self.state.borrow_mut();
        if !state
            .operation_identities
            .iter()
            .any(|observed| observed == identity)
        {
            state.operation_identities.push(identity.to_owned());
        }
    }
}

fn take_pause(
    state: &mut OperationalRecoveryDriverState,
    point: OperationalRecoveryYieldpoint,
) -> bool {
    if state.pause_at == Some(point) {
        state.pause_at = None;
        true
    } else {
        false
    }
}
