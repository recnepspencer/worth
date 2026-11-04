use worth_foundational::facade::AbsenceLaw;
use worth_foundational::facade::AspectContract;
use worth_foundational::facade::AspectContractRevision;
use worth_foundational::facade::AspectEvolutionPolicy;
use worth_foundational::facade::AspectIdentity;
use worth_foundational::facade::AspectKey;
use worth_foundational::facade::AspectMask;
use worth_foundational::facade::FieldDeclaration;
use worth_foundational::facade::FieldKey;
use worth_foundational::facade::FieldRequirement;
use worth_foundational::facade::ProjectionMask;
use worth_foundational::facade::ScalarAspectType;
use worth_foundational::facade::StructAspectShape;
use worth_query_declaration::facade::authoring::AspectFieldSelector;
use worth_query_declaration::facade::authoring::AuthoredQueryBundleRequest;
use worth_query_declaration::facade::authoring::AuthoredResultShapeField;
use worth_query_declaration::facade::authoring::DetailQueryBuilder;
use worth_query_declaration::facade::authoring::DetailResultShapeBuilder;
use worth_query_declaration::facade::authoring::RootEntityKey;
use worth_query_declaration::facade::binding::QueryBindingDescriptor;
use worth_query_declaration::facade::canonicalization::canonicalize_request;
use worth_query_installation::facade::WorthQueryArtifactContractReference;
use worth_query_installation::facade::WorthQueryDomainEvidenceContract;
use worth_query_installation::facade::WorthQueryDomainOperationDefinition;
use worth_query_installation::facade::WorthQueryDomainOperationGraphReadRole;
use worth_query_installation::facade::WorthQueryDomainOperationIdentity;
use worth_query_installation::facade::WorthQueryDomainOperationSemanticClosure;
use worth_query_installation::facade::WorthQueryInvariantExecutionContract;
use worth_query_installation::facade::WorthQueryOperationCollectionContract;
use worth_query_installation::facade::WorthQueryOperationCostClass;
use worth_query_installation::facade::WorthQueryOperationCostContract;
use worth_query_installation::facade::WorthQueryOperationDecisionFactContract;
use worth_query_installation::facade::WorthQueryOperationEffectContract;
use worth_query_installation::facade::WorthQueryOperationGraphAccess;
use worth_query_installation::facade::WorthQueryOperationGraphParticipation;
use worth_query_installation::facade::WorthQueryOperationGraphReadContract;
use worth_query_installation::facade::WorthQueryOperationInvariantContract;
use worth_query_installation::facade::WorthQueryOperationLineageContract;
use worth_query_installation::facade::WorthQueryOperationLoweringContract;
use worth_query_installation::facade::WorthQueryOperationNativeProjectionContract;
use worth_query_installation::facade::WorthQueryOperationParameterContract;
use worth_query_installation::facade::WorthQueryOperationProjectionConsumptionContract;
use worth_query_installation::facade::WorthQueryOperationPromotionContract;
use worth_query_installation::facade::WorthQueryOperationPublicationContract;
use worth_query_installation::facade::WorthQueryOperationReplayContract;
use worth_query_installation::facade::WorthQueryOperationResultState;
use worth_query_installation::facade::WorthQueryOperationSupportRequirements;
use worth_query_installation::facade::WorthQueryOperationTerminalContract;
use worth_query_installation::facade::WorthQueryOperationTouchContract;
use worth_query_installation::facade::WorthQueryOperationWorkflowContract;
use worth_query_installation::facade::WorthQueryPortableDomainOperationDefinition;
use worth_query_installation::facade::WorthQueryPortableWorkflowDefinition;
use worth_query_installation::facade::WorthQueryPortableWorkflowStage;
use worth_query_installation::facade::WorthQuerySupportRequirement;
use worth_query_installation::facade::WorthQueryWorkflowStageSemantics;
use worth_query_installation::facade::WorthQueryWorkflowValueContract;

use crate::domain_computation::convergence_epoch::tests::fixture::fixture_identity::GRAPH_ROLE;
use crate::domain_computation::convergence_epoch::tests::fixture::fixture_identity::WORKFLOW_STAGE;

pub fn workflow_operation(
    reference: WorthQueryArtifactContractReference,
    operation_resources: worth_query_installation::facade::WorthQueryExecutionResourceContract,
    stage_resources: worth_query_installation::facade::WorthQueryExecutionResourceContract,
    graph_access: WorthQueryOperationGraphAccess,
) -> WorthQueryPortableDomainOperationDefinition {
    let direct = direct_operation(reference.clone(), operation_resources.clone(), graph_access);
    let mut semantics = direct.semantics().clone();
    let stage = WorthQueryPortableWorkflowStage::new(
        WORKFLOW_STAGE,
        std::iter::empty::<&str>(),
        true,
        false,
        [],
    )
    .with_semantics(WorthQueryWorkflowStageSemantics {
        evidence: WorthQueryDomainEvidenceContract::installed_artifact(reference.clone()),
        output: WorthQueryWorkflowValueContract::installed_artifact(reference),
        graph_read_roles: vec![GRAPH_ROLE.into()],
        resources: stage_resources,
        terminal_result_states: vec![WorthQueryOperationResultState::Ready],
        ..WorthQueryWorkflowStageSemantics::default()
    });
    semantics.workflow = WorthQueryOperationWorkflowContract::Declared(
        WorthQueryPortableWorkflowDefinition::new(WORKFLOW_STAGE, [stage]),
    );
    semantics.evidence = WorthQueryDomainEvidenceContract::not_required();
    semantics.resources = operation_resources;
    WorthQueryDomainOperationDefinition::<(), (), ()>::new(
        WorthQueryDomainOperationIdentity::new("iterate-workflow", 1),
        semantics,
    )
    .into_portable()
}

pub fn direct_operation(
    reference: WorthQueryArtifactContractReference,
    resources: worth_query_installation::facade::WorthQueryExecutionResourceContract,
    graph_access: WorthQueryOperationGraphAccess,
) -> WorthQueryPortableDomainOperationDefinition {
    let projection = native_projection();
    let semantics = WorthQueryDomainOperationSemanticClosure {
        parameters: WorthQueryOperationParameterContract::NotRequired,
        native_projection: projection.clone(),
        canonical_query: canonical_query(),
        collection: WorthQueryOperationCollectionContract::NotCollection,
        required_capabilities: Vec::new(),
        required_domains: Vec::new(),
        workflow: WorthQueryOperationWorkflowContract::NotRequired,
        evidence: WorthQueryDomainEvidenceContract::installed_artifact(reference),
        conditional_nodes: Vec::new(),
        graph_reads: WorthQueryOperationGraphReadContract::DeclaredDomain {
            roles: vec![WorthQueryDomainOperationGraphReadRole {
                role: GRAPH_ROLE.into(),
                participation: WorthQueryOperationGraphParticipation::SeparateAuthority {
                    role: GRAPH_ROLE.into(),
                },
                access: graph_access,
                semantic_reads: vec![projection],
            }],
        },
        decision_facts: WorthQueryOperationDecisionFactContract::NotRequired,
        touches: WorthQueryOperationTouchContract::NotRequired,
        effects: WorthQueryOperationEffectContract::NotRequired,
        invariants: WorthQueryOperationInvariantContract::NotRequired,
        invariant_execution: WorthQueryInvariantExecutionContract::NotRequired,
        replay: WorthQueryOperationReplayContract::ReExecutable,
        aftermath: None,
        lineage: WorthQueryOperationLineageContract::NotRequired,
        promotion: WorthQueryOperationPromotionContract::NotRequired,
        publication: WorthQueryOperationPublicationContract::NotRequired,
        projection_consumption: WorthQueryOperationProjectionConsumptionContract::NotRequired,
        terminal: WorthQueryOperationTerminalContract {
            result_states: vec![WorthQueryOperationResultState::Ready],
            failure_classes: Vec::new(),
        },
        cost: WorthQueryOperationCostContract {
            lookup: WorthQueryOperationCostClass::Constant,
            execution: WorthQueryOperationCostClass::GraphBreadth,
            result_width: WorthQueryOperationCostClass::Constant,
        },
        resources,
        support: no_support(),
        lowering: WorthQueryOperationLoweringContract {
            family: "worth.convergence.iterate".into(),
            deterministic: true,
        },
    };
    WorthQueryDomainOperationDefinition::<(), (), ()>::new(
        WorthQueryDomainOperationIdentity::new("iterate", 1),
        semantics,
    )
    .into_portable()
}

pub fn native_projection() -> WorthQueryOperationNativeProjectionContract {
    let field = FieldDeclaration::new(
        FieldKey::new("state").unwrap(),
        ScalarAspectType::String,
        FieldRequirement::Required,
        AbsenceLaw::Required,
        AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .unwrap();
    let contract = AspectContract::struct_aspect(
        AspectKey::new("candidate").unwrap(),
        AspectIdentity(9_156),
        AspectContractRevision(1),
        StructAspectShape::new([field]).unwrap(),
    );
    WorthQueryOperationNativeProjectionContract::new(
        contract,
        AspectMask::<ProjectionMask>::whole_aspect(),
    )
    .unwrap()
}

pub fn canonical_query() -> worth_query_declaration::facade::canonicalization::CanonicalQueryBundle
{
    let query = DetailQueryBuilder::new(RootEntityKey::new("Candidate").unwrap())
        .project(AspectFieldSelector::new("candidate", "state").unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(AuthoredResultShapeField::new("candidate", "state", "state").unwrap())
        .build()
        .unwrap()
        .into_raw();
    canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new())
            .unwrap(),
    )
    .unwrap()
}

pub fn no_support() -> WorthQueryOperationSupportRequirements {
    WorthQueryOperationSupportRequirements {
        live: WorthQuerySupportRequirement::NotRequired,
        continuation: WorthQuerySupportRequirement::NotRequired,
        async_result_state: WorthQuerySupportRequirement::NotRequired,
        recovery: WorthQuerySupportRequirement::NotRequired,
        inspection: WorthQuerySupportRequirement::NotRequired,
        projection_consumption: WorthQuerySupportRequirement::NotRequired,
        dependency_impact: WorthQuerySupportRequirement::NotRequired,
        sharing: WorthQuerySupportRequirement::NotRequired,
        invalidation: WorthQuerySupportRequirement::NotRequired,
        collection_delivery: WorthQuerySupportRequirement::NotRequired,
        conditional_evaluation: WorthQuerySupportRequirement::NotRequired,
        conditional_comparator: WorthQuerySupportRequirement::NotRequired,
        conditional_trigger: WorthQuerySupportRequirement::NotRequired,
        conditional_temporal_or_on_demand: WorthQuerySupportRequirement::NotRequired,
    }
}
