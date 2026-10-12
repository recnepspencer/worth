use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use worth_query::facade::{domain, read, runtime};

use super::{GeometryDomain, ReadExecutionInput, ReadFamily, ReadVertex};

pub(crate) fn resource_admission_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
    contract: domain::WorthQueryExecutionResourceContract,
    support: domain::WorthQueryExecutionResourceSupport,
) -> Result<
    (runtime::WorthQueryWorkspace, Arc<AtomicUsize>),
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    let base = super::read_vertex_definition(domain::WorthQuerySupportRequirement::Required);
    let mut semantics = base.semantics().clone();
    semantics.resources = contract;
    let operation = domain::WorthQueryDomainOperationDefinition::<
        GeometryDomain,
        ReadVertex,
        ReadFamily,
    >::new(base.identity().clone(), semantics);
    let package = domain::WorthQueryDomainPackage::declare(
        GeometryDomain,
        domain::WorthQueryDomainIdentityDeclaration::new(
            domain::WorthQueryDomainIdentityNamespace::new("WORTH.tests").unwrap(),
            domain::WorthQueryDomainIdentityName::new("geometry").unwrap(),
            domain::WorthQueryDomainSemanticVersion::new(1, 0),
        ),
    )
    .operation(operation);
    let contacts = Arc::new(AtomicUsize::new(0));
    let executor = ResourceAdmissionExecutor {
        support,
        contacts: Arc::clone(&contacts),
    };
    let workspace = super::configured_runtime_without_executors(package)
        .domain_operation_executor(GeometryDomain, ReadVertex, ReadFamily, executor)
        .workspace(request_execution, name)?;
    Ok((workspace, contacts))
}

struct ResourceAdmissionExecutor {
    support: domain::WorthQueryExecutionResourceSupport,
    contacts: Arc<AtomicUsize>,
}

impl domain::WorthQueryDomainOperationExecutor<GeometryDomain, ReadVertex, ReadFamily>
    for ResourceAdmissionExecutor
{
    const LOWERING_FAMILY: &'static str = "read-vertex-v1";
    const DETERMINISTIC: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        self.support.clone()
    }

    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
        <super::ReadVertexExecutor as domain::WorthQueryDomainOperationExecutor<
            GeometryDomain,
            ReadVertex,
            ReadFamily,
        >>::installed_read_declaration(&super::ReadVertexExecutor)
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
        self.contacts.fetch_add(1, Ordering::SeqCst);
        <super::ReadVertexExecutor as domain::WorthQueryDomainOperationExecutor<
            GeometryDomain,
            ReadVertex,
            ReadFamily,
        >>::execute(&super::ReadVertexExecutor, input, context, workspace)
    }
}
