pub use worth_query::facade::{domain, installed};

pub use super::{conditional_node_contract, installed_operation_fixture::conditional_workspace};

pub fn workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    identity: &str,
) -> worth_query::facade::runtime::WorthQueryWorkspace {
    let node = conditional_node_contract::node(
        "geometry",
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQuerySemanticLocality::SourceRecord,
    );
    conditional_workspace(request_execution, identity, node).unwrap()
}
