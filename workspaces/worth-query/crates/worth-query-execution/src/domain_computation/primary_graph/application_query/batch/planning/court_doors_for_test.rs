//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_query::batch::WorthQueryApplicationQueryBatchResourceDenial;

impl WorthQueryApplicationQueryBatchReadPlan {
    #[doc(hidden)]
    pub fn query_for_test(&self) -> &WorthQueryInstalledApplicationQueryIdentity {
        &self.query
    }
}

impl WorthQueryApplicationQueryBatchReadPlan {
    #[doc(hidden)]
    pub fn roots_for_test(&self) -> &usize {
        &self.roots
    }
}

impl WorthQueryApplicationQueryBatchReadPlan {
    #[doc(hidden)]
    pub fn runtime_for_test(&self) -> &u64 {
        &self.runtime
    }
}

impl WorthQueryApplicationQueryBatchReadPlan {
    #[doc(hidden)]
    pub fn schema_for_test(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema
    }
}

impl WorthQueryApplicationQueryBatchAdmission {
    #[doc(hidden)]
    pub fn take_planned_read_for_test(
        &self,
        runtime: u64,
        schema: &ApplicationSchemaBindingIdentity,
        query: &WorthQueryInstalledApplicationQueryIdentity,
        maximum_roots: usize,
    ) -> Result<Option<PlannedBatchItem>, WorthQueryApplicationQueryBatchResourceDenial> {
        Self::take_planned_read(self, runtime, schema, query, maximum_roots)
    }
}
