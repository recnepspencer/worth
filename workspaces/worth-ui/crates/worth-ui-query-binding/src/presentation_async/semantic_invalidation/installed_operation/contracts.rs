use worth_foundational::facade::{
    AbsenceLaw, AspectContract, AspectContractRevision, AspectEvolutionPolicy, AspectIdentity,
    AspectKey, AspectMask, FieldDeclaration, FieldKey, FieldRequirement, ProjectionMask,
    ScalarAspectType, StructAspectShape,
};
use worth_query::facade::domain;
use worth_query_decl::facade::{
    authoring::{
        AspectFieldSelector, AuthoredQueryBundleRequest, AuthoredResultShapeField,
        DetailQueryBuilder, DetailResultShapeBuilder, RootEntityKey,
    },
    binding::QueryBindingDescriptor,
    canonicalization::canonicalize_request,
};

use super::{
    WorthUiPresentationAsyncDomainEntry, WorthUiPresentationAsyncOperation,
    WorthUiPresentationAsyncOperationFamily,
};

const ROOT: &str = "WorthUiPresentation";
const LOWERING: &str = "worth-ui-presentation-async-v1";
pub(crate) const FIELDS: [(&str, &str, &str, u64); super::DEPENDENCY_COUNT] = [
    ("presentation-content", "content", "content", 0x5755_5001),
    ("presentation-width", "width", "width", 0x5755_5002),
    (
        "presentation-paint-value",
        "paint_value",
        "paint-value",
        0x5755_5003,
    ),
    (
        "presentation-paint-boundary",
        "paint_boundary",
        "paint-boundary",
        0x5755_5004,
    ),
    ("presentation-dpi", "dpi", "dpi", 0x5755_5005),
    (
        "presentation-upload",
        "upload_completion",
        "upload",
        0x5755_5006,
    ),
    (
        "presentation-pin-release",
        "pin_release",
        "pin-release",
        0x5755_5007,
    ),
    (
        "presentation-currentness",
        "currentness",
        "currentness",
        0x5755_5008,
    ),
];

pub(crate) fn presentation_aspect_contracts() -> Vec<AspectContract> {
    FIELDS
        .iter()
        .map(|(aspect, field, _, identity)| contract(aspect, field, *identity))
        .collect()
}

pub(crate) fn presentation_async_definition() -> domain::WorthQueryDomainOperationDefinition<
    WorthUiPresentationAsyncDomainEntry,
    WorthUiPresentationAsyncOperation,
    WorthUiPresentationAsyncOperationFamily,
> {
    domain::WorthQueryDomainOperationDefinition::new(
        domain::WorthQueryDomainOperationIdentity::new("presentation-async", 1),
        domain::WorthQueryDomainOperationSemanticClosure {
            parameters: domain::WorthQueryOperationParameterContract::NotRequired,
            native_projection: native_projection(&presentation_aspect_contracts()[7]),
            canonical_query: canonical_query(),
            collection: domain::WorthQueryOperationCollectionContract::NotCollection,
            required_capabilities: vec![
                domain::WorthQueryOperationCapabilityRequirement::QueryRead,
                domain::WorthQueryOperationCapabilityRequirement::QueryComposition,
            ],
            required_domains: Vec::new(),
            workflow: domain::WorthQueryOperationWorkflowContract::NotRequired,
            evidence: domain::WorthQueryDomainEvidenceContract::not_required(),
            conditional_nodes: Vec::new(),
            graph_reads: domain::WorthQueryOperationGraphReadContract::DeclaredDomain {
                roles: vec![domain::WorthQueryDomainOperationGraphReadRole {
                    role: "presentation".into(),
                    participation:
                        domain::WorthQueryOperationGraphParticipation::PrimaryLogicalGraph,
                    access: domain::WorthQueryOperationGraphAccess::Project,
                    semantic_reads: presentation_aspect_contracts()
                        .iter()
                        .map(native_projection)
                        .collect(),
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
            publication: domain::WorthQueryOperationPublicationContract::DerivedProjection {
                projection_role: domain::WorthQueryOperationProjectionRole::new("presentation")
                    .expect("static projection role must admit"),
            },
            projection_consumption:
                domain::WorthQueryOperationProjectionConsumptionContract::QueryReadAuthority,
            terminal: domain::WorthQueryOperationTerminalContract {
                result_states: vec![
                    domain::WorthQueryOperationResultState::Ready,
                    domain::WorthQueryOperationResultState::Partial,
                ],
                failure_classes: vec![domain::WorthQueryOperationFailureClass::Dependency],
            },
            cost: domain::WorthQueryOperationCostContract {
                lookup: domain::WorthQueryOperationCostClass::Constant,
                execution: domain::WorthQueryOperationCostClass::DeclaredWidth,
                result_width: domain::WorthQueryOperationCostClass::DeclaredWidth,
            },
            resources:
                crate::installed_domain::execution_resources::operation_execution_resource_contract(
                ),
            support: support_requirements(),
            lowering: domain::WorthQueryOperationLoweringContract {
                family: LOWERING.into(),
                deterministic: true,
            },
        },
    )
}

fn contract(aspect: &str, field: &str, identity: u64) -> AspectContract {
    let field = FieldDeclaration::new(
        FieldKey::new(field).expect("static presentation field must admit"),
        ScalarAspectType::UInt64,
        FieldRequirement::Required,
        AbsenceLaw::Required,
        AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .expect("presentation field contract must admit");
    AspectContract::struct_aspect(
        AspectKey::new(aspect).expect("static presentation aspect must admit"),
        AspectIdentity(identity),
        AspectContractRevision(1),
        StructAspectShape::new([field]).expect("presentation shape must admit"),
    )
}

fn native_projection(
    contract: &AspectContract,
) -> domain::WorthQueryOperationNativeProjectionContract {
    domain::WorthQueryOperationNativeProjectionContract::new(
        contract.clone(),
        AspectMask::<ProjectionMask>::whole_aspect(),
    )
    .expect("whole presentation aspect must admit")
}

fn canonical_query() -> worth_query_decl::facade::canonicalization::CanonicalQueryBundle {
    let mut query = DetailQueryBuilder::new(
        RootEntityKey::new(ROOT).expect("static presentation root must admit"),
    );
    let mut shape = DetailResultShapeBuilder::new();
    for (aspect, field, _, _) in FIELDS {
        query = query.project(selector(aspect, field));
        shape = shape.field(result_field(aspect, field));
    }
    canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(
            query
                .build()
                .expect("presentation query must admit")
                .into_raw(),
            shape
                .build()
                .expect("presentation shape must admit")
                .into_raw(),
            QueryBindingDescriptor::new(),
        )
        .expect("presentation bundle must admit"),
    )
    .expect("presentation bundle must canonicalize")
}

fn selector(aspect: &str, field: &str) -> AspectFieldSelector {
    AspectFieldSelector::new(aspect, field).expect("static presentation selector must admit")
}

fn result_field(aspect: &str, field: &str) -> AuthoredResultShapeField {
    AuthoredResultShapeField::new(aspect, field, format!("{aspect}.{field}"))
        .expect("static presentation result field must admit")
}

fn support_requirements() -> domain::WorthQueryOperationSupportRequirements {
    domain::WorthQueryOperationSupportRequirements {
        live: domain::WorthQuerySupportRequirement::Required,
        continuation: domain::WorthQuerySupportRequirement::NotRequired,
        async_result_state: domain::WorthQuerySupportRequirement::Required,
        recovery: domain::WorthQuerySupportRequirement::Required,
        inspection: domain::WorthQuerySupportRequirement::NotRequired,
        projection_consumption: domain::WorthQuerySupportRequirement::Required,
        dependency_impact: domain::WorthQuerySupportRequirement::NotRequired,
        sharing: domain::WorthQuerySupportRequirement::Required,
        invalidation: domain::WorthQuerySupportRequirement::NotRequired,
        collection_delivery: domain::WorthQuerySupportRequirement::NotRequired,
        conditional_evaluation: domain::WorthQuerySupportRequirement::NotRequired,
        conditional_comparator: domain::WorthQuerySupportRequirement::NotRequired,
        conditional_trigger: domain::WorthQuerySupportRequirement::NotRequired,
        conditional_temporal_or_on_demand: domain::WorthQuerySupportRequirement::NotRequired,
    }
}
