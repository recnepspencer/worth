use crate::harness::certification::CertificationMatrix;
use crate::identity::{BasisDigest, CanonicalQueryDigest};
use crate::identity_evolution::{
    compare_identity_evolution_denial_replay, compare_identity_evolution_result_replay,
    IdentityEvolutionAdmissionError, IdentityEvolutionCertificationDenialEvidence,
    IdentityEvolutionCertificationResultEvidence, IdentityEvolutionExecutionArtifact,
    InspectorIdentityArtifact,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum IdentityEvolutionCertificationPerturbationClass {
    LineageTraversal,
    CorrespondenceComparison,
    BranchLocality,
    IdentityBreak,
    CrossFeatureConsumption,
    Disagreement,
    ReplayParity,
    Performance,
    ComplexityContract,
    FallbackBoundary,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityEvolutionCertificationFailureClass {
    AdmissionDenied,
    ExecutionDenied,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityEvolutionCertificationLane {
    pub query_digest: String,
    pub basis_digest: String,
    pub lineage_digest: String,
    pub branch_locality_digest: String,
    pub complexity_contract_digest: String,
    pub result_digest: String,
    pub failure_digest: String,
    pub replay_digest: String,
    pub counter_snapshot_digest: String,
    pub outcome_family: String,
    pub inspector_identity_digest: String,
    pub inspector_identity_classification: String,
    pub inspector_replay_stable_digest: String,
    pub branch_locality_class: String,
    pub complexity_status: String,
    pub prediction_drift_outcome: String,
    pub exact_counter_values: Vec<String>,
}

impl IdentityEvolutionCertificationLane {
    pub fn from_execution_artifact(artifact: &IdentityEvolutionExecutionArtifact) -> Self {
        let evidence =
            IdentityEvolutionCertificationResultEvidence::from_execution_artifact(artifact);
        let inspector_identity = InspectorIdentityArtifact::from_result_evidence(&evidence);
        Self::from_execution_artifact_and_inspector(artifact, &evidence, inspector_identity)
    }

    pub fn from_execution_artifact_with_bundle_inspector(
        artifact: &IdentityEvolutionExecutionArtifact,
    ) -> Self {
        let evidence =
            IdentityEvolutionCertificationResultEvidence::from_execution_artifact(artifact);
        let inspector_identity =
            InspectorIdentityArtifact::from_result_bundle(artifact.result_bundle());
        Self::from_execution_artifact_and_inspector(artifact, &evidence, inspector_identity)
    }

    fn from_execution_artifact_and_inspector(
        artifact: &IdentityEvolutionExecutionArtifact,
        evidence: &IdentityEvolutionCertificationResultEvidence,
        inspector_identity: InspectorIdentityArtifact,
    ) -> Self {
        let replay = compare_identity_evolution_result_replay(evidence, evidence);
        Self {
            query_digest: evidence.query_digest().as_str().to_string(),
            basis_digest: evidence.basis_digest().as_str().to_string(),
            lineage_digest: evidence.lineage_digest().as_str().to_string(),
            branch_locality_digest: evidence.branch_locality_digest().as_str().to_string(),
            complexity_contract_digest: evidence.complexity_contract_digest().as_str().to_string(),
            result_digest: evidence.result_digest().to_string(),
            failure_digest: evidence.failure_digest().as_str().to_string(),
            replay_digest: replay.replay_digest().as_str().to_string(),
            counter_snapshot_digest: evidence
                .counter_snapshot()
                .counter_snapshot_digest()
                .as_str()
                .to_string(),
            outcome_family: evidence.outcome_family().as_str().to_string(),
            inspector_identity_digest: inspector_identity.digest().as_str().to_string(),
            inspector_identity_classification: inspector_identity
                .classification()
                .as_str()
                .to_string(),
            inspector_replay_stable_digest: inspector_identity.replay_stable_digest().to_string(),
            branch_locality_class: evidence.branch_locality_class().as_str().to_string(),
            complexity_status: evidence.complexity_status().as_str().to_string(),
            prediction_drift_outcome: artifact.prediction_drift_outcome().as_str().to_string(),
            exact_counter_values: evidence.counter_snapshot().exact_counter_values().to_vec(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityEvolutionCertificationRejection {
    pub failure_class: IdentityEvolutionCertificationFailureClass,
    pub query_digest: String,
    pub basis_digest: String,
    pub lineage_digest: String,
    pub branch_locality_digest: String,
    pub complexity_contract_digest: String,
    pub result_digest: String,
    pub failure_digest: String,
    pub replay_digest: String,
    pub counter_snapshot_digest: String,
    pub exact_counter_values: Vec<String>,
}

impl IdentityEvolutionCertificationRejection {
    pub fn from_admission_error(
        error: &IdentityEvolutionAdmissionError,
        query_digest: &CanonicalQueryDigest,
        basis_digest: &BasisDigest,
    ) -> Self {
        let evidence = IdentityEvolutionCertificationDenialEvidence::from_admission_error(
            error,
            query_digest,
            basis_digest,
        );
        let replay = compare_identity_evolution_denial_replay(&evidence, &evidence);
        Self {
            failure_class: IdentityEvolutionCertificationFailureClass::AdmissionDenied,
            query_digest: evidence.query_digest().as_str().to_string(),
            basis_digest: evidence.basis_digest().as_str().to_string(),
            lineage_digest: evidence.lineage_digest().as_str().to_string(),
            branch_locality_digest: evidence.branch_locality_digest().as_str().to_string(),
            complexity_contract_digest: evidence.complexity_contract_digest().as_str().to_string(),
            result_digest: evidence.result_digest().to_string(),
            failure_digest: evidence.failure_digest().as_str().to_string(),
            replay_digest: replay.replay_digest().as_str().to_string(),
            counter_snapshot_digest: evidence
                .counter_snapshot()
                .counter_snapshot_digest()
                .as_str()
                .to_string(),
            exact_counter_values: evidence.counter_snapshot().exact_counter_values().to_vec(),
        }
    }

    pub fn from_execution_artifact(artifact: &IdentityEvolutionExecutionArtifact) -> Self {
        let evidence =
            IdentityEvolutionCertificationDenialEvidence::from_execution_artifact(artifact);
        let replay = compare_identity_evolution_denial_replay(&evidence, &evidence);
        Self {
            failure_class: IdentityEvolutionCertificationFailureClass::ExecutionDenied,
            query_digest: evidence.query_digest().as_str().to_string(),
            basis_digest: evidence.basis_digest().as_str().to_string(),
            lineage_digest: evidence.lineage_digest().as_str().to_string(),
            branch_locality_digest: evidence.branch_locality_digest().as_str().to_string(),
            complexity_contract_digest: evidence.complexity_contract_digest().as_str().to_string(),
            result_digest: evidence.result_digest().to_string(),
            failure_digest: evidence.failure_digest().as_str().to_string(),
            replay_digest: replay.replay_digest().as_str().to_string(),
            counter_snapshot_digest: evidence
                .counter_snapshot()
                .counter_snapshot_digest()
                .as_str()
                .to_string(),
            exact_counter_values: evidence.counter_snapshot().exact_counter_values().to_vec(),
        }
    }
}

pub type IdentityEvolutionCertificationMatrix = CertificationMatrix<
    IdentityEvolutionCertificationPerturbationClass,
    IdentityEvolutionCertificationLane,
    IdentityEvolutionCertificationRejection,
>;
