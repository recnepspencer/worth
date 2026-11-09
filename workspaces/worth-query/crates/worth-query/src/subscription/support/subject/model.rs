use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::super::super::activation::SubscriptionActivationInput;
use super::super::super::active_lane::ActiveSubscriptionLaneAdmission;
use super::super::super::admission::QuerySubscriptionAdmissionArtifact;
use super::super::super::closeout::SubscriptionLifecycleCloseout;
use super::super::super::continuation::SubscriptionContinuationReport;
use super::super::super::declaration::QuerySubscriptionDeclarationArtifact;
use super::super::super::evidence_identities::support_subject_identity;
use super::super::super::family::QuerySubscriptionFamily;
use super::super::super::future_selection::QuerySubscriptionFutureSelection;
use super::capability::QuerySubscriptionSupportClass;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionSupportSubject {
    support_class: QuerySubscriptionSupportClass,
    family: QuerySubscriptionFamily,
    future_selection: QuerySubscriptionFutureSelection,
    declaration_identity: WorthQueryEvidenceIdentity,
    admission_identity: Option<WorthQueryEvidenceIdentity>,
    source_identity: WorthQueryEvidenceIdentity,
    subject_identity: WorthQueryEvidenceIdentity,
}

impl QuerySubscriptionSupportSubject {
    pub fn declaration(
        declaration: &QuerySubscriptionDeclarationArtifact,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            QuerySubscriptionSupportClass::Declaration,
            declaration.family().clone(),
            declaration.future_selection().clone(),
            declaration.declaration_identity(),
            None,
            declaration.declaration_identity(),
        )?)
    }

    pub fn activation(
        declaration: &QuerySubscriptionDeclarationArtifact,
        activation: &SubscriptionActivationInput,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            QuerySubscriptionSupportClass::Activation,
            declaration.family().clone(),
            declaration.future_selection().clone(),
            declaration.declaration_identity(),
            Some(activation.admission_identity()),
            activation.evidence_identity(),
        )?)
    }

    pub fn active_lifecycle(
        declaration: &QuerySubscriptionDeclarationArtifact,
        admission: &QuerySubscriptionAdmissionArtifact,
        active_admission: &ActiveSubscriptionLaneAdmission,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            QuerySubscriptionSupportClass::ActiveLifecycle,
            declaration.family().clone(),
            declaration.future_selection().clone(),
            declaration.declaration_identity(),
            Some(admission.evidence_identity()),
            active_admission.lane_digest().evidence_identity(),
        )?)
    }

    pub fn continuation(
        declaration: &QuerySubscriptionDeclarationArtifact,
        admission: &QuerySubscriptionAdmissionArtifact,
        continuation: &SubscriptionContinuationReport,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            QuerySubscriptionSupportClass::Continuation,
            declaration.family().clone(),
            declaration.future_selection().clone(),
            declaration.declaration_identity(),
            Some(admission.evidence_identity()),
            continuation.evidence_identity(),
        )?)
    }

    pub fn preview_closeout(
        declaration: &QuerySubscriptionDeclarationArtifact,
        admission: &QuerySubscriptionAdmissionArtifact,
        closeout: &SubscriptionLifecycleCloseout,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            QuerySubscriptionSupportClass::PreviewCloseout,
            declaration.family().clone(),
            closeout.future_selection().clone(),
            declaration.declaration_identity(),
            Some(admission.evidence_identity()),
            closeout.evidence_identity(),
        )?)
    }

    pub fn durable_replay(
        declaration: &QuerySubscriptionDeclarationArtifact,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            QuerySubscriptionSupportClass::DurableReplay,
            declaration.family().clone(),
            declaration.future_selection().clone(),
            declaration.declaration_identity(),
            None,
            declaration.declaration_identity(),
        )?)
    }

    pub fn store_backed_restart(
        declaration: &QuerySubscriptionDeclarationArtifact,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            QuerySubscriptionSupportClass::StoreBackedRestart,
            declaration.family().clone(),
            declaration.future_selection().clone(),
            declaration.declaration_identity(),
            None,
            declaration.declaration_identity(),
        )?)
    }

    fn new(
        support_class: QuerySubscriptionSupportClass,
        family: QuerySubscriptionFamily,
        future_selection: QuerySubscriptionFutureSelection,
        declaration_identity: &WorthQueryEvidenceIdentity,
        admission_identity: Option<&WorthQueryEvidenceIdentity>,
        source_identity: &WorthQueryEvidenceIdentity,
    ) -> Result<Self, IdentityDerivationDenial> {
        let subject_identity = support_subject_identity(
            support_class.as_str(),
            &family,
            future_selection.projection_identity(),
            declaration_identity,
            admission_identity,
            source_identity,
        )?;
        Ok(Self {
            support_class,
            family,
            future_selection,
            declaration_identity: declaration_identity.clone(),
            admission_identity: admission_identity.cloned(),
            source_identity: source_identity.clone(),
            subject_identity,
        })
    }

    pub fn support_class(&self) -> &QuerySubscriptionSupportClass {
        &self.support_class
    }

    pub fn family(&self) -> &QuerySubscriptionFamily {
        &self.family
    }

    pub fn declaration_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.declaration_identity
    }

    pub fn future_selection(&self) -> &QuerySubscriptionFutureSelection {
        &self.future_selection
    }

    pub fn admission_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        self.admission_identity.as_ref()
    }

    pub fn source_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.source_identity
    }

    pub fn subject_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.subject_identity
    }
}
