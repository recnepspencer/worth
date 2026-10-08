use worth_query_declaration::facade::domain_computation::{
    WorthQueryCancellationSafePointFamily, WorthQueryResourceDimension,
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
            .checked_add(additional_authorization_fact_count)
            .ok_or(())?;
        let (effects, invariants, invariant_execution) =
            mutation_contracts(graph_mutation_count, &invariant_invocations)?;
        let overlap_index = WorthQueryOperationReadTouchOverlapIndex::new(
            graph_reads
                .roles()
                .iter()
                .flat_map(|role| role.read_scopes().iter().cloned())
                .collect(),
            touches.scopes().to_vec(),
        );
        let decision_facts = application_decision_fact_contract(authorization_fact_count);
        let resources = application_resource_contract(authored_program_width, candidate_demand)?;
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
            projection_work_budget,
            additional_authorization_fact_count,
            mutation_preconditions,
            execution_posture,
            external_effect,
            aftermath,
            overlap_index,
            platform_candidate_ceiling: candidate_demand.candidate_ceiling(),
            workflow_settlement_ceiling: candidate_demand.workflow_settlement_ceiling(),
        })
    }
}

fn application_decision_fact_contract(
    authorization_fact_count: usize,
) -> WorthQueryOperationDecisionFactContract {
    let application = WorthQueryDecisionFactFamily::new(
        APPLICATION_DECISION_FACT_FAMILY,
        WorthQueryDecisionFactKind::DomainStructuralProof,
    )
    .expect("installed application decision-fact family is canonical")
    .with_variable_fact_count();
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
    program_width: usize,
    candidate_demand: WorthQueryApplicationCandidateDemand,
) -> Result<WorthQueryExecutionResourceContract, ()> {
    let program_width = u64::try_from(program_width).map_err(|_| ())?;
    let scale = WorthQuerySemanticScaleRequest::selective()
        .with(
            WorthQuerySemanticScaleAxis::CandidateItems,
            candidate_demand.candidate_items(),
        )
        .with(WorthQuerySemanticScaleAxis::BatchWidth, program_width);
    let scale = match candidate_demand
        .candidate_ceiling()
        .and_then(|c| c.resources().maximum_validator_work())
    {
        Some(_) => scale.with(
            WorthQuerySemanticScaleAxis::WorkItems,
            candidate_demand.validator_work(),
        ),
        None => scale,
    };
    let envelope = WorthQueryExecutionResourceEnvelope::atomic(
        scale,
        WorthQueryResourceLimitRequest::selective()
            .with(
                WorthQueryResourceDimension::CandidateRetainedRepresentationBytes,
                candidate_demand.retained_representation_bytes(),
            )
            .with(WorthQueryResourceDimension::RetainedBytes, 262_144),
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
    .map_err(|_| ())
}

#[cfg(test)]
mod atomic_tests;
