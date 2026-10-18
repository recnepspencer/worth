use std::marker::PhantomData;
use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::error::{BridgeSpeculationError, BridgeSpeculationErrorKind};
use crate::identity::{
    BridgeIdentity, BridgeIdentityEvidence, PreviewExecutionRecordIdentityTag,
    PreviewSessionIdentityTag,
};

use super::contracts::BridgePromotionAdmissibilityProof;
use super::taxonomy::{
    BridgePreviewLifecycleStateKind, BridgePreviewTypestate, PreviewActive, PreviewAdmitted,
    PreviewDeclared, PreviewDiscarded, PreviewPromoted,
};
use super::validation::ValidatedBridgePreviewSessionDeclaration;

pub type BridgePreviewSessionIdentity = BridgeIdentity<PreviewSessionIdentityTag>;
pub type PreviewExecutionRecordIdentity = BridgeIdentity<PreviewExecutionRecordIdentityTag>;

impl BridgePreviewSessionIdentity {
    pub fn from_bridge_evidence(evidence_identity: &BridgeIdentityEvidence) -> Self {
        Self::admit_bridge_owned(format!(
            "bridge-preview-session:external-authority-evidence:{}",
            evidence_identity.as_str()
        ))
    }

    pub fn from_stable_name(value: impl Into<Arc<str>>) -> Self {
        Self::admit_bridge_owned(value)
    }
}

impl PreviewExecutionRecordIdentity {
    pub fn from_stable_name(value: impl Into<Arc<str>>) -> Self {
        Self::admit_bridge_owned(value)
    }

    pub fn from_bridge_evidence(evidence_identity: &BridgeIdentityEvidence) -> Self {
        Self::admit_bridge_owned(format!(
            "bridge-preview-execution-record:external-authority-evidence:{}",
            evidence_identity.as_str()
        ))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct PreviewSessionActivation {
    execution_record_identity: PreviewExecutionRecordIdentity,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl PreviewSessionActivation {
    pub(crate) fn new(execution_record_identity: PreviewExecutionRecordIdentity) -> Self {
        let canonical_basis = Arc::<str>::from(format!(
            "preview-session-activation|execution-record={}",
            execution_record_identity.as_str()
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            execution_record_identity,
            canonical_basis,
            digest: Arc::from(format!("preview-session-activation:sha256:{digest:x}")),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct BridgePreviewSession<State> {
    session_identity: BridgePreviewSessionIdentity,
    declaration: ValidatedBridgePreviewSessionDeclaration,
    execution_record_identity: Option<PreviewExecutionRecordIdentity>,
    lifecycle_state_kind: BridgePreviewLifecycleStateKind,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
    _state: PhantomData<State>,
}

impl BridgePreviewSession<PreviewDeclared> {
    pub(crate) fn declare(
        session_identity: BridgePreviewSessionIdentity,
        declaration: ValidatedBridgePreviewSessionDeclaration,
    ) -> Self {
        Self::new(session_identity, declaration, None)
    }

    pub(crate) fn admit(self) -> BridgePreviewSession<PreviewAdmitted> {
        BridgePreviewSession::new(self.session_identity, self.declaration, None)
    }
}

impl BridgePreviewSession<PreviewAdmitted> {
    pub(crate) fn activate(
        self,
        activation: PreviewSessionActivation,
    ) -> BridgePreviewSession<PreviewActive> {
        BridgePreviewSession::new(
            self.session_identity,
            self.declaration,
            Some(activation.execution_record_identity),
        )
    }
}

impl BridgePreviewSession<PreviewActive> {
    pub(crate) fn discard(self) -> BridgePreviewSession<PreviewDiscarded> {
        BridgePreviewSession::new(
            self.session_identity,
            self.declaration,
            self.execution_record_identity,
        )
    }

    pub fn promotion_admissibility_proof(&self) -> BridgePromotionAdmissibilityProof {
        BridgePromotionAdmissibilityProof::from_active_session(self)
    }

    pub(crate) fn promote(
        self,
        proof: &BridgePromotionAdmissibilityProof,
    ) -> Result<BridgePreviewSession<PreviewPromoted>, (Self, BridgeSpeculationError)> {
        if !proof.matches_active_session(&self) {
            let denial = BridgeSpeculationError::new(
                BridgeSpeculationErrorKind::PromotionAdmissibilityMismatch,
                format!(
                    "Promotion proof `{}` did not match active preview session `{}`.",
                    proof.proof_identity().as_str(),
                    self.session_identity.as_str(),
                ),
            );
            return Err((self, denial));
        }

        Ok(BridgePreviewSession::new(
            self.session_identity,
            self.declaration,
            self.execution_record_identity,
        ))
    }
}

impl<State> BridgePreviewSession<State>
where
    State: BridgePreviewTypestate,
{
    fn new(
        session_identity: BridgePreviewSessionIdentity,
        declaration: ValidatedBridgePreviewSessionDeclaration,
        execution_record_identity: Option<PreviewExecutionRecordIdentity>,
    ) -> Self {
        let lifecycle_state_kind = State::lifecycle_state_kind();
        let canonical_basis = Arc::<str>::from(format!(
            "preview-session|id={}|declaration={}|state:{lifecycle_state_kind:?}|execution-record={}",
            session_identity.as_str(),
            declaration.digest(),
            execution_record_identity
                .as_ref()
                .map(PreviewExecutionRecordIdentity::as_str)
                .unwrap_or("none"),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            session_identity,
            declaration,
            execution_record_identity,
            lifecycle_state_kind,
            canonical_basis,
            digest: Arc::from(format!("preview-session:sha256:{digest:x}")),
            _state: PhantomData,
        }
    }

    pub fn session_identity(&self) -> &BridgePreviewSessionIdentity {
        &self.session_identity
    }

    pub fn declaration(&self) -> &ValidatedBridgePreviewSessionDeclaration {
        &self.declaration
    }

    pub fn execution_record_identity(&self) -> Option<&PreviewExecutionRecordIdentity> {
        self.execution_record_identity.as_ref()
    }

    pub fn lifecycle_state_kind(&self) -> BridgePreviewLifecycleStateKind {
        self.lifecycle_state_kind
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[cfg(test)]
mod tests {

    use crate::snapshot::BridgeTruthViewSelector;
    use crate::source::{BridgeSourceCapability, BridgeSourceCapabilitySet};

    use super::{
        BridgePreviewSession, BridgePreviewSessionIdentity, PreviewExecutionRecordIdentity,
        PreviewSessionActivation,
    };
    use crate::speculation::{
        BridgePreviewRetainedArtifactSchema, BridgePreviewSessionBasis,
        BridgePreviewSessionDeclaration, BridgePreviewSessionDeclarationIdentity,
        BridgeRequestKind, BridgeSignalBranchIdentity, BridgeSpeculativeBranchBinding,
        BridgeSpeculativeBranchBindingIdentity, PreviewAdmitted, PreviewDeclared,
    };

    fn preview_session_basis() -> BridgePreviewSessionBasis {
        BridgePreviewSessionBasis::new(
            BridgeTruthViewSelector::committed_snapshot(
                crate::truth_identity_fixtures::truth_branch_fixture("truth-branch"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            ),
            BridgeSourceCapabilitySet::new(vec![BridgeSourceCapability::SnapshotRead]),
            BridgePreviewRetainedArtifactSchema::PreviewLifecycleArtifactsV1,
        )
    }

    #[test]
    fn preview_session_typestate_progression_is_canonical() {
        let declaration = BridgePreviewSessionDeclaration::new(
            BridgePreviewSessionDeclarationIdentity::admit_bridge_owned("preview-declaration"),
            BridgeRequestKind::Preview,
            BridgeSpeculativeBranchBinding::new(
                BridgeSpeculativeBranchBindingIdentity::admit_bridge_owned("binding"),
                crate::truth_identity_fixtures::truth_branch_fixture("truth-branch"),
                BridgeSignalBranchIdentity::admit_bridge_owned("signal-branch"),
            ),
            preview_session_basis(),
        )
        .validate()
        .expect("preview declaration should validate");

        let declared = BridgePreviewSession::<PreviewDeclared>::declare(
            BridgePreviewSessionIdentity::admit_bridge_owned("preview-session"),
            declaration.clone(),
        );
        assert_eq!(
            declared.canonical_basis(),
            format!(
                "preview-session|id=preview-session|declaration={}|state:Declared|execution-record=none",
                declaration.digest(),
            )
        );
        let admitted: BridgePreviewSession<PreviewAdmitted> = declared.admit();
        let active = admitted.activate(PreviewSessionActivation::new(
            PreviewExecutionRecordIdentity::admit_bridge_owned("preview-execution"),
        ));
        let proof = active.promotion_admissibility_proof();
        let promoted = active
            .promote(&proof)
            .expect("matching proof should promote");
        let declared = BridgePreviewSession::<PreviewDeclared>::declare(
            BridgePreviewSessionIdentity::admit_bridge_owned("preview-session-discard"),
            promoted.declaration().clone(),
        );
        let admitted: BridgePreviewSession<PreviewAdmitted> = declared.admit();
        let active = admitted.activate(PreviewSessionActivation::new(
            PreviewExecutionRecordIdentity::admit_bridge_owned("preview-execution-discard"),
        ));
        let discarded = active.discard();

        assert_eq!(
            promoted.canonical_basis(),
            format!(
                "preview-session|id=preview-session|declaration={}|state:Promoted|execution-record=preview-execution",
                promoted.declaration().digest(),
            )
        );
        assert_eq!(
            discarded.canonical_basis(),
            format!(
                "preview-session|id=preview-session-discard|declaration={}|state:Discarded|execution-record=preview-execution-discard",
                discarded.declaration().digest(),
            )
        );
    }
}
