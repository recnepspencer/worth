//! Account only where the actual lease or its authentic ancestor requires it.
use super::{Form, QueryRequestExecution};

impl QueryRequestExecution<'_> {
    pub(in crate::domain_computation::primary_graph) fn has_accounting(&self) -> bool {
        match &self.form {
            Form::Leased { lease, policy } | Form::Serial { lease, policy } => lease
                .as_ref()
                .map_or(policy.accounting().is_some(), |lease| {
                    lease.has_accounting()
                }),
        }
    }
}
