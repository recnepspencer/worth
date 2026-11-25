//! Authority-free artifact-contract carriage and exact fresh readmission.

use super::*;
use crate::domain_computation::{
    WorthQueryArtifactAccessPathContract, WorthQueryArtifactOccurrenceContract,
    WorthQueryArtifactReproducibilityContract, WorthQueryCandidateSearchContract,
    WorthQueryConvergenceContract, WorthQueryDecisionRecordContract,
    WorthQueryStructuralCounterContract, WorthQueryTransformationEvidenceContract,
};
use worth_query_declaration::archive_decode_authority::ArchiveDecodeAuthority;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableArtifactContractRecord {
    parts: WorthQueryPortableArtifactContractParts,
}

impl WorthQueryPortableArtifactContractRecord {
    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        parts: WorthQueryPortableArtifactContractParts,
    ) -> Self {
        Self { parts }
    }

    pub fn project(source: &WorthQueryPortableArtifactContract) -> Self {
        Self {
            parts: WorthQueryPortableArtifactContractParts::project(source),
        }
    }

    pub fn family(&self) -> &WorthQueryArtifactFamilyIdentity {
        &self.parts.family
    }

    pub const fn schema_version(&self) -> WorthQueryArtifactSchemaVersion {
        self.parts.schema_version
    }

    pub const fn protocol_version(&self) -> WorthQueryArtifactProtocolVersion {
        self.parts.protocol_version
    }

    pub const fn parts(&self) -> &WorthQueryPortableArtifactContractParts {
        &self.parts
    }

    pub fn into_parts(self) -> WorthQueryPortableArtifactContractParts {
        self.parts
    }

    pub(crate) fn reconstruction_work(&self) -> (u64, u64) {
        canonical_artifact_contract_reconstruction_work(&self.parts)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableArtifactContractParts {
    pub family: WorthQueryArtifactFamilyIdentity,
    pub schema_version: WorthQueryArtifactSchemaVersion,
    pub protocol_version: WorthQueryArtifactProtocolVersion,
    pub content_identity: WorthQueryArtifactContentIdentityContract,
    pub ownership: WorthQueryArtifactOwnershipContract,
    pub occurrence: WorthQueryArtifactOccurrenceContract,
    pub evidence: WorthQueryArtifactEvidenceContract,
    pub reproducibility: WorthQueryArtifactReproducibilityContract,
    pub search: WorthQueryCandidateSearchContract,
    pub convergence: WorthQueryConvergenceContract,
    pub transformation: WorthQueryTransformationEvidenceContract,
    pub access_path: WorthQueryArtifactAccessPathContract,
    pub carriage: WorthQueryArtifactCarriageContract,
    pub lifecycle: WorthQueryArtifactLifecycleContract,
    pub counters: WorthQueryStructuralCounterContract,
    pub decisions: WorthQueryDecisionRecordContract,
    pub governance: WorthQueryArtifactGovernanceContract,
    pub compatibility: WorthQueryArtifactCompatibilityContract,
    pub producer_roles: Vec<String>,
    pub consumer_roles: Vec<String>,
}

impl WorthQueryPortableArtifactContractParts {
    fn project(source: &WorthQueryPortableArtifactContract) -> Self {
        Self {
            family: source.family.clone(),
            schema_version: source.schema_version,
            protocol_version: source.protocol_version,
            content_identity: source.content_identity.clone(),
            ownership: source.ownership.clone(),
            occurrence: source.occurrence.clone(),
            evidence: source.evidence.clone(),
            reproducibility: source.reproducibility.clone(),
            search: source.search.clone(),
            convergence: source.convergence.clone(),
            transformation: source.transformation.clone(),
            access_path: source.access_path.clone(),
            carriage: source.carriage,
            lifecycle: source.lifecycle,
            counters: source.counters.clone(),
            decisions: source.decisions.clone(),
            governance: source.governance.clone(),
            compatibility: source.compatibility.clone(),
            producer_roles: source.producer_roles.clone(),
            consumer_roles: source.consumer_roles.clone(),
        }
    }
}

impl super::canonical_identity::WorthQueryArtifactContractCanonicalSemantics
    for WorthQueryPortableArtifactContractParts
{
    fn family(&self) -> &WorthQueryArtifactFamilyIdentity {
        &self.family
    }
    fn schema_version(&self) -> WorthQueryArtifactSchemaVersion {
        self.schema_version
    }
    fn protocol_version(&self) -> WorthQueryArtifactProtocolVersion {
        self.protocol_version
    }
    fn content_identity(&self) -> &WorthQueryArtifactContentIdentityContract {
        &self.content_identity
    }
    fn ownership(&self) -> &WorthQueryArtifactOwnershipContract {
        &self.ownership
    }
    fn occurrence(&self) -> &WorthQueryArtifactOccurrenceContract {
        &self.occurrence
    }
    fn evidence(&self) -> &WorthQueryArtifactEvidenceContract {
        &self.evidence
    }
    fn reproducibility(&self) -> &WorthQueryArtifactReproducibilityContract {
        &self.reproducibility
    }
    fn search(&self) -> &WorthQueryCandidateSearchContract {
        &self.search
    }
    fn convergence(&self) -> &WorthQueryConvergenceContract {
        &self.convergence
    }
    fn transformation(&self) -> &WorthQueryTransformationEvidenceContract {
        &self.transformation
    }
    fn access_path(&self) -> &WorthQueryArtifactAccessPathContract {
        &self.access_path
    }
    fn carriage(&self) -> WorthQueryArtifactCarriageContract {
        self.carriage
    }
    fn lifecycle(&self) -> WorthQueryArtifactLifecycleContract {
        self.lifecycle
    }
    fn counters(&self) -> &WorthQueryStructuralCounterContract {
        &self.counters
    }
    fn decisions(&self) -> &WorthQueryDecisionRecordContract {
        &self.decisions
    }
    fn governance(&self) -> &WorthQueryArtifactGovernanceContract {
        &self.governance
    }
    fn compatibility(&self) -> &WorthQueryArtifactCompatibilityContract {
        &self.compatibility
    }
    fn producer_roles(&self) -> &[String] {
        &self.producer_roles
    }
    fn consumer_roles(&self) -> &[String] {
        &self.consumer_roles
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryPortableArtifactContractReadmissionDenial {
    NonCanonical,
    Invalid(WorthQueryArtifactContractValidationDenial),
    CanonicalWorkBudgetExceeded { observed: u64, maximum: u64 },
}

pub fn validate_portable_artifact_contract_freshly(
    record: WorthQueryPortableArtifactContractRecord,
) -> Result<WorthQueryPortableArtifactContract, WorthQueryPortableArtifactContractReadmissionDenial>
{
    validate_portable_artifact_contract_freshly_with_work(record, u64::MAX)
        .map(|(contract, _work)| contract)
}

pub(crate) fn validate_portable_artifact_contract_freshly_with_work(
    record: WorthQueryPortableArtifactContractRecord,
    maximum_canonical_work_bytes: u64,
) -> Result<
    (WorthQueryPortableArtifactContract, u64),
    WorthQueryPortableArtifactContractReadmissionDenial,
> {
    let parts = record.parts;
    let mut contract = WorthQueryPortableArtifactContract {
        family: parts.family,
        schema_version: parts.schema_version,
        protocol_version: parts.protocol_version,
        content_identity: parts.content_identity,
        ownership: parts.ownership,
        occurrence: parts.occurrence,
        evidence: parts.evidence,
        reproducibility: parts.reproducibility,
        search: parts.search,
        convergence: parts.convergence,
        transformation: parts.transformation,
        access_path: parts.access_path,
        carriage: parts.carriage,
        lifecycle: parts.lifecycle,
        counters: parts.counters,
        decisions: parts.decisions,
        governance: parts.governance,
        compatibility: parts.compatibility,
        producer_roles: parts.producer_roles,
        consumer_roles: parts.consumer_roles,
        identity: WorthQueryArtifactContractIdentity::minted(String::new()),
    };
    let canonical_work_bytes = canonical_artifact_contract_encoded_bytes(&contract);
    if canonical_work_bytes > maximum_canonical_work_bytes {
        return Err(
            WorthQueryPortableArtifactContractReadmissionDenial::CanonicalWorkBudgetExceeded {
                observed: canonical_work_bytes,
                maximum: maximum_canonical_work_bytes,
            },
        );
    }
    let observed = contract.clone();
    contract.producer_roles.sort();
    contract.producer_roles.dedup();
    contract.consumer_roles.sort();
    contract.consumer_roles.dedup();
    contract.occurrence.canonicalize();
    contract.reproducibility.canonicalize();
    contract.governance.canonicalize();
    if contract != observed {
        return Err(WorthQueryPortableArtifactContractReadmissionDenial::NonCanonical);
    }
    validate_artifact_contract(&contract)
        .map_err(WorthQueryPortableArtifactContractReadmissionDenial::Invalid)?;
    contract.identity = canonical_artifact_contract_identity(&contract);
    Ok((contract, canonical_work_bytes))
}
