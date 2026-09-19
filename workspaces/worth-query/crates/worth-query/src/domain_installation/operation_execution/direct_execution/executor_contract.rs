use crate::domain_installation::WorthQueryOperationResultState;
use crate::runtime::WorthQueryWorkspace;

pub trait WorthQueryOperationPublicationMode: 'static {
    const PUBLISHES: bool;
}

mod execution_mode_seal {
    pub trait Sealed {}
}

pub trait WorthQueryOperationExecutionMode: execution_mode_seal::Sealed + 'static {
    const IS_WORKFLOW: bool;
}

#[derive(Debug)]
pub struct WorthQueryDirectOperation;
impl execution_mode_seal::Sealed for WorthQueryDirectOperation {}
impl WorthQueryOperationExecutionMode for WorthQueryDirectOperation {
    const IS_WORKFLOW: bool = false;
}

#[derive(Debug)]
pub struct WorthQueryWorkflowOperation;
impl execution_mode_seal::Sealed for WorthQueryWorkflowOperation {}
impl WorthQueryOperationExecutionMode for WorthQueryWorkflowOperation {
    const IS_WORKFLOW: bool = true;
}

#[derive(Debug)]
pub struct WorthQueryPublishingOperation;
impl WorthQueryOperationPublicationMode for WorthQueryPublishingOperation {
    const PUBLISHES: bool = true;
}

#[derive(Debug)]
pub struct WorthQueryTerminalOperation;
impl WorthQueryOperationPublicationMode for WorthQueryTerminalOperation {
    const PUBLISHES: bool = false;
}

pub trait WorthQueryExecutableDomainOperation<D, F>: 'static {
    type Input: super::WorthQueryOperationInput;
    type Output: super::WorthQueryOperationOutput;
    type Publication: WorthQueryOperationPublicationMode;
    type Execution: WorthQueryOperationExecutionMode;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryOperationExecutionWarning {
    Advisory(String),
    Partial(String),
}

pub struct WorthQueryOperationExecutionMaterial<T> {
    output: T,
    result_state: WorthQueryOperationResultState,
    warnings: Vec<WorthQueryOperationExecutionWarning>,
    domain_evidence: Option<super::WorthQueryDomainEvidenceMaterial>,
}

impl<T> WorthQueryOperationExecutionMaterial<T> {
    pub fn new(output: T, result_state: WorthQueryOperationResultState) -> Self {
        Self {
            output,
            result_state,
            warnings: Vec::new(),
            domain_evidence: None,
        }
    }

    pub fn with_warning(mut self, warning: WorthQueryOperationExecutionWarning) -> Self {
        self.warnings.push(warning);
        self
    }

    pub fn with_domain_evidence(
        mut self,
        evidence: super::WorthQueryDomainEvidenceMaterial,
    ) -> Self {
        self.domain_evidence = Some(evidence);
        self
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        T,
        WorthQueryOperationResultState,
        Vec<WorthQueryOperationExecutionWarning>,
        Option<super::WorthQueryDomainEvidenceMaterial>,
    ) {
        (
            self.output,
            self.result_state,
            self.warnings,
            self.domain_evidence,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOperationExecutorFailure {
    class: worth_query_installation::facade::WorthQueryOperationFailureClass,
    detail: String,
}

impl WorthQueryOperationExecutorFailure {
    pub fn new(
        class: worth_query_installation::facade::WorthQueryOperationFailureClass,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            class,
            detail: detail.into(),
        }
    }

    pub fn class(&self) -> &worth_query_installation::facade::WorthQueryOperationFailureClass {
        &self.class
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

pub struct WorthQueryOperationExecutionContext<'a> {
    operation: &'a worth_query_installation::facade::WorthQueryPortableDomainOperationDefinition,
    binding_identity: &'a str,
    basis_identity: &'a str,
    basis: &'a crate::basis_lifecycle::NormalizedBasisIntent,
    installed_read: Option<&'a crate::ordinary::read::WorthQueryReadDeclaration>,
    graph_receipts: &'a [super::WorthQueryBoundGraphExecutionReceipt],
    resources: &'a super::WorthQueryAdmittedExecutionResourcePlan,
    provider_session_identity: &'a str,
}

/// The execution-scoped workspace surface available to a registered lowering.
/// Query retains the underlying workspace so lowering code cannot bypass the
/// installed operation contract with unrelated runtime commands.
pub struct WorthQueryOperationWorkspace<'a> {
    workspace: &'a mut WorthQueryWorkspace,
    installed_read_executions: usize,
}

impl<'a> WorthQueryOperationWorkspace<'a> {
    pub(crate) fn new(workspace: &'a mut WorthQueryWorkspace) -> Self {
        Self {
            workspace,
            installed_read_executions: 0,
        }
    }

    pub(crate) fn installed_read_executions(&self) -> usize {
        self.installed_read_executions
    }
}

impl<'a> WorthQueryOperationExecutionContext<'a> {
    pub(crate) fn new(
        operation: &'a worth_query_installation::facade::WorthQueryPortableDomainOperationDefinition,
        binding_identity: &'a str,
        basis_identity: &'a str,
        basis: &'a crate::basis_lifecycle::NormalizedBasisIntent,
        installed_read: Option<&'a crate::ordinary::read::WorthQueryReadDeclaration>,
        graph_receipts: &'a [super::WorthQueryBoundGraphExecutionReceipt],
        resources: &'a super::WorthQueryAdmittedExecutionResourcePlan,
        provider_session_identity: &'a str,
    ) -> Self {
        Self {
            operation,
            binding_identity,
            basis_identity,
            basis,
            installed_read,
            graph_receipts,
            resources,
            provider_session_identity,
        }
    }

    pub fn operation(
        &self,
    ) -> &worth_query_installation::facade::WorthQueryPortableDomainOperationDefinition {
        self.operation
    }

    pub fn binding_identity(&self) -> &str {
        self.binding_identity
    }

    pub fn basis_identity(&self) -> &str {
        self.basis_identity
    }

    pub fn basis(&self) -> &crate::basis_lifecycle::NormalizedBasisIntent {
        self.basis
    }

    pub fn resources(&self) -> &super::WorthQueryAdmittedExecutionResourcePlan {
        self.resources
    }

    pub fn provider_session_identity(&self) -> &str {
        self.provider_session_identity
    }

    pub fn graph_projection(
        &self,
        role: &str,
    ) -> Option<&super::WorthQueryExecutionGraphReadProduct> {
        self.graph_receipts
            .iter()
            .find(|receipt| {
                receipt.role() == role
                    && receipt.kind()
                        == crate::domain_installation::WorthQueryGraphProviderCallKind::Project
            })
            .and_then(super::WorthQueryBoundGraphExecutionReceipt::graph_read_product)
    }

    pub(crate) fn has_installed_read(&self) -> bool {
        self.installed_read.is_some()
    }

    pub fn execute_installed_read(
        &self,
        workspace: &mut WorthQueryOperationWorkspace<'_>,
    ) -> Result<crate::ordinary::read::WorthQueryReadCompletion, WorthQueryOperationExecutorFailure>
    {
        let declaration = self.installed_read.ok_or_else(|| {
            WorthQueryOperationExecutorFailure::new(
                crate::domain_installation::WorthQueryOperationFailureClass::Indeterminate,
                "operation executor has no Query-installed read declaration",
            )
        })?;
        if workspace.installed_read_executions != 0 {
            return Err(WorthQueryOperationExecutorFailure::new(
                crate::domain_installation::WorthQueryOperationFailureClass::Indeterminate,
                "installed canonical read may execute only once per operation",
            ));
        }
        workspace.installed_read_executions += 1;
        declaration
            .clone_for_installed_execution()
            .using(crate::ordinary::read::current())
            .run(workspace.workspace)
            .into_result()
            .map_err(|stop| {
                WorthQueryOperationExecutorFailure::new(
                    crate::domain_installation::WorthQueryOperationFailureClass::Dependency,
                    format!("{stop:?}"),
                )
            })
    }
}

pub trait WorthQueryDomainOperationExecutor<D, O, F>: Send + Sync + 'static
where
    O: WorthQueryExecutableDomainOperation<D, F>,
{
    const LOWERING_FAMILY: &'static str;
    const DETERMINISTIC: bool;
    const EXECUTION_COST: crate::domain_installation::WorthQueryOperationCostClass;
    const RESULT_WIDTH_COST: crate::domain_installation::WorthQueryOperationCostClass;

    fn installed_read_declaration(
        &self,
    ) -> Option<&crate::ordinary::read::WorthQueryReadDeclaration> {
        None
    }

    fn execution_resource_support(
        &self,
    ) -> crate::domain_installation::WorthQueryExecutionResourceSupport;

    fn execute(
        &self,
        input: O::Input,
        context: &WorthQueryOperationExecutionContext<'_>,
        workspace: &mut WorthQueryOperationWorkspace<'_>,
    ) -> Result<WorthQueryOperationExecutionMaterial<O::Output>, WorthQueryOperationExecutorFailure>;
}
