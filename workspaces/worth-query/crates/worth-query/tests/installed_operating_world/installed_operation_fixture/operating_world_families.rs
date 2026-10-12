use std::marker::PhantomData;

use worth_query::facade::{consumer_kit, domain, read, runtime};

use super::{
    configured_runtime_without_executors, read_vertex_definition, AuxiliaryDomain, GeometryDomain,
    ReadExecutionInput,
};

#[derive(Clone, Copy, Debug)]
pub struct ConstructOperation;
#[derive(Clone, Copy, Debug)]
pub struct ConstructFamily;
#[derive(Clone, Copy, Debug)]
pub struct BooleanOperation;
#[derive(Clone, Copy, Debug)]
pub struct BooleanFamily;
#[derive(Clone, Copy, Debug)]
pub struct TransformOperation;
#[derive(Clone, Copy, Debug)]
pub struct TransformFamily;
#[derive(Clone, Copy, Debug)]
pub struct RouteOperation;
#[derive(Clone, Copy, Debug)]
pub struct RouteFamily;

macro_rules! executable_read_operation {
    ($operation:ty, $family:ty) => {
        impl domain::WorthQueryExecutableDomainOperation<GeometryDomain, $family> for $operation {
            type Input = ReadExecutionInput;
            type Output = read::WorthQueryReadCompletion;
            type Publication = domain::WorthQueryPublishingOperation;
            type Execution = domain::WorthQueryDirectOperation;
        }
    };
}

executable_read_operation!(ConstructOperation, ConstructFamily);
executable_read_operation!(BooleanOperation, BooleanFamily);
executable_read_operation!(TransformOperation, TransformFamily);
executable_read_operation!(RouteOperation, RouteFamily);

#[derive(Clone, Copy)]
struct FamilyReadExecutor<O, F>(PhantomData<fn() -> (O, F)>);

impl<O, F> FamilyReadExecutor<O, F> {
    const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<O, F> domain::WorthQueryDomainOperationExecutor<GeometryDomain, O, F>
    for FamilyReadExecutor<O, F>
where
    O: domain::WorthQueryExecutableDomainOperation<
            GeometryDomain,
            F,
            Input = ReadExecutionInput,
            Output = read::WorthQueryReadCompletion,
        > + 'static,
    F: 'static,
{
    const LOWERING_FAMILY: &'static str = "read-vertex-v1";
    const DETERMINISTIC: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;

    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
        Some(super::executors::installed_read_declaration())
    }

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::execution_resource_support()
    }

    fn execute(
        &self,
        input: ReadExecutionInput,
        context: &domain::WorthQueryOperationExecutionContext<'_>,
        workspace: &mut domain::WorthQueryOperationWorkspace<'_>,
    ) -> Result<
        domain::WorthQueryOperationExecutionMaterial<read::WorthQueryReadCompletion>,
        domain::WorthQueryOperationExecutorFailure,
    > {
        if let Some(class) = input.failure {
            return Err(domain::WorthQueryOperationExecutorFailure::new(
                class,
                "deliberate single-root family executor failure",
            ));
        }
        let completion = context.execute_installed_read(workspace)?;
        let material = domain::WorthQueryOperationExecutionMaterial::new(completion, input.state);
        Ok(match input.warning {
            Some(warning) => material.with_warning(warning),
            None => material,
        })
    }
}

pub fn operating_world_family_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> Result<runtime::WorthQueryWorkspace, consumer_kit::WorthQueryTestBackendError> {
    let base = read_vertex_definition(domain::WorthQuerySupportRequirement::Required);
    let ordinary_semantics = base.semantics().clone();
    let mut route_semantics = ordinary_semantics.clone();
    route_semantics
        .required_domains
        .push(domain::WorthQueryOperationRequiredDomainRole::new("auxiliary").unwrap());
    let geometry = domain::WorthQueryDomainPackage::declare(
        GeometryDomain,
        domain_identity::<GeometryDomain>("geometry"),
    )
    .operation(definition::<ConstructOperation, ConstructFamily>(
        "construct",
        ordinary_semantics.clone(),
    ))
    .operation(definition::<BooleanOperation, BooleanFamily>(
        "boolean",
        ordinary_semantics.clone(),
    ))
    .operation(definition::<TransformOperation, TransformFamily>(
        "transform",
        ordinary_semantics,
    ))
    .operation(definition::<RouteOperation, RouteFamily>(
        "route",
        route_semantics,
    ))
    .operation_required_domain::<RouteOperation, RouteFamily, AuxiliaryDomain>("auxiliary");
    configured_runtime_without_executors(geometry)
        .domain_operation_executor(
            GeometryDomain,
            ConstructOperation,
            ConstructFamily,
            FamilyReadExecutor::new(),
        )
        .domain_operation_executor(
            GeometryDomain,
            BooleanOperation,
            BooleanFamily,
            FamilyReadExecutor::new(),
        )
        .domain_operation_executor(
            GeometryDomain,
            TransformOperation,
            TransformFamily,
            FamilyReadExecutor::new(),
        )
        .domain_operation_executor(
            GeometryDomain,
            RouteOperation,
            RouteFamily,
            FamilyReadExecutor::new(),
        )
        .domain_package(domain::WorthQueryDomainPackage::declare(
            AuxiliaryDomain,
            domain_identity::<AuxiliaryDomain>("auxiliary"),
        ))
        .workspace(request_execution, name)
}

fn definition<O, F>(
    identity: &str,
    semantics: domain::WorthQueryDomainOperationSemanticClosure,
) -> domain::WorthQueryDomainOperationDefinition<GeometryDomain, O, F> {
    domain::WorthQueryDomainOperationDefinition::new(
        domain::WorthQueryDomainOperationIdentity::new(identity, 1),
        semantics,
    )
}

fn domain_identity<D>(name: &str) -> domain::WorthQueryDomainIdentityDeclaration<D> {
    domain::WorthQueryDomainIdentityDeclaration::new(
        domain::WorthQueryDomainIdentityNamespace::new("WORTH.tests").unwrap(),
        domain::WorthQueryDomainIdentityName::new(name).unwrap(),
        domain::WorthQueryDomainSemanticVersion::new(1, 0),
    )
}
