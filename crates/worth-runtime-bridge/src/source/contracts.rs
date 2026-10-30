use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::error::{BridgeBuildError, BridgeBuildErrorKind};
use crate::identity::{BridgeIdentity, SourceContractIdentityTag};

use super::{BridgeSourceCapabilitySet, SourceDeclaration};

pub type SourceContractIdentity = BridgeIdentity<SourceContractIdentityTag>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedSourceContract {
    declaration: SourceDeclaration,
    contract_identity: SourceContractIdentity,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl AdmittedSourceContract {
    fn from_declaration(declaration: SourceDeclaration) -> Self {
        let canonical_basis = Arc::<str>::from(format!(
            "admitted-source-contract|declaration={}",
            declaration.digest()
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        let contract_identity = SourceContractIdentity::admit_bridge_owned(format!(
            "source-contract:sha256:{digest:x}"
        ));

        Self {
            declaration,
            contract_identity,
            canonical_basis,
            digest: Arc::from(format!("source-contract:sha256:{digest:x}")),
        }
    }

    pub fn declaration(&self) -> &SourceDeclaration {
        &self.declaration
    }

    pub fn contract_identity(&self) -> &SourceContractIdentity {
        &self.contract_identity
    }

    pub fn required_capabilities(&self) -> &BridgeSourceCapabilitySet {
        self.declaration.required_capabilities()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedSourceRegistry {
    contracts: Arc<[AdmittedSourceContract]>,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl AdmittedSourceRegistry {
    pub fn freeze(mut declarations: Vec<SourceDeclaration>) -> Result<Self, BridgeBuildError> {
        declarations.sort_by(|left, right| {
            left.declaration_identity()
                .cmp(right.declaration_identity())
                .then_with(|| left.digest().cmp(right.digest()))
        });

        for pair in declarations.windows(2) {
            let left = &pair[0];
            let right = &pair[1];
            if left.declaration_identity() == right.declaration_identity() {
                if left.digest() == right.digest() {
                    return Err(BridgeBuildError::new(
                        BridgeBuildErrorKind::DuplicateSourceDeclaration,
                        format!(
                            "Bridge source declaration `{}` was registered more than once.",
                            left.declaration_identity().as_str()
                        ),
                    ));
                }

                return Err(BridgeBuildError::new(
                    BridgeBuildErrorKind::AmbiguousSourceDeclaration,
                    format!(
                        "Bridge source declaration `{}` was registered with conflicting canonical definitions.",
                        left.declaration_identity().as_str()
                    ),
                ));
            }
        }

        let contracts = declarations
            .into_iter()
            .map(AdmittedSourceContract::from_declaration)
            .collect::<Vec<_>>();

        let canonical_basis = Arc::<str>::from(format!(
            "admitted-source-registry|contracts={}",
            contracts
                .iter()
                .map(|contract| contract.digest())
                .collect::<Vec<_>>()
                .join(",")
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());

        Ok(Self {
            contracts: Arc::from(contracts),
            canonical_basis,
            digest: Arc::from(format!("admitted-source-registry:sha256:{digest:x}")),
        })
    }

    pub fn contracts(&self) -> &[AdmittedSourceContract] {
        &self.contracts
    }

    pub fn contract_for_declaration(
        &self,
        declaration: &SourceDeclaration,
    ) -> Option<&AdmittedSourceContract> {
        self.contracts
            .iter()
            .find(|contract| contract.declaration() == declaration)
    }

    pub fn contract_for_identity(
        &self,
        contract_identity: &str,
    ) -> Option<&AdmittedSourceContract> {
        self.contracts
            .iter()
            .find(|contract| contract.contract_identity().as_str() == contract_identity)
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn required_capabilities(&self) -> BridgeSourceCapabilitySet {
        let mut capabilities = Vec::new();
        for contract in self.contracts() {
            capabilities.extend_from_slice(contract.required_capabilities().capabilities());
        }
        BridgeSourceCapabilitySet::new(capabilities)
    }
}
