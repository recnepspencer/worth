pub use worth_query::facade::{domain, read};

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct UnrelatedDomain;

#[derive(Clone, Copy)]
pub struct UnrelatedOperation;

#[derive(Clone, Copy)]
pub struct UnrelatedFamily;

impl domain::WorthQueryDomainEntryMarker for UnrelatedDomain {
    fn domain_key(&self) -> &'static str {
        "WORTH.certification.unrelated-query"
    }

    fn display_name(&self) -> &'static str {
        "Unrelated query"
    }

    fn required_capability_families(&self) -> &'static [domain::WorthQueryCapabilityFamily] {
        &[]
    }
}

impl domain::WorthQueryExecutableDomainOperation<UnrelatedDomain, UnrelatedFamily>
    for UnrelatedOperation
{
    type Input = ();
    type Output = read::WorthQueryReadCompletion;
    type Publication = domain::WorthQueryPublishingOperation;
    type Execution = domain::WorthQueryDirectOperation;
}

pub fn unrelated_package() -> domain::WorthQueryDomainPackage<UnrelatedDomain> {
    let base = super::consumer_operation_definition(super::ConsumerProfile::ValuePatch);
    let mut semantics = base.semantics().clone();
    semantics.conditional_nodes.clear();
    domain::WorthQueryDomainPackage::declare(
        UnrelatedDomain,
        domain::WorthQueryDomainIdentityDeclaration::new(
            domain::WorthQueryDomainIdentityNamespace::new("WORTH.certification").unwrap(),
            domain::WorthQueryDomainIdentityName::new("unrelated-query").unwrap(),
            domain::WorthQueryDomainSemanticVersion::new(1, 0),
        ),
    )
    .operation(domain::WorthQueryDomainOperationDefinition::<
        UnrelatedDomain,
        UnrelatedOperation,
        UnrelatedFamily,
    >::new(base.identity().clone(), semantics))
}

pub struct UnrelatedExecutor;

impl domain::WorthQueryDomainOperationExecutor<UnrelatedDomain, UnrelatedOperation, UnrelatedFamily>
    for UnrelatedExecutor
{
    const LOWERING_FAMILY: &'static str = "temporal-host-courtroom-v1";
    const DETERMINISTIC: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::Constant;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::Constant;

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::resource_support()
    }

    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
        Some(super::detail_read_declaration())
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
        Ok(domain::WorthQueryOperationExecutionMaterial::new(
            context.execute_installed_read(workspace)?,
            domain::WorthQueryOperationResultState::Ready,
        ))
    }
}
