use worth_query::facade::consumer_kit::{
    in_memory_test_runtime, WorthQueryInMemoryTestRuntimeBuilder, WorthQueryTestBackendSchema,
};
use worth_query::facade::{domain, foundation};

use super::super::super::conditional_node_contract::identity_contract;
use super::super::super::installed_operation_fixture::{
    canonical_bundle, semantic_closure, GeometryDomain, ReadFamily,
};

#[derive(Clone, Copy, Debug)]
pub(in super::super) struct CompatibilityNoPrimaryRead;

impl domain::WorthQueryExecutableDomainOperation<GeometryDomain, ReadFamily>
    for CompatibilityNoPrimaryRead
{
    type Input = ();
    type Output = u64;
    type Publication = domain::WorthQueryTerminalOperation;
    type Execution = domain::WorthQueryDirectOperation;
}

#[derive(Clone, Copy)]
struct CompatibilityNoPrimaryReadExecutor;

impl
    domain::WorthQueryDomainOperationExecutor<
        GeometryDomain,
        CompatibilityNoPrimaryRead,
        ReadFamily,
    > for CompatibilityNoPrimaryReadExecutor
{
    const LOWERING_FAMILY: &'static str = "read-vertex-v1";
    const DETERMINISTIC: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::super::super::installed_operation_fixture::execution_resource_support()
    }

    fn execute(
        &self,
        _: (),
        _: &domain::WorthQueryOperationExecutionContext<'_>,
        _: &mut domain::WorthQueryOperationWorkspace<'_>,
    ) -> Result<
        domain::WorthQueryOperationExecutionMaterial<u64>,
        domain::WorthQueryOperationExecutorFailure,
    > {
        Ok(domain::WorthQueryOperationExecutionMaterial::new(
            0,
            domain::WorthQueryOperationResultState::Ready,
        ))
    }
}

pub(in super::super) type BoundRead = domain::WorthQueryBoundDomainOperation<
    GeometryDomain,
    CompatibilityNoPrimaryRead,
    ReadFamily,
    foundation::ObservationLaneWitness,
>;

pub(in super::super) fn no_primary_read_runtime() -> WorthQueryInMemoryTestRuntimeBuilder {
    let mut semantics = semantic_closure(
        canonical_bundle("Vertex"),
        domain::WorthQuerySupportRequirement::NotRequired,
        false,
    );
    semantics.graph_reads = domain::WorthQueryOperationGraphReadContract::NotRequired;
    let operation = domain::WorthQueryDomainOperationDefinition::<
        GeometryDomain,
        CompatibilityNoPrimaryRead,
        ReadFamily,
    >::new(
        domain::WorthQueryDomainOperationIdentity::new("compatibility-no-primary-read", 1),
        semantics,
    );
    let package = domain::WorthQueryDomainPackage::declare(
        GeometryDomain,
        domain::WorthQueryDomainIdentityDeclaration::new(
            domain::WorthQueryDomainIdentityNamespace::new("WORTH.tests").unwrap(),
            domain::WorthQueryDomainIdentityName::new("geometry").unwrap(),
            domain::WorthQueryDomainSemanticVersion::new(1, 0),
        ),
    )
    .operation(operation);
    let schema = WorthQueryTestBackendSchema::single_collection("Vertex")
        .aspect_contract(identity_contract())
        .unwrap()
        .aspect("identity.id", "identity.id")
        .unwrap();
    in_memory_test_runtime()
        .with_schema(schema)
        .domain_package(package)
        .domain_operation_executor(
            GeometryDomain,
            CompatibilityNoPrimaryRead,
            ReadFamily,
            CompatibilityNoPrimaryReadExecutor,
        )
}

pub(in super::super) fn bind_current(
    workspace: &worth_query::facade::runtime::WorthQueryWorkspace,
    installed: &domain::WorthQueryInstalledDomainHandle<GeometryDomain>,
) -> BoundRead {
    workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(installed, CompatibilityNoPrimaryRead)
        .unwrap()
}

pub(in super::super) fn bind_branch(
    workspace: &worth_query::facade::runtime::WorthQueryWorkspace,
    installed: &domain::WorthQueryInstalledDomainHandle<GeometryDomain>,
) -> BoundRead {
    let branch = workspace
        .branches()
        .fork(workspace.current_world())
        .components(|components| components.reuse_exact_relational_basis().fork_signal())
        .create()
        .unwrap();
    workspace
        .observe_operating_world(branch)
        .unwrap()
        .family(ReadFamily)
        .bind(installed, CompatibilityNoPrimaryRead)
        .unwrap()
}
