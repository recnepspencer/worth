//! Cold canonical support snapshot installation.

use super::*;

impl WorthQueryExecutionResourceSupportSnapshot {
    pub fn new(
        executor: WorthQueryExecutionResourceSupport,
        mut conditional_nodes: Vec<(String, WorthQueryExecutionResourceSupport)>,
        mut graph_providers: Vec<(String, WorthQueryExecutionResourceSupport)>,
        mut commit_providers: Vec<(String, WorthQueryExecutionResourceSupport)>,
        parallel_admission: Option<WorthQueryExecutionResourceSupport>,
    ) -> Self {
        conditional_nodes.sort_by(|left, right| left.0.cmp(&right.0));
        graph_providers.sort_by(|left, right| left.0.cmp(&right.0));
        commit_providers.sort_by(|left, right| left.0.cmp(&right.0));
        let identity = Arc::<str>::from(hash_parts(&[
            "worth_query_execution_resource_support_snapshot_v1".into(),
            format!("executor:{}", executor.identity()),
            format!(
                "conditionals:{}",
                conditional_nodes
                    .iter()
                    .map(|(location, support)| format!("{location}:{}", support.identity()))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            format!(
                "graphs:{}",
                graph_providers
                    .iter()
                    .map(|(role, support)| format!("{role}:{}", support.identity()))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            format!(
                "commits:{}",
                commit_providers
                    .iter()
                    .map(|(group, support)| format!("{group}:{}", support.identity()))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            format!(
                "parallel:{}",
                parallel_admission
                    .as_ref()
                    .map_or("none", WorthQueryExecutionResourceSupport::identity)
            ),
        ]));
        Self {
            installed: Arc::new(InstalledResourceSupportSnapshot {
                executor,
                conditional_nodes,
                graph_providers,
                commit_providers,
                parallel_admission,
                identity,
            }),
        }
    }
}
