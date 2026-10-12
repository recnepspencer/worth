use std::sync::OnceLock;

use worth_foundational::facade::{AspectContractRevision, AspectKey, FieldKey};
use worth_query::facade::consumer_kit::{in_memory_test_runtime, WorthQueryTestBackendSchema};
use worth_query::facade::{domain, foundation, read, runtime};

use super::super::installed_operation_fixture::{
    canonical_collection_bundle, operation_identity_contract, semantic_closure, GeometryDomain,
    ReadFamily,
};

#[derive(Clone, Copy, Debug)]
pub(super) struct CollectionRead;

pub(super) fn native_id_request<D, O, F, L: foundation::BasisOperationLane>(
    consumer: domain::WorthQueryConsumerProjectionContract<D, O, F, L>,
) -> (
    domain::WorthQueryBoundProjectionRequest<D, O, F, L>,
    domain::WorthQueryNativeAccessKey,
) {
    let mut builder = consumer.projection_request();
    let selection = builder.select_display_native_field(id_field()).unwrap();
    let request = builder.build().unwrap();
    let key = request.resolve_native_key(&selection).unwrap().into_key();
    (request, key)
}

impl domain::WorthQueryExecutableDomainOperation<GeometryDomain, ReadFamily> for CollectionRead {
    type Input = ();
    type Output = read::WorthQueryReadCompletion;
    type Publication = domain::WorthQueryPublishingOperation;
    type Execution = domain::WorthQueryDirectOperation;
}

#[derive(Clone, Copy)]
struct CollectionReadExecutor;

impl domain::WorthQueryDomainOperationExecutor<GeometryDomain, CollectionRead, ReadFamily>
    for CollectionReadExecutor
{
    const LOWERING_FAMILY: &'static str = "read-vertex-v1";
    const DETERMINISTIC: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;

    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
        Some(collection_read_declaration())
    }

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::super::installed_operation_fixture::execution_resource_support()
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

pub(super) fn collection_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> runtime::WorthQueryWorkspace {
    let mut semantics = semantic_closure(
        canonical_collection_bundle("Vertex"),
        domain::WorthQuerySupportRequirement::Required,
        true,
    );
    semantics.collection = domain::WorthQueryOperationCollectionContract::Collection {
        row_identity_field: domain::WorthQueryOperationCollectionField::from_dotted("identity.id")
            .expect("valid collection field"),
        ordering_fields: vec![domain::WorthQueryOperationCollectionField::from_dotted(
            "identity.id",
        )
        .expect("valid collection field")],
        grouping: domain::WorthQueryOperationGroupingContract::Ungrouped,
        window: domain::WorthQueryOperationWindowPolicy::CompleteCollection,
        continuation: domain::WorthQueryOperationContinuationPosture::NotRequired,
    };
    let operation = domain::WorthQueryDomainOperationDefinition::<
        GeometryDomain,
        CollectionRead,
        ReadFamily,
    >::new(
        domain::WorthQueryDomainOperationIdentity::new("native-collection-read", 1),
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
        .aspect_contract(operation_identity_contract(1))
        .unwrap()
        .aspect("identity.id", "identity.id")
        .unwrap();
    in_memory_test_runtime()
        .with_schema(schema)
        .domain_package(package)
        .domain_operation_executor(
            GeometryDomain,
            CollectionRead,
            ReadFamily,
            CollectionReadExecutor,
        )
        .workspace(request_execution, name)
        .unwrap()
}

fn collection_read_declaration() -> &'static read::WorthQueryReadDeclaration {
    static DECLARATION: OnceLock<read::WorthQueryReadDeclaration> = OnceLock::new();
    DECLARATION.get_or_init(|| {
        read::declare(|builder| {
            builder.local_collection(
                "Vertex",
                read::QuerySchemaView::new(
                    "installed-native-collection",
                    [read::SchemaFieldView::new(
                        read::AspectName::new("identity").unwrap(),
                        read::FieldName::new("id").unwrap(),
                        read::ScalarAspectType::String,
                    )],
                    [],
                ),
                |query| query.project(read::AspectFieldSelector::new("identity", "id").unwrap()),
                |shape| {
                    shape
                        .field(read::AuthoredResultShapeField::new("identity", "id", "id").unwrap())
                },
            )
        })
        .unwrap()
    })
}

pub(super) fn assert_request_denial(
    denial: &domain::WorthQueryNativeProjectionRequestDenial,
    expected_kind: domain::WorthQueryNativeProjectionRequestDenialKind,
    expected_field: Option<&FieldKey>,
) {
    assert_eq!(denial.kind(), expected_kind);
    assert_eq!(denial.contract_key(), &AspectKey::new("identity").unwrap());
    assert_eq!(denial.contract_revision(), AspectContractRevision(1));
    assert_eq!(denial.requested_field(), expected_field);
}

pub(super) fn id_field() -> FieldKey {
    FieldKey::new("id").unwrap()
}
