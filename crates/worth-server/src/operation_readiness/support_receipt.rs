#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerOperationSupportCompositionReceipt {
    query_rows_consulted: Vec<String>,
    product_rows_consulted: Vec<String>,
    dependency_relation: String,
    planner_posture: String,
    canonical_digest: String,
}

impl WorthServerOperationSupportCompositionReceipt {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        use crate::product_adapter::execution_pipeline::read_batch_accounting::{string, strings};
        strings(&self.query_rows_consulted)
            .saturating_add(strings(&self.product_rows_consulted))
            .saturating_add(string(&self.dependency_relation))
            .saturating_add(string(&self.planner_posture))
            .saturating_add(string(&self.canonical_digest))
    }
    pub(crate) fn new(
        query_rows_consulted: Vec<String>,
        product_rows_consulted: Vec<String>,
        dependency_relation: impl Into<String>,
        planner_posture: impl Into<String>,
    ) -> Self {
        let dependency_relation = dependency_relation.into();
        let planner_posture = planner_posture.into();
        let canonical_digest = format!(
            "worth-server-operation-support-composition-receipt-v1|query={}|product={}|dependency={dependency_relation}|planner={planner_posture}",
            query_rows_consulted.join(","),
            product_rows_consulted.join(","),
        );
        Self {
            query_rows_consulted,
            product_rows_consulted,
            dependency_relation,
            planner_posture,
            canonical_digest,
        }
    }

    pub fn query_rows_consulted(&self) -> &[String] {
        &self.query_rows_consulted
    }

    pub fn product_rows_consulted(&self) -> &[String] {
        &self.product_rows_consulted
    }

    pub fn dependency_relation(&self) -> &str {
        &self.dependency_relation
    }

    pub fn planner_posture(&self) -> &str {
        &self.planner_posture
    }

    pub fn canonical_digest(&self) -> &str {
        &self.canonical_digest
    }
}
