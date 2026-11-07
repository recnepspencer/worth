use super::*;

pub(super) struct WorthQueryRuntimeDownstreamDeliveryParts {
    pub(super) view_name: String,
    pub(super) delivery_batch_identity: WorthQueryEvidenceIdentity,
    pub(super) delivery_class: WorthQueryRuntimeDownstreamDeliveryClass,
    pub(super) delivery_cause_kind: QuerySubscriptionDeliveryCauseKind,
    pub(super) delivery_cause_identity: WorthQueryEvidenceIdentity,
    pub(super) sequence: u64,
    pub(super) basis_identity: WorthQueryEvidenceIdentity,
    pub(super) support_posture: WorthQueryRuntimeDownstreamSupportPosture,
    pub(super) support_identity: WorthQueryEvidenceIdentity,
    pub(super) mixed_cause_identity: Option<WorthQueryEvidenceIdentity>,
    pub(super) async_result_state_identity: Option<WorthQueryEvidenceIdentity>,
    pub(super) remask_identity: Option<WorthQueryEvidenceIdentity>,
    pub(super) runtime_resume_support_posture: WorthQueryLowerRuntimeSupportPosture,
    pub(super) runtime_resume_support_identity: WorthQueryEvidenceIdentity,
    pub(super) durable_resume_support_posture: WorthQueryLowerRuntimeSupportPosture,
    pub(super) durable_resume_support_identity: WorthQueryEvidenceIdentity,
    pub(super) delivery_identity: WorthQueryEvidenceIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRuntimeDownstreamDelivery {
    view_name: String,
    delivery_batch_identity: WorthQueryEvidenceIdentity,
    delivery_class: WorthQueryRuntimeDownstreamDeliveryClass,
    delivery_cause_kind: QuerySubscriptionDeliveryCauseKind,
    delivery_cause_identity: WorthQueryEvidenceIdentity,
    sequence: u64,
    basis_identity: WorthQueryEvidenceIdentity,
    support_posture: WorthQueryRuntimeDownstreamSupportPosture,
    support_identity: WorthQueryEvidenceIdentity,
    mixed_cause_identity: Option<WorthQueryEvidenceIdentity>,
    async_result_state_identity: Option<WorthQueryEvidenceIdentity>,
    remask_identity: Option<WorthQueryEvidenceIdentity>,
    runtime_resume_support_posture: WorthQueryLowerRuntimeSupportPosture,
    runtime_resume_support_identity: WorthQueryEvidenceIdentity,
    durable_resume_support_posture: WorthQueryLowerRuntimeSupportPosture,
    durable_resume_support_identity: WorthQueryEvidenceIdentity,
    delivery_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryRuntimeDownstreamDelivery {
    pub(super) fn from_projection(parts: WorthQueryRuntimeDownstreamDeliveryParts) -> Self {
        Self {
            view_name: parts.view_name,
            delivery_batch_identity: parts.delivery_batch_identity,
            delivery_class: parts.delivery_class,
            delivery_cause_kind: parts.delivery_cause_kind,
            delivery_cause_identity: parts.delivery_cause_identity,
            sequence: parts.sequence,
            basis_identity: parts.basis_identity,
            support_posture: parts.support_posture,
            support_identity: parts.support_identity,
            mixed_cause_identity: parts.mixed_cause_identity,
            async_result_state_identity: parts.async_result_state_identity,
            remask_identity: parts.remask_identity,
            runtime_resume_support_posture: parts.runtime_resume_support_posture,
            runtime_resume_support_identity: parts.runtime_resume_support_identity,
            durable_resume_support_posture: parts.durable_resume_support_posture,
            durable_resume_support_identity: parts.durable_resume_support_identity,
            delivery_identity: parts.delivery_identity,
        }
    }

    pub fn view_name(&self) -> &str {
        &self.view_name
    }

    pub fn delivery_batch_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.delivery_batch_identity
    }

    pub fn delivery_batch_for_reporting(&self) -> &str {
        self.delivery_batch_identity.as_str()
    }

    pub fn delivery_class(&self) -> WorthQueryRuntimeDownstreamDeliveryClass {
        self.delivery_class
    }

    pub fn delivery_cause_kind(&self) -> QuerySubscriptionDeliveryCauseKind {
        self.delivery_cause_kind
    }

    pub fn delivery_cause_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.delivery_cause_identity
    }

    pub fn delivery_cause_for_reporting(&self) -> &str {
        self.delivery_cause_identity.as_str()
    }

    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    pub fn basis_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.basis_identity
    }

    pub fn basis_for_reporting(&self) -> &str {
        self.basis_identity.as_str()
    }

    pub fn support_posture(&self) -> WorthQueryRuntimeDownstreamSupportPosture {
        self.support_posture
    }

    pub fn support_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.support_identity
    }

    pub fn support_for_reporting(&self) -> &str {
        self.support_identity.as_str()
    }

    pub fn mixed_cause_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        self.mixed_cause_identity.as_ref()
    }

    pub fn mixed_cause_for_reporting(&self) -> Option<&str> {
        self.mixed_cause_identity
            .as_ref()
            .map(WorthQueryEvidenceIdentity::as_str)
    }

    pub fn async_result_state_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        self.async_result_state_identity.as_ref()
    }

    pub fn async_result_state_for_reporting(&self) -> Option<&str> {
        self.async_result_state_identity
            .as_ref()
            .map(WorthQueryEvidenceIdentity::as_str)
    }

    pub fn remask_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        self.remask_identity.as_ref()
    }

    pub fn remask_for_reporting(&self) -> Option<&str> {
        self.remask_identity
            .as_ref()
            .map(WorthQueryEvidenceIdentity::as_str)
    }

    pub fn delivery_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.delivery_identity
    }

    pub fn delivery_for_reporting(&self) -> &str {
        self.delivery_identity.as_str()
    }

    pub fn runtime_resume_support_posture(&self) -> WorthQueryLowerRuntimeSupportPosture {
        self.runtime_resume_support_posture
    }

    pub fn durable_resume_support_posture(&self) -> WorthQueryLowerRuntimeSupportPosture {
        self.durable_resume_support_posture
    }

    pub fn negotiate_runtime_resume(
        &self,
        basis_identity: Option<&WorthQueryEvidenceIdentity>,
    ) -> WorthQueryRuntimeDownstreamResumePosture {
        let support_posture = self.runtime_resume_support_posture;
        if support_posture != WorthQueryLowerRuntimeSupportPosture::Admitted {
            return WorthQueryRuntimeDownstreamResumePosture::new(
                support_gate_resume_kind(support_posture),
                Some(self.basis_identity.clone()),
                support_posture,
                self.runtime_resume_support_identity.clone(),
            );
        }
        match basis_identity {
            Some(candidate) if candidate == &self.basis_identity => {
                WorthQueryRuntimeDownstreamResumePosture::new(
                    WorthQueryRuntimeDownstreamResumePostureKind::RuntimeBackedAdmitted,
                    Some(self.basis_identity.clone()),
                    support_posture,
                    self.runtime_resume_support_identity.clone(),
                )
            }
            Some(_) => WorthQueryRuntimeDownstreamResumePosture::new(
                WorthQueryRuntimeDownstreamResumePostureKind::StaleBasisDenied,
                Some(self.basis_identity.clone()),
                support_posture,
                self.runtime_resume_support_identity.clone(),
            ),
            None => WorthQueryRuntimeDownstreamResumePosture::new(
                WorthQueryRuntimeDownstreamResumePostureKind::MissingBasisDenied,
                Some(self.basis_identity.clone()),
                support_posture,
                self.runtime_resume_support_identity.clone(),
            ),
        }
    }

    pub fn durable_resume_posture(&self) -> WorthQueryRuntimeDownstreamResumePosture {
        let support_posture = self.durable_resume_support_posture;
        let kind = match support_posture {
            WorthQueryLowerRuntimeSupportPosture::Deferred => {
                WorthQueryRuntimeDownstreamResumePostureKind::DurableDeferredDebt
            }
            other => support_gate_resume_kind(other),
        };
        WorthQueryRuntimeDownstreamResumePosture::new(
            kind,
            Some(self.basis_identity.clone()),
            support_posture,
            self.durable_resume_support_identity.clone(),
        )
    }
}
