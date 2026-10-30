pub use worth_query::facade::{domain, read};

pub use super::super::GeometryDomain;
pub use super::material::{evidence_material, EvidenceScenario};
pub use super::{EvidenceFamily, EvidenceRead};

#[derive(Clone, Copy)]
pub struct EvidenceDirectExecutor {
    pub scenario: EvidenceScenario,
}

pub struct EvidenceGraphDirectExecutor(pub EvidenceDirectExecutor);

impl EvidenceGraphDirectExecutor {
    pub const fn honest() -> Self {
        Self(EvidenceDirectExecutor::new(EvidenceScenario::Honest))
    }
}

impl EvidenceDirectExecutor {
    pub const fn new(scenario: EvidenceScenario) -> Self {
        Self { scenario }
    }
}

impl domain::WorthQueryDomainOperationExecutor<GeometryDomain, EvidenceRead, EvidenceFamily>
    for EvidenceDirectExecutor
{
    const LOWERING_FAMILY: &'static str = "domain-evidence-read-v1";
    const DETERMINISTIC: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;

    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
        Some(super::super::executors::installed_read_declaration())
    }

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::super::execution_resource_support()
    }

    fn execute(
        &self,
        _: (),
        context: &domain::WorthQueryOperationExecutionContext<'_>,
        workspace: &mut domain::WorthQueryOperationWorkspace<'_>,
    ) -> Result<
        domain::WorthQueryOperationExecutionMaterial<read::WorthQueryReadCompletion>,
        domain::WorthQueryOperationExecutorFailure,
    > {
        let completion = context.execute_installed_read(workspace)?;
        let output_identity =
            domain::WorthQueryOperationOutput::operation_output_identity(&completion);
        Ok(domain::WorthQueryOperationExecutionMaterial::new(
            completion,
            domain::WorthQueryOperationResultState::Ready,
        )
        .with_domain_evidence(evidence_material(&output_identity, self.scenario)))
    }
}

impl domain::WorthQueryDomainOperationExecutor<GeometryDomain, EvidenceRead, EvidenceFamily>
    for EvidenceGraphDirectExecutor
{
    const LOWERING_FAMILY: &'static str = "domain-evidence-read-v1";
    const DETERMINISTIC: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::ExternalBoundary;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;

    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
        self.0.installed_read_declaration()
    }

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        self.0.execution_resource_support()
    }

    fn execute(
        &self,
        input: (),
        context: &domain::WorthQueryOperationExecutionContext<'_>,
        workspace: &mut domain::WorthQueryOperationWorkspace<'_>,
    ) -> Result<
        domain::WorthQueryOperationExecutionMaterial<read::WorthQueryReadCompletion>,
        domain::WorthQueryOperationExecutorFailure,
    > {
        self.0.execute(input, context, workspace)
    }
}
