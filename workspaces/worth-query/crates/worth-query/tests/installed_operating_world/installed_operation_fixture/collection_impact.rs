use std::sync::OnceLock;

use worth_query::facade::{domain, read, runtime};

use super::{
    canonical_ordered_collection_bundle, configured_runtime_without_executors,
    configured_runtime_without_executors_with_schema, semantic_closure, GeometryDomain, ReadFamily,
};

mod routing_contract;

use routing_contract::{
    routing_collection_read_declaration, routing_collection_schema, routing_collection_semantics,
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct ImpactCollectionRead;

impl domain::WorthQueryExecutableDomainOperation<GeometryDomain, ReadFamily>
    for ImpactCollectionRead
{
    type Input = ();
    type Output = read::WorthQueryReadCompletion;
    type Publication = domain::WorthQueryPublishingOperation;
    type Execution = domain::WorthQueryDirectOperation;
}

#[derive(Clone, Copy)]
struct ImpactCollectionExecutor {
    routing_order: bool,
}

impl ImpactCollectionExecutor {
    const fn identity_ordered() -> Self {
        Self {
            routing_order: false,
        }
    }

    const fn routing_ordered() -> Self {
        Self {
            routing_order: true,
        }
    }
}

impl domain::WorthQueryDomainOperationExecutor<GeometryDomain, ImpactCollectionRead, ReadFamily>
    for ImpactCollectionExecutor
{
    const LOWERING_FAMILY: &'static str = "read-vertex-v1";
    const DETERMINISTIC: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;

    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
        Some(if self.routing_order {
            routing_collection_read_declaration()
        } else {
            collection_read_declaration()
        })
    }

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::execution_resource_support()
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

pub(crate) fn impact_collection_workspace(
    name: &str,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    configured_runtime_without_executors(plain_collection_package())
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::Continuation,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .domain_operation_executor(
            GeometryDomain,
            ImpactCollectionRead,
            ReadFamily,
            ImpactCollectionExecutor::identity_ordered(),
        )
        .workspace(name)
}

pub(crate) fn impact_collection_invalidation_workspace(
    name: &str,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    configured_runtime_without_executors_with_schema(
        package(routing_collection_semantics()),
        routing_collection_schema(),
    )
    .consumer_support_posture(
        domain::WorthQueryConsumerSupportDimension::Continuation,
        domain::WorthQueryConsumerSupportPosture::Supported,
    )
    .consumer_support_posture(
        domain::WorthQueryConsumerSupportDimension::Sharing,
        domain::WorthQueryConsumerSupportPosture::Supported,
    )
    .consumer_support_posture(
        domain::WorthQueryConsumerSupportDimension::Invalidation,
        domain::WorthQueryConsumerSupportPosture::Supported,
    )
    .consumer_support_posture(
        domain::WorthQueryConsumerSupportDimension::DependencyImpact,
        domain::WorthQueryConsumerSupportPosture::Supported,
    )
    .domain_operation_executor(
        GeometryDomain,
        ImpactCollectionRead,
        ReadFamily,
        ImpactCollectionExecutor::routing_ordered(),
    )
    .workspace(name)
}

fn plain_collection_package() -> domain::WorthQueryDomainPackage<GeometryDomain> {
    package(collection_semantics())
}

fn collection_semantics() -> domain::WorthQueryDomainOperationSemanticClosure {
    let mut semantics = semantic_closure(
        canonical_ordered_collection_bundle("Vertex", "identity", "id"),
        domain::WorthQuerySupportRequirement::Required,
        true,
    );
    semantics.collection = domain::WorthQueryOperationCollectionContract::Collection {
        row_identity_field: field(),
        ordering_fields: vec![field()],
        grouping: domain::WorthQueryOperationGroupingContract::Grouped {
            grouping_fields: vec![field()],
        },
        window: domain::WorthQueryOperationWindowPolicy::ContinuationBounded,
        continuation: domain::WorthQueryOperationContinuationPosture::SnapshotCursor,
    };
    semantics.support.continuation = domain::WorthQuerySupportRequirement::Required;
    semantics
}

fn package(
    semantics: domain::WorthQueryDomainOperationSemanticClosure,
) -> domain::WorthQueryDomainPackage<GeometryDomain> {
    let operation = domain::WorthQueryDomainOperationDefinition::<
        GeometryDomain,
        ImpactCollectionRead,
        ReadFamily,
    >::new(
        domain::WorthQueryDomainOperationIdentity::new("impact-collection-read", 1),
        semantics,
    );
    domain::WorthQueryDomainPackage::declare(
        GeometryDomain,
        domain::WorthQueryDomainIdentityDeclaration::new(
            domain::WorthQueryDomainIdentityNamespace::new("WORTH.tests").unwrap(),
            domain::WorthQueryDomainIdentityName::new("geometry").unwrap(),
            domain::WorthQueryDomainSemanticVersion::new(1, 0),
        ),
    )
    .operation(operation)
}

fn field() -> domain::WorthQueryOperationCollectionField {
    domain::WorthQueryOperationCollectionField::from_dotted("identity.id")
        .expect("valid collection field")
}

fn collection_read_declaration() -> &'static read::WorthQueryReadDeclaration {
    static DECLARATION: OnceLock<read::WorthQueryReadDeclaration> = OnceLock::new();
    DECLARATION.get_or_init(|| {
        read::declare(|builder| {
            builder.local_collection(
                "Vertex",
                read::QuerySchemaView::new(
                    "impact-collection",
                    [
                        read::SchemaFieldView::new(
                            read::AspectName::new("identity").unwrap(),
                            read::FieldName::new("id").unwrap(),
                            read::ScalarAspectType::String,
                        ),
                        read::SchemaFieldView::new(
                            read::AspectName::new("ordering").unwrap(),
                            read::FieldName::new("position").unwrap(),
                            read::ScalarAspectType::String,
                        ),
                    ],
                    [],
                ),
                |query| {
                    query
                        .project(read::AspectFieldSelector::new("identity", "id").unwrap())
                        .order_by(read::OrderingSelector::ascending("identity", "id").unwrap())
                },
                |shape| {
                    shape
                        .field(read::AuthoredResultShapeField::new("identity", "id", "id").unwrap())
                },
            )
        })
        .expect("collection declaration is canonical")
    })
}
