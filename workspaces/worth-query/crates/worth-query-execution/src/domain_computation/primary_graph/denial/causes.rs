//! Original installation refusals survive every publication wrapper.
use super::WorthQueryPrimaryGraphInstallationDenial;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum InstallationDenialCause {
    ValueEncoding(worth_query_installation::facade::ApplicationValueEncodeDenial),
    IndexExecution(worth_relational::facade::indexes::DerivedIndexExecutionDenial),
    FieldEncoding {
        field: worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus,
        cause: worth_query_installation::facade::ApplicationValueEncodeDenial,
    },
    Authorization(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial),
    InitialSchema(worth_relational::facade::runtime::RelationalInitialSchemaInstallationDenial),
    BridgeConditional(worth_runtime_bridge::facade::BridgeConditionalDenial),
    BridgeBuild(worth_runtime_bridge::facade::BridgeBuildError),
    ManagedSource(worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial),
    NativeRecovery(worth_relational::facade::durability::RelationalRecoveryStop),
    Content(worth_relational::facade::content::RelationalContentDenial),
    Readiness(super::super::application_contribution::WorthQueryOutputReadinessContractDenial),
    NativeCommit {
        original: Box<worth_relational::facade::transactions::TransactionCommitError>,
        execution: crate::domain_computation::WorthQueryInvariantExecutionFailure,
    },
    Schema(worth_query_installation::facade::WorthQueryInstalledApplicationSchemaDenial),
    RuntimeConstruction(worth_relational::facade::runtime::RelationalRuntimeConstructionDenial),
    BranchBasis(worth_relational::facade::branch::RelationalBranchBasisDenial),
    TransactionAdmission(
        worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial,
    ),
    Staging(worth_relational::facade::mvcc::RelationalTransactionStagingDenial),
}

impl WorthQueryPrimaryGraphInstallationDenial {
    pub fn value_encoding_denial(
        &self,
    ) -> Option<&worth_query_installation::facade::ApplicationValueEncodeDenial> {
        match self.cause.as_deref()? {
            InstallationDenialCause::ValueEncoding(cause)
            | InstallationDenialCause::FieldEncoding { cause, .. } => Some(cause),
            _ => None,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn with_value_encoding_denial(
        mut self,
        cause: worth_query_installation::facade::ApplicationValueEncodeDenial,
    ) -> Self {
        self.cause = Some(Box::new(InstallationDenialCause::ValueEncoding(cause)));
        self
    }
    pub fn index_execution_cause(
        &self,
    ) -> Option<&worth_relational::facade::indexes::DerivedIndexExecutionDenial> {
        match self.cause.as_deref()? {
            InstallationDenialCause::IndexExecution(cause) => Some(cause),
            _ => None,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn with_index_execution_cause(
        mut self,
        cause: worth_relational::facade::indexes::DerivedIndexExecutionDenial,
    ) -> Self {
        self.cause = Some(Box::new(InstallationDenialCause::IndexExecution(cause)));
        self
    }
    /// The exact field and original codec refusal from a deferred bootstrap seed.
    pub fn field_encoding_denial(
        &self,
    ) -> Option<(
        &worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus,
        &worth_query_installation::facade::ApplicationValueEncodeDenial,
    )> {
        match self.cause.as_deref()? {
            InstallationDenialCause::FieldEncoding { field, cause } => Some((field, cause)),
            _ => None,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn with_field_encoding_denial(
        mut self,
        field: worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus,
        cause: worth_query_installation::facade::ApplicationValueEncodeDenial,
    ) -> Self {
        self.subject = format!(
            "{}.{}.{}: {}",
            field.entity(),
            field.aspect(),
            field.field(),
            self.subject
        );
        self.cause = Some(Box::new(InstallationDenialCause::FieldEncoding {
            field,
            cause,
        }));
        self
    }
    pub fn authorization_cause(
        &self,
    ) -> Option<&crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial>
    {
        match self.cause.as_deref()? {
            InstallationDenialCause::Authorization(cause) => Some(cause),
            _ => None,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn with_authorization_cause(
        mut self,
        cause: crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    ) -> Self {
        self.cause = Some(Box::new(InstallationDenialCause::Authorization(cause)));
        self
    }
    /// Original Native current-ownership rejection, without parsing display text.
    pub fn native_recovery_cause(
        &self,
    ) -> Option<&worth_relational::facade::durability::RelationalRecoveryStop> {
        if let Some(InstallationDenialCause::NativeRecovery(cause)) = self.cause.as_deref() {
            Some(cause)
        } else {
            None
        }
    }
    pub(in crate::domain_computation::primary_graph) fn with_native_recovery_cause(
        mut self,
        cause: worth_relational::facade::durability::RelationalRecoveryStop,
    ) -> Self {
        self.cause = Some(Box::new(InstallationDenialCause::NativeRecovery(cause)));
        self
    }
    pub fn content_cause(
        &self,
    ) -> Option<&worth_relational::facade::content::RelationalContentDenial> {
        match self.cause.as_deref() {
            Some(InstallationDenialCause::Content(cause))
            | Some(InstallationDenialCause::NativeRecovery(
                worth_relational::facade::durability::RelationalRecoveryStop::Content(cause),
            )) => Some(cause),
            _ => None,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn with_content_cause(
        mut self,
        cause: worth_relational::facade::content::RelationalContentDenial,
    ) -> Self {
        self.cause = Some(Box::new(InstallationDenialCause::Content(cause)));
        self
    }
    pub fn readiness_denial(
        &self,
    ) -> Option<&super::super::application_contribution::WorthQueryOutputReadinessContractDenial>
    {
        match self.cause.as_deref()? {
            InstallationDenialCause::Readiness(denial) => Some(denial),
            _ => None,
        }
    }

    /// Original Native preparation/validation failure from the refused seed commit.
    pub fn invariant_execution_failure(
        &self,
    ) -> Option<&crate::domain_computation::WorthQueryInvariantExecutionFailure> {
        match self.cause.as_deref()? {
            InstallationDenialCause::NativeCommit { execution, .. } => Some(execution),
            _ => None,
        }
    }
    pub fn managed_source_denial(
        &self,
    ) -> Option<&worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial> {
        match self.cause.as_deref()? {
            InstallationDenialCause::ManagedSource(cause) => Some(cause),
            InstallationDenialCause::NativeCommit { execution, .. } => {
                execution.managed_source_denial()
            }
            _ => None,
        }
    }
    /// Original custom invariant verdict/failure, not a string classification.
    pub fn custom_invariant_denial(
        &self,
    ) -> Option<&crate::domain_computation::WorthQueryCustomInvariantDenial> {
        self.invariant_execution_failure()?
            .custom_invariant_denial()
    }
    /// Original native law evidence from the refused bootstrap commit.
    pub fn relation_law(
        &self,
    ) -> Option<crate::domain_computation::WorthQueryRelationLawDenial<'_>> {
        self.invariant_execution_failure()?.relation_law()
    }
    pub fn native_commit_denial(
        &self,
    ) -> Option<&worth_relational::facade::transactions::TransactionCommitError> {
        match self.cause.as_deref()? {
            InstallationDenialCause::NativeCommit { original, .. } => Some(original),
            _ => None,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn with_native_commit_denial(
        mut self,
        original: worth_relational::facade::transactions::TransactionCommitError,
    ) -> Self {
        let execution = crate::domain_computation::primary_graph::provider::invariant_execution_failure::map_validation_failure(original.clone());
        self.cause = Some(Box::new(InstallationDenialCause::NativeCommit {
            original: Box::new(original),
            execution,
        }));
        self
    }
    pub fn schema_cause(
        &self,
    ) -> Option<&worth_query_installation::facade::WorthQueryInstalledApplicationSchemaDenial> {
        match self.cause.as_deref()? {
            InstallationDenialCause::Schema(cause) => Some(cause),
            _ => None,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn with_schema_cause(
        mut self,
        cause: worth_query_installation::facade::WorthQueryInstalledApplicationSchemaDenial,
    ) -> Self {
        self.cause = Some(Box::new(InstallationDenialCause::Schema(cause)));
        self
    }
    pub fn runtime_construction_cause(
        &self,
    ) -> Option<&worth_relational::facade::runtime::RelationalRuntimeConstructionDenial> {
        match self.cause.as_deref()? {
            InstallationDenialCause::RuntimeConstruction(cause) => Some(cause),
            _ => None,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn with_runtime_construction_cause(
        mut self,
        cause: worth_relational::facade::runtime::RelationalRuntimeConstructionDenial,
    ) -> Self {
        self.cause = Some(Box::new(InstallationDenialCause::RuntimeConstruction(
            cause,
        )));
        self
    }
    pub fn branch_basis_cause(
        &self,
    ) -> Option<&worth_relational::facade::branch::RelationalBranchBasisDenial> {
        match self.cause.as_deref()? {
            InstallationDenialCause::BranchBasis(cause) => Some(cause),
            _ => None,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn with_branch_basis_cause(
        mut self,
        cause: worth_relational::facade::branch::RelationalBranchBasisDenial,
    ) -> Self {
        self.cause = Some(Box::new(InstallationDenialCause::BranchBasis(cause)));
        self
    }
    pub fn transaction_admission_cause(
        &self,
    ) -> Option<&worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial> {
        match self.cause.as_deref()? {
            InstallationDenialCause::TransactionAdmission(cause) => Some(cause),
            _ => None,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn with_transaction_admission_cause(
        mut self,
        cause: worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial,
    ) -> Self {
        self.cause = Some(Box::new(InstallationDenialCause::TransactionAdmission(
            cause,
        )));
        self
    }
    pub fn staging_cause(
        &self,
    ) -> Option<&worth_relational::facade::mvcc::RelationalTransactionStagingDenial> {
        match self.cause.as_deref()? {
            InstallationDenialCause::Staging(cause) => Some(cause),
            _ => None,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn with_staging_cause(
        mut self,
        cause: worth_relational::facade::mvcc::RelationalTransactionStagingDenial,
    ) -> Self {
        self.cause = Some(Box::new(InstallationDenialCause::Staging(cause)));
        self
    }
    pub fn initial_schema_cause(
        &self,
    ) -> Option<&worth_relational::facade::runtime::RelationalInitialSchemaInstallationDenial> {
        match self.cause.as_deref()? {
            InstallationDenialCause::InitialSchema(cause) => Some(cause),
            _ => None,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn with_initial_schema_cause(
        mut self,
        cause: worth_relational::facade::runtime::RelationalInitialSchemaInstallationDenial,
    ) -> Self {
        self.cause = Some(Box::new(InstallationDenialCause::InitialSchema(cause)));
        self
    }
    pub fn bridge_conditional_cause(
        &self,
    ) -> Option<&worth_runtime_bridge::facade::BridgeConditionalDenial> {
        match self.cause.as_deref()? {
            InstallationDenialCause::BridgeConditional(cause) => Some(cause),
            _ => None,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn with_bridge_conditional_cause(
        mut self,
        cause: worth_runtime_bridge::facade::BridgeConditionalDenial,
    ) -> Self {
        self.cause = Some(Box::new(InstallationDenialCause::BridgeConditional(cause)));
        self
    }
    pub fn bridge_build_cause(&self) -> Option<&worth_runtime_bridge::facade::BridgeBuildError> {
        match self.cause.as_deref()? {
            InstallationDenialCause::BridgeBuild(cause) => Some(cause),
            _ => None,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn with_bridge_build_cause(
        mut self,
        cause: worth_runtime_bridge::facade::BridgeBuildError,
    ) -> Self {
        self.cause = Some(Box::new(InstallationDenialCause::BridgeBuild(cause)));
        self
    }
    pub(in crate::domain_computation::primary_graph) fn with_managed_source_cause(
        mut self,
        cause: worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial,
    ) -> Self {
        self.cause = Some(Box::new(InstallationDenialCause::ManagedSource(cause)));
        self
    }
}
