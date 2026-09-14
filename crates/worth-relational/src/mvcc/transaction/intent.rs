use crate::schema::data::{ProposedSchemaTransition, SchemaReconciliationPolicy};

/// Caller-authored mutation policy before the Relational owner binds it to an
/// exact branch basis. It carries no branch selector or operational authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalTransactionIntent {
    pub(crate) allow_nested_savepoints: bool,
    pub(crate) proposed_schema_transition: Option<ProposedSchemaTransition>,
    pub(crate) schema_reconciliation_policy: Option<SchemaReconciliationPolicy>,
    pub(crate) materialization_mode: Option<RelationalMaterializationTransactionMode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RelationalMaterializationTransactionMode {
    Suspend,
    Rematerialize,
}

impl RelationalTransactionIntent {
    pub fn ordinary() -> Self {
        Self {
            allow_nested_savepoints: true,
            proposed_schema_transition: None,
            schema_reconciliation_policy: None,
            materialization_mode: None,
        }
    }

    pub(crate) fn materialization(mode: RelationalMaterializationTransactionMode) -> Self {
        Self {
            allow_nested_savepoints: false,
            proposed_schema_transition: None,
            schema_reconciliation_policy: None,
            materialization_mode: Some(mode),
        }
    }

    pub(crate) const fn materialization_mode(
        &self,
    ) -> Option<RelationalMaterializationTransactionMode> {
        self.materialization_mode
    }

    pub(crate) const fn allow_nested_savepoints(&self) -> bool {
        self.allow_nested_savepoints
    }

    pub(crate) fn proposed_schema_transition(&self) -> Option<&ProposedSchemaTransition> {
        self.proposed_schema_transition.as_ref()
    }

    pub(crate) fn schema_reconciliation_policy(&self) -> Option<&SchemaReconciliationPolicy> {
        self.schema_reconciliation_policy.as_ref()
    }

    pub fn with_schema_transition(
        mut self,
        proposed_schema_transition: ProposedSchemaTransition,
        schema_reconciliation_policy: Option<SchemaReconciliationPolicy>,
    ) -> Self {
        self.proposed_schema_transition = Some(proposed_schema_transition);
        self.schema_reconciliation_policy = schema_reconciliation_policy;
        self
    }
}
