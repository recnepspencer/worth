use worth_foundational::{
    FoundationalBoundaryEvidenceDescriptiveRole, FoundationalBoundaryEvidenceExecutionPosture,
};

fn needs_execution_posture(_posture: FoundationalBoundaryEvidenceExecutionPosture) {}

fn main() {
    let role = FoundationalBoundaryEvidenceDescriptiveRole::SupportGrade;
    needs_execution_posture(role);
}
