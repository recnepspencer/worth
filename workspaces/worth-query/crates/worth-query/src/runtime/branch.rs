use super::{
    WorthQueryAuthorityLane, WorthQueryBranchBasisAdmission, WorthQueryBranchIntentReceipt,
    WorthQueryBranchOptions, WorthQueryEffectPolicy, WorthQueryIntentDeclaration,
    WorthQueryIntentDenialEvidence, WorthQueryIntentSourceLane, WorthQueryRuntime,
    WorthQueryRuntimeError, WorthQueryRuntimeFacadeFamily,
};
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::session_label::WorthQuerySessionLabel;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorthQueryRuntimeBranchComparisonBasis {
    admission: WorthQueryBranchBasisAdmission,
    snapshot: WorthQuerySnapshotIdentity,
}

impl WorthQueryRuntimeBranchComparisonBasis {
    pub(super) fn new(
        admission: WorthQueryBranchBasisAdmission,
        snapshot: WorthQuerySnapshotIdentity,
    ) -> Self {
        Self {
            admission,
            snapshot,
        }
    }

    pub(crate) fn admission(&self) -> &WorthQueryBranchBasisAdmission {
        &self.admission
    }

    pub(crate) fn snapshot(&self) -> &WorthQuerySnapshotIdentity {
        &self.snapshot
    }
}

pub struct WorthQueryBranchSession<'a> {
    label: WorthQuerySessionLabel,
    runtime: &'a mut WorthQueryRuntime,
    effect_policy: WorthQueryEffectPolicy,
    basis_admission: WorthQueryBranchBasisAdmission,
    basis_snapshot_identity: WorthQuerySnapshotIdentity,
    intent_receipts: Vec<WorthQueryBranchIntentReceipt>,
}

impl<'a> WorthQueryBranchSession<'a> {
    pub(super) fn new(
        label: WorthQuerySessionLabel,
        runtime: &'a mut WorthQueryRuntime,
        options: WorthQueryBranchOptions,
        basis_admission: WorthQueryBranchBasisAdmission,
    ) -> Result<Self, WorthQueryRuntimeError> {
        let basis_snapshot_identity = runtime.current_snapshot_identity()?;
        Ok(Self {
            label,
            runtime,
            effect_policy: options.effect_policy(),
            basis_admission,
            basis_snapshot_identity,
            intent_receipts: Vec::new(),
        })
    }

    pub fn label(&self) -> &str {
        self.label.display()
    }

    pub fn session_label(&self) -> &WorthQuerySessionLabel {
        &self.label
    }

    pub fn effect_policy(&self) -> WorthQueryEffectPolicy {
        self.effect_policy
    }

    pub fn basis_admission(&self) -> &WorthQueryBranchBasisAdmission {
        &self.basis_admission
    }

    pub fn branch_intent_receipts(&self) -> &[WorthQueryBranchIntentReceipt] {
        &self.intent_receipts
    }

    pub fn execute_intent(
        &mut self,
        declaration: WorthQueryIntentDeclaration,
    ) -> Result<WorthQueryBranchIntentReceipt, WorthQueryRuntimeError> {
        self.runtime.admit_facade_family_lane(
            WorthQueryRuntimeFacadeFamily::Intent,
            WorthQueryAuthorityLane::BranchLocalTruth,
        )?;
        let declaration = declaration
            .with_source_lane(WorthQueryIntentSourceLane::BranchLocal)
            .with_target_lane(WorthQueryAuthorityLane::BranchLocalTruth);
        let admission = crate::runtime::intent::admit_branch_intent_declaration(
            &declaration,
            self.effect_policy,
        )
        .map_err(|denial| WorthQueryRuntimeError::IntentCommitDenied {
            intent_name: declaration.name().to_string(),
            stage: denial.stage(),
            message: denial.message().to_string(),
            evidence: WorthQueryIntentDenialEvidence::new(&declaration, &denial, None),
        })?;
        let receipt = WorthQueryBranchIntentReceipt::new(
            &declaration,
            self.effect_policy,
            &self.basis_admission,
            &self.basis_snapshot_identity,
            admission,
        );
        self.intent_receipts.push(receipt.clone());
        Ok(receipt)
    }
}
