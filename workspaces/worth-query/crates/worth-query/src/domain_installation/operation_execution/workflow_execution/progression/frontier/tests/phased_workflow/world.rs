use worth_foundational::facade::{
    AbsenceLaw, AspectContract, AspectContractRevision, AspectEvolutionPolicy, AspectIdentity,
    AspectKey, AspectMask, FieldDeclaration, FieldKey, FieldRequirement, ProjectionMask,
    ScalarAspectType, StructAspectShape,
};
use worth_query::facade::domain;
use worth_query_declaration::facade::authoring::{
    AspectFieldSelector, AuthoredQueryBundleRequest, AuthoredResultShapeField, DetailQueryBuilder,
    DetailResultShapeBuilder, RootEntityKey,
};
use worth_query_declaration::facade::binding::QueryBindingDescriptor;
use worth_query_declaration::facade::canonicalization::canonicalize_request;

use std::sync::OnceLock;
use worth_query::facade::{consumer_kit, read};

pub(super) fn builder() -> consumer_kit::WorthQueryInMemoryTestRuntimeBuilder {
    let schema = consumer_kit::WorthQueryTestBackendSchema::single_collection("Vertex")
        .aspect_contract(operation_identity_contract(1))
        .unwrap()
        .aspect("identity.id", "identity.id")
        .unwrap();
    consumer_kit::in_memory_test_runtime()
        .with_schema(schema)
        .domain_package(super::definition::package())
}

pub(super) fn installed_read_declaration() -> &'static read::WorthQueryReadDeclaration {
    static DECLARATION: OnceLock<read::WorthQueryReadDeclaration> = OnceLock::new();
    DECLARATION.get_or_init(|| {
        read::declare(|builder| {
            builder.local_detail(
                "Vertex",
                read::QuerySchemaView::new(
                    "workflow-fold",
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
pub(crate) fn semantic_closure(
    bundle: worth_query_declaration::facade::canonicalization::CanonicalQueryBundle,
    projection_consumption: domain::WorthQuerySupportRequirement,
    publishes: bool,
) -> domain::WorthQueryDomainOperationSemanticClosure {
    let native_projection = domain::WorthQueryOperationNativeProjectionContract::new(
        operation_identity_contract(1),
        AspectMask::<ProjectionMask>::whole_aspect(),
    )
    .unwrap();
    domain::WorthQueryDomainOperationSemanticClosure {
        parameters: domain::WorthQueryOperationParameterContract::NotRequired,
        native_projection: native_projection.clone(),
        canonical_query: bundle,
        collection: domain::WorthQueryOperationCollectionContract::NotCollection,
        required_capabilities: Vec::new(),
        required_domains: Vec::new(),
        workflow: domain::WorthQueryOperationWorkflowContract::NotRequired,
        evidence: domain::WorthQueryDomainEvidenceContract::not_required(),
        conditional_nodes: Vec::new(),
        graph_reads: domain::WorthQueryOperationGraphReadContract::DeclaredDomain {
            roles: vec![domain::WorthQueryDomainOperationGraphReadRole {
                role: "model".into(),
                participation: domain::WorthQueryOperationGraphParticipation::PrimaryLogicalGraph,
                access: domain::WorthQueryOperationGraphAccess::Project,
                semantic_reads: vec![native_projection],
            }],
        },
        decision_facts: domain::WorthQueryOperationDecisionFactContract::NotRequired,
        touches: domain::WorthQueryOperationTouchContract::NotRequired,
        effects: domain::WorthQueryOperationEffectContract::NotRequired,
        invariants: domain::WorthQueryOperationInvariantContract::NotRequired,
        invariant_execution: domain::WorthQueryInvariantExecutionContract::NotRequired,
        replay: domain::WorthQueryOperationReplayContract::ReExecutable,
        aftermath: None,
        lineage: domain::WorthQueryOperationLineageContract::NotRequired,
        promotion: domain::WorthQueryOperationPromotionContract::NotRequired,
        publication: if publishes {
            domain::WorthQueryOperationPublicationContract::DerivedProjection {
                projection_role: domain::WorthQueryOperationProjectionRole::new("vertex").unwrap(),
            }
        } else {
            domain::WorthQueryOperationPublicationContract::NotRequired
        },
        projection_consumption: if publishes {
            domain::WorthQueryOperationProjectionConsumptionContract::QueryReadAuthority
        } else {
            domain::WorthQueryOperationProjectionConsumptionContract::NotRequired
        },
        terminal: domain::WorthQueryOperationTerminalContract {
            result_states: vec![
                domain::WorthQueryOperationResultState::Ready,
                domain::WorthQueryOperationResultState::Advisory,
                domain::WorthQueryOperationResultState::Pending,
                domain::WorthQueryOperationResultState::Partial,
                domain::WorthQueryOperationResultState::Violation,
            ],
            failure_classes: vec![domain::WorthQueryOperationFailureClass::Dependency],
        },
        cost: domain::WorthQueryOperationCostContract {
            lookup: domain::WorthQueryOperationCostClass::Constant,
            execution: domain::WorthQueryOperationCostClass::DeclaredWidth,
            result_width: domain::WorthQueryOperationCostClass::DeclaredWidth,
        },
        resources: execution_resource_contract(),
        support: domain::WorthQueryOperationSupportRequirements {
            live: domain::WorthQuerySupportRequirement::NotRequired,
            continuation: domain::WorthQuerySupportRequirement::NotRequired,
            async_result_state: domain::WorthQuerySupportRequirement::NotRequired,
            recovery: domain::WorthQuerySupportRequirement::NotRequired,
            inspection: domain::WorthQuerySupportRequirement::NotRequired,
            projection_consumption,
            dependency_impact: domain::WorthQuerySupportRequirement::NotRequired,
            sharing: domain::WorthQuerySupportRequirement::NotRequired,
            invalidation: domain::WorthQuerySupportRequirement::NotRequired,
            collection_delivery: domain::WorthQuerySupportRequirement::NotRequired,
            conditional_evaluation: domain::WorthQuerySupportRequirement::NotRequired,
            conditional_comparator: domain::WorthQuerySupportRequirement::NotRequired,
            conditional_trigger: domain::WorthQuerySupportRequirement::NotRequired,
            conditional_temporal_or_on_demand: domain::WorthQuerySupportRequirement::NotRequired,
        },
        lowering: domain::WorthQueryOperationLoweringContract {
            family: "read-vertex-v1".into(),
            deterministic: true,
        },
    }
}

pub(crate) fn execution_resource_request() -> domain::WorthQueryExecutionResourceRequest {
    domain::WorthQueryExecutionResourceRequest::bounded(
        1_000_000,
        1_000_000,
        cancellation_safe_point(),
    )
}

pub(crate) fn execution_resource_contract() -> domain::WorthQueryExecutionResourceContract {
    execution_resource_contract_for(execution_resource_envelope())
}

fn execution_resource_contract_for(
    envelope: domain::WorthQueryExecutionResourceEnvelope,
) -> domain::WorthQueryExecutionResourceContract {
    domain::WorthQueryExecutionResourceContract::declared([
        domain::WorthQueryExecutionStrategyContract::new(
            domain::WorthQueryExecutionStrategyName::new("fixture-bounded").unwrap(),
            envelope,
            domain::WorthQueryExecutionProviderRequirements::new(
                domain::WorthQueryExecutionProviderFamily::new("fixture-provider").unwrap(),
                domain::WorthQueryExecutionAccessProductFamily::new("fixture-access").unwrap(),
                domain::WorthQueryExecutionAllocatorFamily::new("fixture-arena").unwrap(),
            ),
        ),
    ])
    .unwrap()
}

pub(crate) fn execution_resource_support() -> domain::WorthQueryExecutionResourceSupport {
    execution_resource_support_for(execution_resource_envelope())
}

fn execution_resource_support_for(
    envelope: domain::WorthQueryExecutionResourceEnvelope,
) -> domain::WorthQueryExecutionResourceSupport {
    domain::WorthQueryExecutionResourceSupport::new(
        domain::WorthQueryExecutionProviderFamily::new("fixture-provider").unwrap(),
        domain::WorthQueryExecutionAccessProductFamily::new("fixture-access").unwrap(),
        domain::WorthQueryExecutionAllocatorFamily::new("fixture-arena").unwrap(),
        envelope,
        std::sync::Arc::new(
            domain::WorthQueryFixedExecutionCapacity::mint("fixture-provider", 1_000_000).unwrap(),
        ),
    )
}

fn execution_resource_envelope() -> domain::WorthQueryExecutionResourceEnvelope {
    domain::WorthQueryExecutionResourceEnvelope::bounded(
        1_000_000,
        1_000_000,
        domain::WorthQueryExecutionMode::Synchronous,
        cancellation_safe_point(),
    )
}

fn cancellation_safe_point() -> domain::WorthQueryCancellationSafePointFamily {
    domain::WorthQueryCancellationSafePointFamily::new("fixture-chunk-boundary").unwrap()
}

pub(crate) fn operation_identity_contract(revision: u64) -> AspectContract {
    let id = FieldDeclaration::new(
        FieldKey::new("id").unwrap(),
        ScalarAspectType::String,
        FieldRequirement::Required,
        AbsenceLaw::Required,
        AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .unwrap();
    AspectContract::struct_aspect(
        AspectKey::new("identity").unwrap(),
        AspectIdentity(0x9140_0001),
        AspectContractRevision(revision),
        StructAspectShape::new([id]).unwrap(),
    )
}

pub(crate) fn canonical_bundle(
    root: &str,
) -> worth_query_declaration::facade::canonicalization::CanonicalQueryBundle {
    let selector = AspectFieldSelector::new("identity", "id").unwrap();
    let query = DetailQueryBuilder::new(RootEntityKey::new(root).unwrap())
        .project(selector)
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(AuthoredResultShapeField::new("identity", "id", "id").unwrap())
        .build()
        .unwrap()
        .into_raw();
    canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new())
            .unwrap(),
    )
    .unwrap()
}
