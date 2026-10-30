use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::error::{BridgeBuildError, BridgeBuildErrorKind};
use crate::identity::{BridgeIdentity, StructuralContractIdentityTag};

use super::{StructuralIdentityDeclaration, ValidatedStructuralIdentityDeclaration};

pub type StructuralContractIdentity = BridgeIdentity<StructuralContractIdentityTag>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedStructuralComparisonContract {
    validated_declaration: ValidatedStructuralIdentityDeclaration,
    contract_identity: StructuralContractIdentity,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl AdmittedStructuralComparisonContract {
    fn from_validated_declaration(
        validated_declaration: ValidatedStructuralIdentityDeclaration,
    ) -> Self {
        let canonical_basis = Arc::<str>::from(format!(
            "admitted-structural-contract|validated-declaration={}",
            validated_declaration.digest()
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        let contract_identity = StructuralContractIdentity::admit_bridge_owned(format!(
            "structural-contract:sha256:{digest:x}"
        ));
        Self {
            validated_declaration,
            contract_identity,
            canonical_basis,
            digest: Arc::from(format!("structural-contract:sha256:{digest:x}")),
        }
    }

    pub fn validated_declaration(&self) -> &ValidatedStructuralIdentityDeclaration {
        &self.validated_declaration
    }

    pub fn contract_identity(&self) -> &StructuralContractIdentity {
        &self.contract_identity
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedStructuralRegistry {
    contracts: Arc<[AdmittedStructuralComparisonContract]>,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl AdmittedStructuralRegistry {
    pub fn freeze(
        mut declarations: Vec<StructuralIdentityDeclaration>,
    ) -> Result<Self, BridgeBuildError> {
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
                        BridgeBuildErrorKind::DuplicateStructuralDeclaration,
                        format!(
                            "Structural declaration `{}` was registered more than once.",
                            left.declaration_identity().as_str()
                        ),
                    ));
                }

                return Err(BridgeBuildError::new(
                    BridgeBuildErrorKind::AmbiguousStructuralDeclaration,
                    format!(
                        "Structural declaration `{}` was registered with conflicting canonical definitions.",
                        left.declaration_identity().as_str()
                    ),
                ));
            }
        }

        let contracts = declarations
            .into_iter()
            .map(ValidatedStructuralIdentityDeclaration::new)
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(Self::contract_from_validated)
            .collect::<Vec<_>>();

        let canonical_basis = Arc::<str>::from(format!(
            "admitted-structural-registry|contracts={}",
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
            digest: Arc::from(format!("admitted-structural-registry:sha256:{digest:x}")),
        })
    }

    fn contract_from_validated(
        validated: ValidatedStructuralIdentityDeclaration,
    ) -> AdmittedStructuralComparisonContract {
        AdmittedStructuralComparisonContract::from_validated_declaration(validated)
    }

    pub fn contracts(&self) -> &[AdmittedStructuralComparisonContract] {
        &self.contracts
    }

    pub fn contract_for_declaration(
        &self,
        declaration: &StructuralIdentityDeclaration,
    ) -> Option<&AdmittedStructuralComparisonContract> {
        self.contracts
            .iter()
            .find(|contract| contract.validated_declaration().declaration() == declaration)
    }

    pub fn contract_for_identity(
        &self,
        contract_identity: &str,
    ) -> Option<&AdmittedStructuralComparisonContract> {
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
}

#[cfg(test)]
mod tests {
    use super::AdmittedStructuralRegistry;

    use crate::snapshot::BridgeTruthViewSelector;
    use crate::structural::{
        StructuralFingerprintEquivalenceContract, StructuralFingerprintFamily,
        StructuralFingerprintNormalizationRule, StructuralFingerprintOmissionPolicy,
        StructuralFingerprintOrderingRule, StructuralIdentityDeclaration,
        StructuralIdentityDeclarationIdentity, StructuralSchemaIdentity, StructuralTruthViewBasis,
    };

    fn declaration(id: &str) -> StructuralIdentityDeclaration {
        StructuralIdentityDeclaration::advisory_remap(
            StructuralIdentityDeclarationIdentity::admit_bridge_owned(id),
            StructuralSchemaIdentity::admit_bridge_owned("schema:geometry"),
            StructuralFingerprintEquivalenceContract::new(
                StructuralSchemaIdentity::admit_bridge_owned("schema:geometry"),
                StructuralFingerprintFamily::TopologyFingerprint,
                "topology-v1",
                StructuralFingerprintNormalizationRule::SchemaDeclaredCanonicalForm,
                StructuralFingerprintOrderingRule::SchemaDeclaredCanonicalOrder,
                StructuralFingerprintOmissionPolicy::SchemaDeclaredOmissionPolicy,
            ),
            StructuralTruthViewBasis::explicit_snapshot(
                BridgeTruthViewSelector::committed_snapshot(
                    crate::truth_identity_fixtures::truth_branch_fixture("main"),
                    crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                ),
            ),
        )
    }

    #[test]
    fn structural_registry_rejects_duplicate_declaration_identity() {
        let error = AdmittedStructuralRegistry::freeze(vec![
            declaration("structural:geometry"),
            declaration("structural:geometry"),
        ])
        .expect_err("duplicate declaration should be rejected");

        assert_eq!(
            error.kind(),
            crate::error::BridgeBuildErrorKind::DuplicateStructuralDeclaration
        );
    }

    #[test]
    fn structural_registry_freezes_valid_declarations() {
        let registry = AdmittedStructuralRegistry::freeze(vec![declaration("structural:geometry")])
            .expect("valid structural declaration should freeze");

        assert_eq!(registry.contracts().len(), 1);
        assert!(registry
            .digest()
            .starts_with("admitted-structural-registry:sha256:"));
    }
}
