use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::memory_workspace::{WorthQueryMutationReceipt, WorthQueryWorkspaceError};
use crate::program::WorthQueryDerivedView;
use crate::session_label::WorthQuerySessionLabel;
use crate::subscription::SubscriptionActivationInput;

use super::{
    SignalInvalidationBoundaryReceipt, SignalInvalidationRoutingReceipt,
    SubscriptionActivationBoundaryReceipt, SubscriptionActivationReceipt,
};
use crate::runtime::remask_posture::WorthQueryRuntimeRemaskProjection;
use crate::runtime::{
    WorthQueryEffectPolicy, WorthQueryPreviewBasisAdmission, WorthQueryRuntimeEvidenceAuthority,
    WorthQueryRuntimeInspectionEvidence, WorthQueryWriteReceipt,
};
use crate::WorthQueryEvidenceIdentity;

pub trait WorthQueryRuntimeSignalSinkAdapter {
    fn build_signal_invalidation_routing_receipt(
        &self,
        receipt: &WorthQueryMutationReceipt,
    ) -> Result<SignalInvalidationRoutingReceipt, WorthQueryWorkspaceError> {
        SignalInvalidationRoutingReceipt::from_mutation_receipt(receipt)
    }

    fn build_signal_invalidation_boundary_receipt(
        &self,
        receipt: &WorthQueryMutationReceipt,
        routing_receipt: SignalInvalidationRoutingReceipt,
    ) -> Result<SignalInvalidationBoundaryReceipt, WorthQueryWorkspaceError> {
        Ok(SignalInvalidationBoundaryReceipt::from_mutation_receipt(
            receipt,
            routing_receipt,
        )?)
    }

    fn route_write_receipt(
        &mut self,
        receipt: &WorthQueryMutationReceipt,
    ) -> Result<SignalInvalidationBoundaryReceipt, WorthQueryWorkspaceError>;

    fn route_write_batch(
        &mut self,
        receipts: &[WorthQueryMutationReceipt],
    ) -> Result<Vec<SignalInvalidationBoundaryReceipt>, WorthQueryWorkspaceError> {
        receipts
            .iter()
            .map(|receipt| self.route_write_receipt(receipt))
            .collect()
    }
}

pub trait WorthQueryRuntimeSubscriptionActivationAdapter {
    fn support_evidence_identity(&self) -> WorthQueryEvidenceIdentity;

    fn support_evidence_for_reporting(&self) -> String {
        self.support_evidence_identity().as_str().to_string()
    }

    fn remask_projection(
        &self,
        _view_name: &str,
        _activation: &SubscriptionActivationInput,
    ) -> Option<WorthQueryRuntimeRemaskProjection> {
        None
    }

    fn build_subscription_activation_receipt(
        &self,
        view_name: &str,
        activation: &SubscriptionActivationInput,
    ) -> Result<SubscriptionActivationReceipt, IdentityDerivationDenial> {
        SubscriptionActivationReceipt::from_activation(
            view_name,
            activation,
            self.support_evidence_identity(),
            self.remask_projection(view_name, activation),
        )
    }

    fn build_subscription_activation_boundary_receipt(
        &self,
        view_name: &str,
        activation: &SubscriptionActivationInput,
        activation_receipt: SubscriptionActivationReceipt,
    ) -> Result<SubscriptionActivationBoundaryReceipt, IdentityDerivationDenial> {
        SubscriptionActivationBoundaryReceipt::from_activation(
            view_name,
            activation,
            activation_receipt,
        )
    }

    fn admit_activation(
        &mut self,
        view_name: &str,
        activation: &SubscriptionActivationInput,
    ) -> Result<SubscriptionActivationBoundaryReceipt, WorthQueryWorkspaceError>;
}

pub trait WorthQueryRuntimePreviewBasisAdapter {
    fn admit_preview_basis(
        &self,
        label: &WorthQuerySessionLabel,
        effect_policy: WorthQueryEffectPolicy,
        authority: &WorthQueryRuntimeEvidenceAuthority,
    ) -> Result<WorthQueryPreviewBasisAdmission, WorthQueryWorkspaceError>;
}

pub trait WorthQueryRuntimeInspectorEvidenceAdapter {
    fn inspect_write_receipt(
        &self,
        receipt: &WorthQueryWriteReceipt,
        authority: &WorthQueryRuntimeEvidenceAuthority,
    ) -> Result<WorthQueryRuntimeInspectionEvidence, WorthQueryWorkspaceError>;
}

pub trait WorthQueryRuntimeDeclarationInitializationAdapter {
    fn declaration_initialization_metadata(
        &self,
        view: &WorthQueryDerivedView,
    ) -> Result<crate::runtime::WorthQueryMutationMetadata, WorthQueryWorkspaceError>;
}
