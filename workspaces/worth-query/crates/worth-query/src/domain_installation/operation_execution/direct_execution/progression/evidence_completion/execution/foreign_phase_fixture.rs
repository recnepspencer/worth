use crate::facade::{domain, read};
use std::sync::OnceLock;
use worth_foundational::facade::{
    AbsenceLaw, AspectContract, AspectContractRevision, AspectEvolutionPolicy, AspectIdentity,
    AspectKey, AspectMask, FieldDeclaration, FieldKey, FieldRequirement, ProjectionMask,
    ScalarAspectType, StructAspectShape,
};
use worth_query_declaration::facade::authoring::{
    AspectFieldSelector, AuthoredQueryBundleRequest, AuthoredResultShapeField, DetailQueryBuilder,
    DetailResultShapeBuilder, RootEntityKey,
};
use worth_query_declaration::facade::binding::QueryBindingDescriptor;
use worth_query_declaration::facade::canonicalization::canonicalize_request;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct DirectDomain;
#[derive(Clone, Copy, Debug)]
pub(super) struct DirectRead;
#[derive(Clone, Copy, Debug)]
pub(super) struct ReadFamily;
impl domain::WorthQueryDomainEntryMarker for DirectDomain {
    fn domain_key(&self) -> &'static str {
        "WORTH.tests.direct-custody"
    }
    fn display_name(&self) -> &'static str {
        "Direct custody"
    }
    fn required_capability_families(&self) -> &'static [domain::WorthQueryCapabilityFamily] {
        &[]
    }
}
impl domain::WorthQueryExecutableDomainOperation<DirectDomain, ReadFamily> for DirectRead {
    type Input = ();
    type Output = u64;
    type Publication = domain::WorthQueryTerminalOperation;
    type Execution = domain::WorthQueryDirectOperation;
}
struct Executor;
impl domain::WorthQueryDomainOperationExecutor<DirectDomain, DirectRead, ReadFamily> for Executor {
    const LOWERING_FAMILY: &'static str = "read-vertex-v1";
    const DETERMINISTIC: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        execution_resource_support()
    }
    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> { Some(installed_read_declaration()) }
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
pub(super) fn workspace(
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> crate::runtime::WorthQueryWorkspace {
    use crate::consumer_kit::{in_memory_test_runtime, WorthQueryTestBackendSchema};
    let schema = WorthQueryTestBackendSchema::single_collection("Vertex")
        .aspect_contract(operation_identity_contract(1))
        .unwrap()
        .aspect("identity.id", "identity.id")
        .unwrap();
    let package = domain::WorthQueryDomainPackage::declare(
        DirectDomain,
        domain::WorthQueryDomainIdentityDeclaration::new(
            domain::WorthQueryDomainIdentityNamespace::new("WORTH.tests").unwrap(),
            domain::WorthQueryDomainIdentityName::new("direct-custody").unwrap(),
            domain::WorthQueryDomainSemanticVersion::new(1, 0),
        ),
    )
    .operation(domain::WorthQueryDomainOperationDefinition::<
        DirectDomain,
        DirectRead,
        ReadFamily,
    >::new(
        domain::WorthQueryDomainOperationIdentity::new("read", 1),
        semantic_closure(
            canonical_bundle("Vertex"),
            domain::WorthQuerySupportRequirement::NotRequired,
            false,
        ),
    ));
    in_memory_test_runtime()
        .with_schema(schema)
        .domain_package(package)
        .domain_operation_executor(DirectDomain, DirectRead, ReadFamily, Executor)
        .workspace(request, "direct-custody")
        .unwrap()
}
pub(super) fn admitted(
    workspace: &crate::runtime::WorthQueryWorkspace,
) -> domain::WorthQueryAdmittedDirectOperation<
    DirectDomain,
    DirectRead,
    ReadFamily,
    crate::basis_lifecycle::ObservationLaneWitness,
> {
    let installed = workspace.domain(DirectDomain).unwrap();
    workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, DirectRead)
        .unwrap()
        .admit_execution_resources((), execution_resource_request(), workspace)
        .unwrap()
}
fn semantic_closure(
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

fn execution_resource_request() -> domain::WorthQueryExecutionResourceRequest {
    domain::WorthQueryExecutionResourceRequest::bounded(
        1_000_000,
        1_000_000,
        cancellation_safe_point(),
    )
}

fn execution_resource_contract() -> domain::WorthQueryExecutionResourceContract {
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

fn execution_resource_support() -> domain::WorthQueryExecutionResourceSupport {
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

fn operation_identity_contract(revision: u64) -> AspectContract {
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

fn canonical_bundle(
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

fn schema() -> read::QuerySchemaView {
    read::QuerySchemaView::new(
        "installed-operation",
        [read::SchemaFieldView::new(
            read::AspectName::new("identity").unwrap(),
            read::FieldName::new("id").unwrap(),
            read::ScalarAspectType::String,
        )],
        [],
    )
}

fn installed_read_declaration() -> &'static read::WorthQueryReadDeclaration {
    static DECLARATION: OnceLock<read::WorthQueryReadDeclaration> = OnceLock::new();
    DECLARATION.get_or_init(|| {
        read::declare(|builder| {
            builder.local_detail(
                "Vertex",
                schema(),
                |query| query.project(read::AspectFieldSelector::new("identity", "id").unwrap()),
                |shape| {
                    shape
                        .field(read::AuthoredResultShapeField::new("identity", "id", "id").unwrap())
                },
            )
        })
        .expect("installed read declaration fixture is canonical")
    })
}

