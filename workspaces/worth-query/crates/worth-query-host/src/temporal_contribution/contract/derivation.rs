use worth_query_declaration::facade::{
    authoring::{
        AspectFieldSelector, AuthoredQueryBundleRequest, AuthoredResultShapeField,
        DetailQueryBuilder, DetailResultShapeBuilder, RootEntityKey,
    },
    binding::QueryBindingDescriptor,
    canonicalization::{canonicalize_request, CanonicalQueryBundle},
    domain_computation::{WorthQueryCancellationSafePointFamily, WorthQueryExecutionMode},
};
use worth_query_execution::facade::application_contribution::WorthQueryOutputReadinessContractDenial;
use worth_query_installation::facade as domain;

use super::WorthQueryDesktopTemporalNodeSpec;

fn contract_denial(detail: impl Into<String>) -> WorthQueryOutputReadinessContractDenial {
    WorthQueryOutputReadinessContractDenial::from_detail(detail)
}

pub(super) fn build_temporal_node(
    spec: &WorthQueryDesktopTemporalNodeSpec,
    gate_contract: domain::AspectContract,
    gate_mask: domain::AspectMask<domain::ProjectionMask>,
) -> Result<
    domain::WorthQueryPortableConditionalNodeDeclaration,
    WorthQueryOutputReadinessContractDenial,
> {
    let dependency = domain::WorthQuerySemanticTruthDependency::new(
        domain::WorthQueryConditionalGraphReadRole::new("primary")
            .map_err(|error| contract_denial(format!("{error:?}")))?,
        gate_contract,
        gate_mask,
        domain::AspectBinding::EntityField {
            field: domain::FieldKey::new(&spec.gate_aspect)
                .ok_or_else(|| contract_denial("invalid gate aspect field key"))?,
        },
        domain::WorthQuerySemanticLocality::SourceRecord,
        [domain::AuthoritativeAspectChangeKind::FieldSet],
    )
    .map_err(|error| contract_denial(format!("{error:?}")))?;
    domain::WorthQueryPortableConditionalNodeDeclaration::declare(
        &spec.node_identity,
        domain::WorthQueryConditionalNodeRole::OperationGate,
    )
    .dependencies([dependency])
    .outputs([domain::WorthQueryConditionalNodeOutput::OperationOutput {
        projection_role: domain::WorthQueryOperationProjectionRole::new("intent")
            .map_err(|error| contract_denial(format!("{error:?}")))?,
    }])
    .required_context([
        domain::WorthQueryConditionalNodeContext::Snapshot,
        domain::WorthQueryConditionalNodeContext::OperationInput,
    ])
    .evaluation(
        domain::WorthQueryConditionalEvaluationCondition::temporal(
            domain::WorthQueryTemporalCondition::AfterNanoseconds(spec.after_nanoseconds),
        ),
        domain::WorthQueryConditionalTrigger::Temporal(spec.wake),
    )
    .comparison(
        domain::WorthQueryComparatorRequirement::FoundationalContractEquivalence,
        domain::WorthQueryOutputEquivalenceRequirement::ExactCanonicalValue,
    )
    .artifact_policy(
        domain::WorthQueryArtifactReuseEquivalence::NotReusable,
        domain::WorthQueryMaintenancePosture::Temporal,
        domain::WorthQueryArtifactPosture::Ephemeral,
    )
    .output_relationship(domain::WorthQueryOutputRelationship::IsOperationOutput)
    .finish()
    .map_err(|error| contract_denial(format!("{error:?}")))
}

pub(super) fn build_canonical_query(
    spec: &WorthQueryDesktopTemporalNodeSpec,
) -> Result<CanonicalQueryBundle, WorthQueryOutputReadinessContractDenial> {
    let query = DetailQueryBuilder::new(
        RootEntityKey::new(&spec.intent_entity)
            .map_err(|error| contract_denial(format!("{error:?}")))?,
    )
    .project(
        AspectFieldSelector::new(&spec.identity_aspect, &spec.identity_field)
            .map_err(|error| contract_denial(format!("{error:?}")))?,
    )
    .build()
    .map_err(|error| contract_denial(format!("{error:?}")))?
    .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(
            AuthoredResultShapeField::new(&spec.identity_aspect, &spec.identity_field, "identity")
                .map_err(|error| contract_denial(format!("{error:?}")))?,
        )
        .build()
        .map_err(|error| contract_denial(format!("{error:?}")))?
        .into_raw();
    canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new())
            .map_err(|error| contract_denial(format!("{error:?}")))?,
    )
    .map_err(|error| contract_denial(format!("{error:?}")))
}

pub(super) fn build_resource_contract(
    spec: &WorthQueryDesktopTemporalNodeSpec,
) -> Result<domain::WorthQueryExecutionResourceContract, WorthQueryOutputReadinessContractDenial> {
    if spec.max_work == 0 || spec.max_memory_bytes == 0 {
        return Err(contract_denial(
            "desktop temporal resource bounds must be strictly positive",
        ));
    }
    let envelope = domain::WorthQueryExecutionResourceEnvelope::bounded(
        spec.max_work,
        spec.max_memory_bytes,
        WorthQueryExecutionMode::Synchronous,
        WorthQueryCancellationSafePointFamily::new(domain::APPLICATION_EXECUTION_SAFE_POINT_FAMILY)
            .map_err(|error| contract_denial(format!("{error:?}")))?,
    );
    domain::WorthQueryExecutionResourceContract::declared([
        domain::WorthQueryExecutionStrategyContract::new(
            domain::WorthQueryExecutionStrategyName::new("application-temporal")
                .map_err(|error| contract_denial(format!("{error:?}")))?,
            envelope,
            domain::WorthQueryExecutionProviderRequirements::new(
                domain::WorthQueryExecutionProviderFamily::new(
                    domain::APPLICATION_EXECUTION_PROVIDER_FAMILY,
                )
                .map_err(|error| contract_denial(format!("{error:?}")))?,
                domain::WorthQueryExecutionAccessProductFamily::new(
                    domain::APPLICATION_EXECUTION_ACCESS_PRODUCT_FAMILY,
                )
                .map_err(|error| contract_denial(format!("{error:?}")))?,
                domain::WorthQueryExecutionAllocatorFamily::new(
                    domain::APPLICATION_EXECUTION_ALLOCATOR_FAMILY,
                )
                .map_err(|error| contract_denial(format!("{error:?}")))?,
            ),
        ),
    ])
    .map_err(|error| contract_denial(format!("{error:?}")))
}

pub(super) fn temporal_support_contract() -> domain::WorthQueryOperationSupportRequirements {
    let no = domain::WorthQuerySupportRequirement::NotRequired;
    let required = domain::WorthQuerySupportRequirement::Required;
    domain::WorthQueryOperationSupportRequirements {
        live: required,
        continuation: no,
        async_result_state: no,
        recovery: no,
        inspection: no,
        projection_consumption: required,
        dependency_impact: required,
        sharing: required,
        invalidation: required,
        collection_delivery: no,
        conditional_evaluation: required,
        conditional_comparator: required,
        conditional_trigger: required,
        conditional_temporal_or_on_demand: required,
    }
}
