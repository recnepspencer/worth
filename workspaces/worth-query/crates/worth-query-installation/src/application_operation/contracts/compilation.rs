use worth_query_declaration::facade::domain_computation::{
    WorthQueryCancellationSafePointFamily, WorthQueryExecutionMode, WorthQueryResourceDimension,
    WorthQueryResourceLimitRequest, WorthQuerySemanticScaleAxis, WorthQuerySemanticScaleRequest,
};

use super::compiled_contract::WorthQueryCompiledApplicationOperationContracts;
use super::invariant_compilation::mutation_contracts;
use crate::application_operation::WorthQueryApplicationCandidateDemand;
use crate::application_operation::WorthQuerySealedOperationContractCompilation;
use crate::domain_computation::{
    WorthQueryExecutionAccessProductFamily, WorthQueryExecutionAllocatorFamily,
    WorthQueryExecutionProviderFamily, WorthQueryExecutionProviderRequirements,
    WorthQueryExecutionResourceContract, WorthQueryExecutionResourceEnvelope,
    WorthQueryExecutionStrategyContract, WorthQueryExecutionStrategyName,
};
use crate::domain_operation::{
    WorthQueryDecisionFactFamily, WorthQueryDecisionFactKind,
    WorthQueryOperationDecisionFactContract, WorthQueryOperationReadTouchOverlapIndex,
};

pub const APPLICATION_EXECUTION_PROVIDER_FAMILY: &str = "primary-relational-provider";
pub const APPLICATION_EXECUTION_ACCESS_PRODUCT_FAMILY: &str = "typed-primary-graph";
pub const APPLICATION_EXECUTION_ALLOCATOR_FAMILY: &str = "primary-attempt-arena";
pub const APPLICATION_EXECUTION_SAFE_POINT_FAMILY: &str = "application-attempt-boundary";
pub const APPLICATION_DECISION_FACT_FAMILY: &str = "application-operation-decision-facts";
pub const APPLICATION_AUTHORIZATION_FACT_FAMILY: &str = "application-operation-authorization-facts";
pub const APPLICATION_INVARIANT_SLOT: &str = "application-touched-graph";

impl WorthQueryCompiledApplicationOperationContracts {
    pub(in crate::application_operation) fn compile(
        compilation: WorthQuerySealedOperationContractCompilation,
    ) -> Result<Self, ()> {
        let (
            authorization,
            mut ability_requirements,
            authored_program_width,
            decision_fact_budget,
            projection_work_budget,
            additional_authorization_fact_count,
            mutation_preconditions,
            execution_posture,
            external_effect,
            aftermath,
            graph_reads,
            touches,
            emissions,
            graph_mutation_count,
            candidate_demand,
            invariant_invocations,
        ) = compilation.into_parts();
        ability_requirements.sort();
        ability_requirements.dedup();
        let authorization_fact_count = authorization
            .exact_fact_count(ability_requirements.len())
            .saturating_add(additional_authorization_fact_count);
        let (effects, invariants, invariant_execution) = mutation_contracts(
            decision_fact_budget,
            graph_mutation_count,
            candidate_demand,
            &invariant_invocations,
        )?;
        let overlap_index = WorthQueryOperationReadTouchOverlapIndex::new(
            graph_reads
                .roles()
                .iter()
                .flat_map(|role| role.read_scopes().iter().cloned())
                .collect(),
            touches.scopes().to_vec(),
        );
        let decision_facts =
            application_decision_fact_contract(decision_fact_budget, authorization_fact_count);
        let resources = application_resource_contract(
            decision_fact_budget.saturating_add(authorization_fact_count),
            authored_program_width,
            candidate_demand,
        );
        Ok(Self {
            authorization,
            ability_requirements,
            graph_reads,
            touches,
            emissions,
            effects,
            invariants,
            decision_facts,
            invariant_execution,
            resources,
            decision_fact_budget,
            projection_work_budget,
            additional_authorization_fact_count,
            mutation_preconditions,
            execution_posture,
            external_effect,
            aftermath,
            overlap_index,
        })
    }
}

fn application_decision_fact_contract(
    maximum: usize,
    authorization_fact_count: usize,
) -> WorthQueryOperationDecisionFactContract {
    let application = WorthQueryDecisionFactFamily::new(
        APPLICATION_DECISION_FACT_FAMILY,
        WorthQueryDecisionFactKind::DomainStructuralProof,
    )
    .and_then(|family| family.with_bounded_fact_count(maximum))
    .expect("installed application decision-fact budget is nonzero and canonical");
    let mut families = vec![application];
    if authorization_fact_count > 0 {
        families.push(
            WorthQueryDecisionFactFamily::new(
                APPLICATION_AUTHORIZATION_FACT_FAMILY,
                WorthQueryDecisionFactKind::DomainStructuralProof,
            )
            .and_then(|family| family.with_exact_fact_count(authorization_fact_count))
            .expect("installed authorization requirement count is nonzero and canonical"),
        );
    }
    WorthQueryOperationDecisionFactContract::declared(families)
        .expect("application decision-fact families are valid")
}

fn application_resource_contract(
    decision_fact_budget: usize,
    program_width: usize,
    candidate_demand: WorthQueryApplicationCandidateDemand,
) -> WorthQueryExecutionResourceContract {
    let semantic_width = decision_fact_budget.saturating_add(program_width).max(1) as u64;
    let envelope = WorthQueryExecutionResourceEnvelope::new(
        WorthQuerySemanticScaleRequest::bounded(semantic_width)
            .with(
                WorthQuerySemanticScaleAxis::CandidateItems,
                semantic_width.max(candidate_demand.candidate_items()),
            )
            .with(
                WorthQuerySemanticScaleAxis::WorkItems,
                semantic_width.max(candidate_demand.validator_work()),
            ),
        WorthQueryResourceLimitRequest::bounded(semantic_width)
            .with(
                WorthQueryResourceDimension::CandidateRetainedRepresentationBytes,
                semantic_width.max(candidate_demand.retained_representation_bytes()),
            )
            .with(WorthQueryResourceDimension::RetainedBytes, 262_144),
        WorthQueryExecutionMode::Synchronous,
        None,
        WorthQueryCancellationSafePointFamily::new(APPLICATION_EXECUTION_SAFE_POINT_FAMILY)
            .expect("static application safe-point family is canonical"),
    );
    let requirements = WorthQueryExecutionProviderRequirements::new(
        WorthQueryExecutionProviderFamily::new(APPLICATION_EXECUTION_PROVIDER_FAMILY)
            .expect("static application provider family is canonical"),
        WorthQueryExecutionAccessProductFamily::new(APPLICATION_EXECUTION_ACCESS_PRODUCT_FAMILY)
            .expect("static application access-product family is canonical"),
        WorthQueryExecutionAllocatorFamily::new(APPLICATION_EXECUTION_ALLOCATOR_FAMILY)
            .expect("static application allocator family is canonical"),
    );
    WorthQueryExecutionResourceContract::declared([WorthQueryExecutionStrategyContract::new(
        WorthQueryExecutionStrategyName::new("primary-application-atomic")
            .expect("static application strategy name is canonical"),
        envelope,
        requirements,
    )])
    .expect("installed application execution resource contract is valid")
}
