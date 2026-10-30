use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::identity::{BridgeIdentity, StructuralCandidateIdentityTag};

use super::{
    AdmittedStructuralComparisonContract, StructuralComparisonMode, StructuralFingerprint,
    StructuralMatchOutcomeClass, ValidatedStructuralIdentityDeclaration,
};

pub type StructuralCandidateIdentity = BridgeIdentity<StructuralCandidateIdentityTag>;

impl StructuralCandidateIdentity {
    pub fn from_stable_name(value: impl Into<Arc<str>>) -> Self {
        Self::admit_bridge_owned(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StructuralMatchCandidateKind {
    ExactAdvisoryMatch,
    AdvisoryReuseCandidate,
    IdentityAuthorityConflict,
    LineageStructuralDivergence,
    BranchDiff,
}

impl StructuralMatchCandidateKind {
    pub fn implied_outcome_class(self) -> Option<StructuralMatchOutcomeClass> {
        match self {
            Self::ExactAdvisoryMatch => Some(StructuralMatchOutcomeClass::ExactAdvisoryMatch),
            Self::AdvisoryReuseCandidate => {
                Some(StructuralMatchOutcomeClass::AdvisoryReuseCandidate)
            }
            Self::IdentityAuthorityConflict => {
                Some(StructuralMatchOutcomeClass::RejectedIdentityAuthorityConflict)
            }
            Self::LineageStructuralDivergence => {
                Some(StructuralMatchOutcomeClass::RejectedLineageStructuralDivergence)
            }
            Self::BranchDiff => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralMatchCandidate {
    candidate_identity: StructuralCandidateIdentity,
    candidate_kind: StructuralMatchCandidateKind,
    fingerprint: Option<StructuralFingerprint>,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl StructuralMatchCandidate {
    pub fn new(
        candidate_identity: StructuralCandidateIdentity,
        candidate_kind: StructuralMatchCandidateKind,
    ) -> Self {
        Self::with_fingerprint(candidate_identity, candidate_kind, None)
    }

    pub fn with_fingerprint(
        candidate_identity: StructuralCandidateIdentity,
        candidate_kind: StructuralMatchCandidateKind,
        fingerprint: Option<StructuralFingerprint>,
    ) -> Self {
        let canonical_basis = Arc::<str>::from(format!(
            "structural-match-candidate|id={}|kind:{candidate_kind:?}|fingerprint={}",
            candidate_identity.as_str(),
            fingerprint
                .as_ref()
                .map(StructuralFingerprint::digest)
                .unwrap_or("none"),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            candidate_identity,
            candidate_kind,
            fingerprint,
            canonical_basis,
            digest: Arc::from(format!("structural-match-candidate:sha256:{digest:x}")),
        }
    }

    pub fn candidate_identity(&self) -> &StructuralCandidateIdentity {
        &self.candidate_identity
    }

    pub fn candidate_kind(&self) -> StructuralMatchCandidateKind {
        self.candidate_kind
    }

    pub fn fingerprint(&self) -> Option<&StructuralFingerprint> {
        self.fingerprint.as_ref()
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedStructuralMatchPacketSet {
    contract: AdmittedStructuralComparisonContract,
    validated_declaration: ValidatedStructuralIdentityDeclaration,
    target_fingerprint: Option<StructuralFingerprint>,
    comparison_fingerprint: Option<StructuralFingerprint>,
    candidates: Arc<[StructuralMatchCandidate]>,
    candidate_count: usize,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl PlannedStructuralMatchPacketSet {
    pub(crate) fn new(
        contract: AdmittedStructuralComparisonContract,
        validated_declaration: ValidatedStructuralIdentityDeclaration,
        target_fingerprint: Option<StructuralFingerprint>,
        comparison_fingerprint: Option<StructuralFingerprint>,
        mut candidates: Vec<StructuralMatchCandidate>,
    ) -> Self {
        candidates.sort_by(|left, right| {
            left.candidate_identity()
                .cmp(right.candidate_identity())
                .then_with(|| left.candidate_kind().cmp(&right.candidate_kind()))
        });
        candidates.dedup();

        let candidate_count = candidates.len();
        let canonical_basis = Arc::<str>::from(format!(
            "planned-structural-match-packet-set|contract={}|validated={}|target={}|comparison={}|candidates={}",
            contract.digest(),
            validated_declaration.digest(),
            target_fingerprint
                .as_ref()
                .map(StructuralFingerprint::digest)
                .unwrap_or("none"),
            comparison_fingerprint
                .as_ref()
                .map(StructuralFingerprint::digest)
                .unwrap_or("none"),
            candidates
                .iter()
                .map(|candidate| candidate.digest())
                .collect::<Vec<_>>()
                .join(","),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());

        Self {
            contract,
            validated_declaration,
            target_fingerprint,
            comparison_fingerprint,
            candidates: Arc::from(candidates),
            candidate_count,
            canonical_basis,
            digest: Arc::from(format!(
                "planned-structural-match-packet-set:sha256:{digest:x}"
            )),
        }
    }

    pub fn contract(&self) -> &AdmittedStructuralComparisonContract {
        &self.contract
    }

    pub fn validated_declaration(&self) -> &ValidatedStructuralIdentityDeclaration {
        &self.validated_declaration
    }

    pub fn candidates(&self) -> &[StructuralMatchCandidate] {
        &self.candidates
    }

    pub fn target_fingerprint(&self) -> Option<&StructuralFingerprint> {
        self.target_fingerprint.as_ref()
    }

    pub fn comparison_fingerprint(&self) -> Option<&StructuralFingerprint> {
        self.comparison_fingerprint.as_ref()
    }

    pub fn candidate_count(&self) -> usize {
        self.candidate_count
    }

    pub fn comparison_mode(&self) -> StructuralComparisonMode {
        self.validated_declaration.declaration().comparison_mode()
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
    use super::{
        PlannedStructuralMatchPacketSet, StructuralCandidateIdentity, StructuralMatchCandidate,
        StructuralMatchCandidateKind,
    };

    use crate::snapshot::BridgeTruthViewSelector;
    use crate::structural::{
        AdmittedStructuralRegistry, StructuralFingerprintEquivalenceContract,
        StructuralFingerprintFamily, StructuralFingerprintNormalizationRule,
        StructuralFingerprintOmissionPolicy, StructuralFingerprintOrderingRule,
        StructuralIdentityDeclaration, StructuralIdentityDeclarationIdentity,
        StructuralSchemaIdentity, StructuralTruthViewBasis, ValidatedStructuralIdentityDeclaration,
    };

    fn admitted_contract() -> crate::structural::AdmittedStructuralComparisonContract {
        let declaration = StructuralIdentityDeclaration::advisory_remap(
            StructuralIdentityDeclarationIdentity::admit_bridge_owned("structural:geometry"),
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
        );
        let registry = AdmittedStructuralRegistry::freeze(vec![declaration.clone()])
            .expect("structural registry should freeze");
        registry.contracts()[0].clone()
    }

    #[test]
    fn planned_structural_packet_set_is_canonical_for_same_inputs() {
        let contract = admitted_contract();
        let validated = ValidatedStructuralIdentityDeclaration::from_contract(&contract);
        let left = PlannedStructuralMatchPacketSet::new(
            contract.clone(),
            validated.clone(),
            None,
            None,
            vec![StructuralMatchCandidate::new(
                StructuralCandidateIdentity::admit_bridge_owned("candidate:a"),
                StructuralMatchCandidateKind::ExactAdvisoryMatch,
            )],
        );
        let right = PlannedStructuralMatchPacketSet::new(
            contract,
            validated,
            None,
            None,
            vec![StructuralMatchCandidate::new(
                StructuralCandidateIdentity::admit_bridge_owned("candidate:a"),
                StructuralMatchCandidateKind::ExactAdvisoryMatch,
            )],
        );

        assert_eq!(left, right);
        assert_eq!(left.candidate_count(), 1);
    }
}
