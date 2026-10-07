use super::{WorthQueryWorkspaceError, WorthQueryWorkspaceErrorKind};

impl From<worth_query_execution::facade::primary_graph::WorthQueryHandleDenial>
    for WorthQueryWorkspaceError
{
    fn from(denial: worth_query_execution::facade::primary_graph::WorthQueryHandleDenial) -> Self {
        Self::with_kind(
            WorthQueryWorkspaceErrorKind::Handle(denial),
            denial.to_string(),
        )
    }
}
