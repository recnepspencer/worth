use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::identity::{BridgeIdentity, WritebackContractIdentityTag};
use crate::policy::LoweredBridgeExecutionPolicy;

use super::ValidatedBridgeWritebackDeclaration;

pub type BridgeWritebackContractIdentity = BridgeIdentity<WritebackContractIdentityTag>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeWritebackAuthorityInputs {
    replay_artifacts_permitted: bool,
    diagnostics_tier: crate::policy::BridgeDiagnosticsTier,
}

impl BridgeWritebackAuthorityInputs {
    pub fn new(
        replay_artifacts_permitted: bool,
        diagnostics_tier: crate::policy::BridgeDiagnosticsTier,
    ) -> Self {
        Self {
            replay_artifacts_permitted,
            diagnostics_tier,
        }
    }

    pub fn replay_artifacts_permitted(&self) -> bool {
        self.replay_artifacts_permitted
    }

    pub fn diagnostics_tier(&self) -> crate::policy::BridgeDiagnosticsTier {
        self.diagnostics_tier
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedBridgeWritebackContract {
    contract_identity: BridgeWritebackContractIdentity,
    validated_declaration: ValidatedBridgeWritebackDeclaration,
    authority_inputs: BridgeWritebackAuthorityInputs,
    lowered_policy_digest: Arc<str>,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl AdmittedBridgeWritebackContract {
    pub(crate) fn new(
        validated_declaration: ValidatedBridgeWritebackDeclaration,
        authority_inputs: BridgeWritebackAuthorityInputs,
        lowered_policy: &LoweredBridgeExecutionPolicy,
    ) -> Result<Self, crate::error::BridgeWritebackError> {
        if validated_declaration.declaration().request_mode()
            != crate::writeback::BridgeWritebackRequestMode::WritebackCapable
        {
            return Err(crate::error::BridgeWritebackError::new(
                crate::error::BridgeWritebackErrorKind::WritebackNotRequested,
                format!(
                    "Writeback declaration `{}` cannot admit authority contract without writeback-capable request mode.",
                    validated_declaration
                        .declaration()
                        .declaration_identity()
                        .as_str()
                ),
            ));
        }

        if !authority_inputs.replay_artifacts_permitted() {
            return Err(crate::error::BridgeWritebackError::new(
                crate::error::BridgeWritebackErrorKind::PolicyRejected,
                format!(
                    "Writeback declaration `{}` requires canonical replay artifacts but runtime policy disables replay retention.",
                    validated_declaration
                        .declaration()
                        .declaration_identity()
                        .as_str()
                ),
            ));
        }

        let lowered_policy_digest = Arc::<str>::from(lowered_policy.digest().to_owned());
        let canonical_basis = Arc::<str>::from(format!(
            "admitted-bridge-writeback-contract|declaration={}|validated={}|diagnostics:{:?}|replay:{}|lowered-policy={}",
            validated_declaration
                .declaration()
                .declaration_identity()
                .as_str(),
            validated_declaration.canonical_basis(),
            authority_inputs.diagnostics_tier(),
            authority_inputs.replay_artifacts_permitted(),
            lowered_policy_digest.as_ref(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());

        Ok(Self {
            contract_identity: BridgeWritebackContractIdentity::admit_bridge_owned(format!(
                "bridge-writeback-contract:sha256:{digest:x}"
            )),
            validated_declaration,
            authority_inputs,
            lowered_policy_digest,
            canonical_basis,
            digest: Arc::from(format!("bridge-writeback-contract:sha256:{digest:x}")),
        })
    }

    pub fn contract_identity(&self) -> &BridgeWritebackContractIdentity {
        &self.contract_identity
    }

    pub fn validated_declaration(&self) -> &ValidatedBridgeWritebackDeclaration {
        &self.validated_declaration
    }

    pub fn authority_inputs(&self) -> &BridgeWritebackAuthorityInputs {
        &self.authority_inputs
    }

    pub fn lowered_policy_digest(&self) -> &str {
        self.lowered_policy_digest.as_ref()
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}
