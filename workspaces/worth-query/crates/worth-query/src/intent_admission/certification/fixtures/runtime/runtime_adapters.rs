use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::facade::foundation::{WorthQueryMutationReceipt, WorthQueryWorkspaceError};
use crate::facade::runtime::{
    runtime_subscription_support_evidence_identity, SignalInvalidationBoundaryReceipt,
    SubscriptionActivationBoundaryReceipt, SubscriptionActivationInput,
    WorthQueryBasisAdmissionEvidenceRow, WorthQueryEffectPolicy, WorthQueryPreviewBasisAdmission,
    WorthQueryRuntimeEvidenceAuthority, WorthQueryRuntimeInspectionEvidence,
    WorthQueryRuntimeInspectorEvidenceAdapter, WorthQueryRuntimePreviewBasisAdapter,
    WorthQueryRuntimeSignalSinkAdapter, WorthQueryRuntimeSubscriptionActivationAdapter,
    WorthQuerySessionLabel, WorthQueryWriteReceipt,
};

pub(super) struct CertificationSignalSink;

impl WorthQueryRuntimeSignalSinkAdapter for CertificationSignalSink {
    fn route_write_receipt(
        &mut self,
        receipt: &WorthQueryMutationReceipt,
    ) -> Result<SignalInvalidationBoundaryReceipt, WorthQueryWorkspaceError> {
        let routed = self.build_signal_invalidation_routing_receipt(receipt)?;
        self.build_signal_invalidation_boundary_receipt(receipt, routed)
    }
}

pub(super) struct CertificationSubscriptionActivation;

impl WorthQueryRuntimeSubscriptionActivationAdapter for CertificationSubscriptionActivation {
    fn support_evidence_identity(&self) -> WorthQueryEvidenceIdentity {
        runtime_subscription_support_evidence_identity("certification-subscription-activation")
            .expect("fixed certification activation support literal is within digest bounds")
    }

    fn admit_activation(
        &mut self,
        view_name: &str,
        activation: &SubscriptionActivationInput,
    ) -> Result<SubscriptionActivationBoundaryReceipt, WorthQueryWorkspaceError> {
        let receipt = self.build_subscription_activation_receipt(view_name, activation)?;
        Ok(self.build_subscription_activation_boundary_receipt(view_name, activation, receipt)?)
    }
}

pub(super) struct CertificationPreviewBasis;

impl WorthQueryRuntimePreviewBasisAdapter for CertificationPreviewBasis {
    fn admit_preview_basis(
        &self,
        label: &WorthQuerySessionLabel,
        effect_policy: WorthQueryEffectPolicy,
        authority: &WorthQueryRuntimeEvidenceAuthority,
    ) -> Result<WorthQueryPreviewBasisAdmission, WorthQueryWorkspaceError> {
        Ok(WorthQueryPreviewBasisAdmission::new(
            authority,
            label.clone(),
            effect_policy,
            WorthQueryBasisAdmissionEvidenceRow::rows_from_values(["certification-preview-basis"])?,
        )?)
    }
}

pub(super) struct CertificationInspectorEvidence;

impl WorthQueryRuntimeInspectorEvidenceAdapter for CertificationInspectorEvidence {
    fn inspect_write_receipt(
        &self,
        receipt: &WorthQueryWriteReceipt,
        authority: &WorthQueryRuntimeEvidenceAuthority,
    ) -> Result<WorthQueryRuntimeInspectionEvidence, WorthQueryWorkspaceError> {
        Ok(WorthQueryRuntimeInspectionEvidence::new(
            authority,
            "certification-write-receipt",
            receipt.authority_lane(),
            ["certification-inspector-evidence"],
        ))
    }
}
