use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::error::{BridgeBuildError, BridgeBuildErrorKind};
use crate::identity::{BridgeIdentity, MergeContractIdentityTag};

use super::{MergeHistoryDeclaration, ValidatedMergeHistoryDeclaration};

pub type MergeContractIdentity = BridgeIdentity<MergeContractIdentityTag>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedMergeHistoryContract {
    validated_declaration: ValidatedMergeHistoryDeclaration,
    contract_identity: MergeContractIdentity,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl AdmittedMergeHistoryContract {
    fn from_validated_declaration(validated_declaration: ValidatedMergeHistoryDeclaration) -> Self {
        let declaration = validated_declaration.declaration();
        let canonical_basis = Arc::<str>::from(format!(
            "admitted-merge-history-contract|validated={}|ontology-version={}|policy-version={}|parent-order={}",
            validated_declaration.digest(),
            declaration.authority_basis().ontology_version(),
            declaration
                .authority_basis()
                .schema_policy_descriptor_version(),
            declaration.authority_basis().parent_order_proof().digest(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        let contract_identity = MergeContractIdentity::admit_bridge_owned(format!(
            "merge-history-contract:sha256:{digest:x}"
        ));

        Self {
            validated_declaration,
            contract_identity,
            canonical_basis,
            digest: Arc::from(format!("merge-history-contract:sha256:{digest:x}")),
        }
    }

    pub fn validated_declaration(&self) -> &ValidatedMergeHistoryDeclaration {
        &self.validated_declaration
    }

    pub fn contract_identity(&self) -> &MergeContractIdentity {
        &self.contract_identity
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedMergeRegistry {
    contracts: Arc<[AdmittedMergeHistoryContract]>,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl AdmittedMergeRegistry {
    pub fn freeze(
        mut declarations: Vec<MergeHistoryDeclaration>,
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
                        BridgeBuildErrorKind::DuplicateMergeDeclaration,
                        format!(
                            "Merge declaration `{}` was registered more than once.",
                            left.declaration_identity().as_str()
                        ),
                    ));
                }

                return Err(BridgeBuildError::new(
                    BridgeBuildErrorKind::AmbiguousMergeDeclaration,
                    format!(
                        "Merge declaration `{}` was registered with conflicting canonical definitions.",
                        left.declaration_identity().as_str()
                    ),
                ));
            }
        }

        let contracts = declarations
            .into_iter()
            .map(ValidatedMergeHistoryDeclaration::new)
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(AdmittedMergeHistoryContract::from_validated_declaration)
            .collect::<Vec<_>>();

        let canonical_basis = Arc::<str>::from(format!(
            "admitted-merge-registry|contracts={}",
            contracts
                .iter()
                .map(AdmittedMergeHistoryContract::digest)
                .collect::<Vec<_>>()
                .join(",")
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());

        Ok(Self {
            contracts: Arc::from(contracts),
            canonical_basis,
            digest: Arc::from(format!("admitted-merge-registry:sha256:{digest:x}")),
        })
    }

    pub fn empty() -> Self {
        Self::freeze(Vec::new()).expect("empty merge registry should always freeze")
    }

    pub fn contracts(&self) -> &[AdmittedMergeHistoryContract] {
        &self.contracts
    }

    pub fn contract_for_declaration(
        &self,
        declaration: &MergeHistoryDeclaration,
    ) -> Option<&AdmittedMergeHistoryContract> {
        self.contracts
            .iter()
            .find(|contract| contract.validated_declaration().declaration() == declaration)
    }

    pub fn contract_for_identity(
        &self,
        contract_identity: &str,
    ) -> Option<&AdmittedMergeHistoryContract> {
        self.contracts
            .iter()
            .find(|contract| contract.contract_identity().as_str() == contract_identity)
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
    use super::AdmittedMergeRegistry;

    use crate::merge::{
        BridgeMergeAuthorityBasis, BridgeMergeAuthorityBasisKind, BridgeMergeConsumptionClass,
        BridgeMergeOntologyMappingSurface, BridgeMergeParentOrderProof, MergeHistoryDeclaration,
        MergeHistoryDeclarationIdentity,
    };

    fn declaration(id: &str) -> MergeHistoryDeclaration {
        MergeHistoryDeclaration::new(
            MergeHistoryDeclarationIdentity::admit_bridge_owned(id),
            BridgeMergeConsumptionClass::AspectReconciliationMerge,
            BridgeMergeOntologyMappingSurface::direct_phase_m9_0("rel-merge-v1"),
            BridgeMergeAuthorityBasis::new(
                BridgeMergeAuthorityBasisKind::OrderedMergeCommit,
                format!("merge-artifact:{id}"),
                "rel-merge-v1",
                "schema-policy-v1",
                BridgeMergeParentOrderProof::new(vec![
                    crate::truth_identity_fixtures::truth_commit_fixture("parent-a"),
                    crate::truth_identity_fixtures::truth_commit_fixture("parent-b"),
                ]),
            ),
        )
    }

    #[test]
    fn merge_registry_rejects_duplicate_declaration_identity() {
        let error = AdmittedMergeRegistry::freeze(vec![
            declaration("merge:ordered-history"),
            declaration("merge:ordered-history"),
        ])
        .expect_err("duplicate merge declaration should be rejected");

        assert_eq!(
            error.kind(),
            crate::error::BridgeBuildErrorKind::DuplicateMergeDeclaration
        );
    }

    #[test]
    fn merge_registry_freezes_valid_declarations() {
        let registry = AdmittedMergeRegistry::freeze(vec![declaration("merge:ordered-history")])
            .expect("valid merge declaration should freeze");

        assert_eq!(registry.contracts().len(), 1);
        assert!(registry
            .digest()
            .starts_with("admitted-merge-registry:sha256:"));
    }
}
