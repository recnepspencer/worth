use worth_query_installation::facade::WorthQueryArtifactComparatorFamily;
use worth_query_installation::facade::WorthQueryArtifactFamily;

pub const OWNER: &str = "worth.convergence.fixture";
pub const OPERATION_SLOT: &str = "iterate:1";
pub const WORKFLOW_OPERATION_SLOT: &str = "iterate-workflow:1";
pub const WORKFLOW_STAGE: &str = "iterate-stage";
pub const GRAPH_ROLE: &str = "model";

pub struct CandidateFamily;

impl WorthQueryArtifactFamily for CandidateFamily {
    const SEMANTIC_FAMILY: &'static str = "worth.convergence.candidate";
}

pub struct ComparatorFamily;

impl WorthQueryArtifactComparatorFamily for ComparatorFamily {
    const SEMANTIC_FAMILY: &'static str = "worth.convergence.comparator";
}
