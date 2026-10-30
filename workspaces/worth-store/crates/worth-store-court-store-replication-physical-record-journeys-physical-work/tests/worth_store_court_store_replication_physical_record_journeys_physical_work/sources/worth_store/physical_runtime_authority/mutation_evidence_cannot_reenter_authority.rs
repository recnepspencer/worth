use worth_store::physical_runtime::{
    PhysicalMutationExecutedBoundaryEvidence, PhysicalMutationPerformanceEvidence,
};

fn executed_evidence_cannot_acknowledge(evidence: PhysicalMutationExecutedBoundaryEvidence) {
    let _ = evidence.into_acknowledgment();
}

fn performance_evidence_cannot_acknowledge(evidence: PhysicalMutationPerformanceEvidence) {
    let _ = evidence.into_acknowledgment();
}

fn main() {}
