use std::collections::BTreeMap;

use worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot;

use crate::execution_digest::hash_parts;

/// Exact provider-support closure installed for one operation execution mode.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryInstalledOperationExecutionSupport {
    Direct {
        operation: WorthQueryExecutionResourceSupportSnapshot,
    },
    Workflow {
        operation: WorthQueryExecutionResourceSupportSnapshot,
        stages: BTreeMap<String, WorthQueryExecutionResourceSupportSnapshot>,
    },
}

impl WorthQueryInstalledOperationExecutionSupport {
    pub fn direct(operation: WorthQueryExecutionResourceSupportSnapshot) -> Self {
        Self::Direct { operation }
    }

    pub fn workflow(
        operation: WorthQueryExecutionResourceSupportSnapshot,
        stages: impl IntoIterator<Item = (String, WorthQueryExecutionResourceSupportSnapshot)>,
    ) -> Self {
        Self::Workflow {
            operation,
            stages: stages.into_iter().collect(),
        }
    }

    pub fn operation(&self) -> &WorthQueryExecutionResourceSupportSnapshot {
        match self {
            Self::Direct { operation } | Self::Workflow { operation, .. } => operation,
        }
    }

    pub fn direct_operation(&self) -> Option<&WorthQueryExecutionResourceSupportSnapshot> {
        match self {
            Self::Direct { operation } => Some(operation),
            Self::Workflow { .. } => None,
        }
    }

    pub fn workflow_operation(&self) -> Option<&WorthQueryExecutionResourceSupportSnapshot> {
        match self {
            Self::Direct { .. } => None,
            Self::Workflow { operation, .. } => Some(operation),
        }
    }

    pub fn workflow_stage(
        &self,
        stage_identity: &str,
    ) -> Option<&WorthQueryExecutionResourceSupportSnapshot> {
        match self {
            Self::Direct { .. } => None,
            Self::Workflow { stages, .. } => stages.get(stage_identity),
        }
    }

    pub fn identity(&self) -> String {
        let (mode, stages) = match self {
            Self::Direct { .. } => ("direct", String::new()),
            Self::Workflow { stages, .. } => (
                "workflow",
                stages
                    .iter()
                    .map(|(stage, support)| format!("{stage}:{}", support.identity()))
                    .collect::<Vec<_>>()
                    .join(","),
            ),
        };
        hash_parts(&[
            "worth_query_installed_operation_execution_support_v1".into(),
            format!("mode:{mode}"),
            format!("operation:{}", self.operation().identity()),
            format!("stages:{stages}"),
        ])
    }
}
