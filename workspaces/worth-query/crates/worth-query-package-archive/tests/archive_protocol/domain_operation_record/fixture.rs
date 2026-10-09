#[path = "fixture/optional_work_resources.rs"]
mod optional_work_resources;

use std::num::NonZeroU32;

use worth_foundational::facade::{
    AbsenceLaw, AspectContract, AspectContractRevision, AspectEvolutionPolicy, AspectIdentity,
    AspectKey, AspectMask, FieldDeclaration, FieldKey, FieldRequirement, ProjectionMask,
    ScalarAspectType, StructAspectShape,
};
use worth_query_declaration::facade::authoring::{
    AspectFieldSelector, AuthoredQueryBundleRequest, AuthoredResultShapeField,
    CollectionQueryBuilder, CollectionResultShapeBuilder, DetailQueryBuilder,
    DetailResultShapeBuilder, OrderingSelector, RootEntityKey,
};
use worth_query_declaration::facade::binding::QueryBindingDescriptor;
use worth_query_declaration::facade::canonicalization::canonicalize_request;
use worth_query_declaration::facade::domain_computation::{
    WorthQueryCancellationSafePointFamily, WorthQueryExecutionMode,
};
use worth_query_installation::facade::*;

struct ArchiveTrigger;

impl WorthQueryOnDemandTriggerFamily for ArchiveTrigger {
    const PORTABLE_IDENTITY: &'static str = "worth.archive.trigger";
}

pub(in super::super) fn operation_package() -> WorthQueryValidatedPortableDomainPackage {
    package("archive.operation", operation())
}

pub(in super::super) fn operation_package_without_work_budget(
) -> WorthQueryValidatedPortableDomainPackage {
    let resources = optional_work_resources::resources();
    package(
        "archive.operation",
        operation_with_query_collection_resources(
            canonical_query(),
            WorthQueryOperationCollectionContract::NotCollection,
            resources,
        ),
    )
}

pub(super) fn collection_operation_package() -> WorthQueryValidatedPortableDomainPackage {
    package("archive.collection", collection_operation())
}

fn package(
    identity: &str,
    operation: WorthQueryPortableDomainOperationDefinition,
) -> WorthQueryValidatedPortableDomainPackage {
    WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(identity, 1, 0))
        .domain_operation(operation)
        .validate()
        .unwrap()
}

pub(crate) fn operation() -> WorthQueryPortableDomainOperationDefinition {
    operation_with_query_and_collection(
        canonical_query(),
        WorthQueryOperationCollectionContract::NotCollection,
    )
}

fn collection_operation() -> WorthQueryPortableDomainOperationDefinition {
    operation_with_query_and_collection(collection_query(), collection_contract())
}

fn operation_with_query_and_collection(
    canonical_query: worth_query_declaration::facade::canonicalization::CanonicalQueryBundle,
    collection: WorthQueryOperationCollectionContract,
) -> WorthQueryPortableDomainOperationDefinition {
    operation_with_query_collection_resources(canonical_query, collection, resource_contract())
}

fn operation_with_query_collection_resources(
    canonical_query: worth_query_declaration::facade::canonicalization::CanonicalQueryBundle,
    collection: WorthQueryOperationCollectionContract,
    resources: WorthQueryExecutionResourceContract,
) -> WorthQueryPortableDomainOperationDefinition {
    let native_projection = native_projection();
    let graph_reads = WorthQueryOperationGraphReadContract::DeclaredDomain {
        roles: vec![WorthQueryDomainOperationGraphReadRole {
            role: "ledger".into(),
            participation: WorthQueryOperationGraphParticipation::PrimaryLogicalGraph,
            access: WorthQueryOperationGraphAccess::Project,
            semantic_reads: vec![native_projection.clone()],
        }],
    };
    let decision_facts =
        WorthQueryOperationDecisionFactContract::declared([WorthQueryDecisionFactFamily::new(
            "account-balance",
            WorthQueryDecisionFactKind::ObservedValue,
        )
        .unwrap()
        .with_exact_fact_count(1)
        .unwrap()])
        .unwrap();
    let invariant_execution = WorthQueryInvariantExecutionContract::declared([
        WorthQueryInstalledInvariantExecutionRequirement::new(
            "balanced",
            "ledger.balance",
            NonZeroU32::new(1).unwrap(),
            WorthQueryInvariantEnforcement::Blocking,
            "ledger",
            ["account-balance"],
            4,
            32,
        )
        .unwrap(),
    ])
    .unwrap();
    let semantics = WorthQueryDomainOperationSemanticClosure {
        parameters: WorthQueryOperationParameterContract::Declared {
            fields: vec![WorthQueryOperationParameterField {
                name: "account".into(),
                value_family: WorthQueryOperationValueFamily::EntityIdentity,
                required: true,
            }],
        },
        native_projection,
        canonical_query,
        collection,
        required_capabilities: vec![WorthQueryOperationCapabilityRequirement::QueryRead],
        required_domains: vec![WorthQueryOperationRequiredDomainRole::new("ledger").unwrap()],
        workflow: workflow(),
        evidence: WorthQueryDomainEvidenceContract::not_required(),
        conditional_nodes: vec![conditional_node()],
        graph_reads,
        decision_facts,
        touches: WorthQueryOperationTouchContract::Declared {
            graph_roles: vec!["ledger".into()],
            scopes: vec![WorthQueryOperationTouchScope::DeclaredDomain(
                WorthQueryDeclaredDomainTouchScopeIdentity::new("account-balance").unwrap(),
            )],
        },
        effects: WorthQueryOperationEffectContract::Declared {
            effect_families: vec![WorthQueryOperationEffectFamily::Mutation],
        },
        invariants: WorthQueryOperationInvariantContract::Declared {
            invariant_slots: vec!["balanced".into()],
        },
        invariant_execution,
        replay: WorthQueryOperationReplayContract::CertReplayableWithNoise {
            comparator: WorthQueryOperationReplayComparatorContract::new("ledger.exact").unwrap(),
            noise: WorthQueryOperationReplayNoiseContract {
                diagnostic_warnings: true,
            },
        },
        aftermath: None,
        lineage: WorthQueryOperationLineageContract::Preserve,
        promotion: WorthQueryOperationPromotionContract::NotRequired,
        publication: WorthQueryOperationPublicationContract::DerivedProjection {
            projection_role: WorthQueryOperationProjectionRole::new("balance").unwrap(),
        },
        projection_consumption:
            WorthQueryOperationProjectionConsumptionContract::QueryReadAuthority,
        terminal: WorthQueryOperationTerminalContract {
            result_states: vec![WorthQueryOperationResultState::Ready],
            failure_classes: vec![WorthQueryOperationFailureClass::Conflict],
        },
        cost: WorthQueryOperationCostContract {
            lookup: WorthQueryOperationCostClass::Constant,
            execution: WorthQueryOperationCostClass::DeclaredWidth,
            result_width: WorthQueryOperationCostClass::Constant,
        },
        resources,
        support: support(),
        lowering: WorthQueryOperationLoweringContract {
            family: "worth.archive.ledger-balance".into(),
            deterministic: true,
        },
    };
    WorthQueryDomainOperationDefinition::<(), (), ()>::new(
        WorthQueryDomainOperationIdentity::new("ledger-balance", 1),
        semantics,
    )
    .into_portable()
}

fn native_projection() -> WorthQueryOperationNativeProjectionContract {
    let field = FieldDeclaration::new(
        FieldKey::new("balance").unwrap(),
        ScalarAspectType::Int64,
        FieldRequirement::Required,
        AbsenceLaw::Required,
        AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .unwrap();
    WorthQueryOperationNativeProjectionContract::new(
        AspectContract::struct_aspect(
            AspectKey::new("account").unwrap(),
            AspectIdentity(9_162),
            AspectContractRevision(1),
            StructAspectShape::new([field]).unwrap(),
        ),
        AspectMask::<ProjectionMask>::whole_aspect(),
    )
    .unwrap()
}

fn workflow() -> WorthQueryOperationWorkflowContract {
    let stage = WorthQueryPortableWorkflowStage::new(
        "apply",
        std::iter::empty::<&str>(),
        true,
        true,
        [WorthQueryOperationCapabilityRequirement::QueryRead],
    )
    .with_semantics(WorthQueryWorkflowStageSemantics {
        output: WorthQueryWorkflowValueContract::Projection,
        required_domain_roles: vec![WorthQueryOperationRequiredDomainRole::new("ledger").unwrap()],
        graph_read_roles: vec!["ledger".into()],
        touch_roles: vec!["ledger".into()],
        effect_roles: vec![WorthQueryOperationEffectFamily::Mutation],
        invariant_roles: vec!["balanced".into()],
        cost_roles: vec![WorthQueryWorkflowCostRole::Execution],
        resources: resource_contract(),
        terminal_result_states: vec![WorthQueryOperationResultState::Ready],
        failure_classes: vec![WorthQueryOperationFailureClass::Conflict],
        ..WorthQueryWorkflowStageSemantics::default()
    });
    WorthQueryOperationWorkflowContract::Declared(WorthQueryPortableWorkflowDefinition::new(
        "apply",
        [stage],
    ))
}

fn conditional_node() -> WorthQueryPortableConditionalNodeDeclaration {
    WorthQueryPortableConditionalNodeDeclaration::declare(
        "request-gate",
        WorthQueryConditionalNodeRole::OperationGate,
    )
    .dependencies([])
    .outputs([WorthQueryConditionalNodeOutput::OperationOutput {
        projection_role: WorthQueryOperationProjectionRole::new("balance").unwrap(),
    }])
    .required_context([WorthQueryConditionalNodeContext::OperationInput])
    .evaluation(
        WorthQueryConditionalEvaluationCondition::on_demand(),
        WorthQueryConditionalTrigger::on_demand::<ArchiveTrigger>(),
    )
    .comparison(
        WorthQueryComparatorRequirement::ExactCanonicalValue,
        WorthQueryOutputEquivalenceRequirement::ExactCanonicalValue,
    )
    .artifact_policy(
        WorthQueryArtifactReuseEquivalence::NotReusable,
        WorthQueryMaintenancePosture::OnDemandOnly,
        WorthQueryArtifactPosture::Ephemeral,
    )
    .output_relationship(WorthQueryOutputRelationship::IsOperationOutput)
    .finish()
    .unwrap()
}

fn canonical_query() -> worth_query_declaration::facade::canonicalization::CanonicalQueryBundle {
    let query = DetailQueryBuilder::new(RootEntityKey::new("Account").unwrap())
        .project(AspectFieldSelector::new("account", "balance").unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(AuthoredResultShapeField::new("account", "balance", "balance").unwrap())
        .build()
        .unwrap()
        .into_raw();
    canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new())
            .unwrap(),
    )
    .unwrap()
}

fn collection_query() -> worth_query_declaration::facade::canonicalization::CanonicalQueryBundle {
    let query = CollectionQueryBuilder::new(RootEntityKey::new("Account").unwrap())
        .project(AspectFieldSelector::new("account", "balance").unwrap())
        .order_by(OrderingSelector::ascending("account", "balance").unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = CollectionResultShapeBuilder::new()
        .field(AuthoredResultShapeField::new("account", "balance", "balance").unwrap())
        .build()
        .unwrap()
        .into_raw();
    canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new())
            .unwrap(),
    )
    .unwrap()
}

fn collection_contract() -> WorthQueryOperationCollectionContract {
    let balance = WorthQueryOperationCollectionField::from_dotted("account.balance").unwrap();
    WorthQueryOperationCollectionContract::Collection {
        row_identity_field: balance.clone(),
        ordering_fields: vec![balance.clone()],
        grouping: WorthQueryOperationGroupingContract::Grouped {
            grouping_fields: vec![balance],
        },
        window: WorthQueryOperationWindowPolicy::ContinuationBounded,
        continuation: WorthQueryOperationContinuationPosture::SnapshotCursor,
    }
}

fn resource_contract() -> WorthQueryExecutionResourceContract {
    WorthQueryExecutionResourceContract::declared([WorthQueryExecutionStrategyContract::new(
        WorthQueryExecutionStrategyName::new("bounded").unwrap(),
        WorthQueryExecutionResourceEnvelope::bounded(
            64,
            64,
            WorthQueryExecutionMode::Synchronous,
            WorthQueryCancellationSafePointFamily::new("record-boundary").unwrap(),
        ),
        WorthQueryExecutionProviderRequirements::new(
            WorthQueryExecutionProviderFamily::new("fixture-provider").unwrap(),
            WorthQueryExecutionAccessProductFamily::new("fixture-access").unwrap(),
            WorthQueryExecutionAllocatorFamily::new("fixture-arena").unwrap(),
        ),
    )])
    .unwrap()
}

fn support() -> WorthQueryOperationSupportRequirements {
    WorthQueryOperationSupportRequirements {
        live: WorthQuerySupportRequirement::NotRequired,
        continuation: WorthQuerySupportRequirement::NotRequired,
        async_result_state: WorthQuerySupportRequirement::NotRequired,
        recovery: WorthQuerySupportRequirement::Required,
        inspection: WorthQuerySupportRequirement::Required,
        projection_consumption: WorthQuerySupportRequirement::Required,
        dependency_impact: WorthQuerySupportRequirement::Required,
        sharing: WorthQuerySupportRequirement::NotRequired,
        invalidation: WorthQuerySupportRequirement::Required,
        collection_delivery: WorthQuerySupportRequirement::NotRequired,
        conditional_evaluation: WorthQuerySupportRequirement::Required,
        conditional_comparator: WorthQuerySupportRequirement::Required,
        conditional_trigger: WorthQuerySupportRequirement::Required,
        conditional_temporal_or_on_demand: WorthQuerySupportRequirement::Required,
    }
}
