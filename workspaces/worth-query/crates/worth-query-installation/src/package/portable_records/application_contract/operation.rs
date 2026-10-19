//! Whole portable application-operation meaning, including an authored body.
use super::{
    WorthQueryPortableApplicationOperationContractParts,
    WorthQueryPortableExternalEffectContractRecord,
    WorthQueryPortableInstalledReconciliationProcedureRecord,
    WorthQueryPortableOperationGraphReadScope, WorthQueryPortableOperationTouchScope,
    WorthQueryPortableTypeIdentity,
};
use worth_query_declaration::facade::application_schema::ApplicationOperationBodyProgram;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationOperationContractRecord {
    pub(super) schema: String,
    pub(super) operation: String,
    pub(super) input_type: WorthQueryPortableTypeIdentity,
    pub(super) graph_reads: Vec<WorthQueryPortableOperationGraphReadScope>,
    pub(super) touches: Vec<WorthQueryPortableOperationTouchScope>,
    pub(super) emissions: Vec<String>,
    pub(super) external_effect: Option<WorthQueryPortableExternalEffectContractRecord>,
    pub(super) reconciliation: Option<WorthQueryPortableInstalledReconciliationProcedureRecord>,
    pub(super) body: Option<ApplicationOperationBodyProgram>,
}

impl WorthQueryPortableApplicationOperationContractRecord {
    pub fn from_untrusted_parts(
        parts: WorthQueryPortableApplicationOperationContractParts,
    ) -> Self {
        Self {
            schema: parts.schema,
            operation: parts.operation,
            input_type: parts.input_type,
            graph_reads: parts.graph_reads,
            touches: parts.touches,
            emissions: parts.emissions,
            external_effect: parts.external_effect,
            reconciliation: parts.reconciliation,
            body: parts.body,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        schema: String,
        operation: String,
        input_type: WorthQueryPortableTypeIdentity,
        graph_reads: Vec<WorthQueryPortableOperationGraphReadScope>,
        touches: Vec<WorthQueryPortableOperationTouchScope>,
        emissions: Vec<String>,
        external_effect: Option<WorthQueryPortableExternalEffectContractRecord>,
        reconciliation: Option<WorthQueryPortableInstalledReconciliationProcedureRecord>,
    ) -> Self {
        Self {
            schema,
            operation,
            input_type,
            graph_reads,
            touches,
            emissions,
            external_effect,
            reconciliation,
            body: None,
        }
    }

    pub fn schema(&self) -> &str {
        &self.schema
    }
    pub fn operation(&self) -> &str {
        &self.operation
    }
    pub const fn input_type(&self) -> &WorthQueryPortableTypeIdentity {
        &self.input_type
    }
    pub fn graph_reads(&self) -> &[WorthQueryPortableOperationGraphReadScope] {
        &self.graph_reads
    }
    pub fn touches(&self) -> &[WorthQueryPortableOperationTouchScope] {
        &self.touches
    }
    pub fn emissions(&self) -> &[String] {
        &self.emissions
    }
    pub const fn external_effect(&self) -> Option<&WorthQueryPortableExternalEffectContractRecord> {
        self.external_effect.as_ref()
    }
    pub const fn reconciliation(
        &self,
    ) -> Option<&WorthQueryPortableInstalledReconciliationProcedureRecord> {
        self.reconciliation.as_ref()
    }
    #[cfg(test)]
    pub(crate) fn replace_operation_for_test(&mut self, operation: impl Into<String>) {
        self.operation = operation.into();
    }
}
