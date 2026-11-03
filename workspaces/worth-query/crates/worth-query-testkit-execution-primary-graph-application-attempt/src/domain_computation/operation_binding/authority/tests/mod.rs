use crate::domain_computation::operation_binding::authority::tests::resource_plan::admitted_plan;
use crate::domain_computation::operation_binding::authority::tests::resource_plan::admitted_plan_with_support_limit;
use std::sync::Arc;
use worth_query_admission::facade::basis::admit_basis_capability;
use worth_query_admission::facade::basis::evaluate_basis_observation_eligibility;
use worth_query_admission::facade::basis::normalize_raw_basis_intent;
use worth_query_admission::facade::basis::BasisOperationLane;
use worth_query_admission::facade::basis::NormalizedBasisIntent;
use worth_query_admission::facade::basis::ObservationLaneWitness;
use worth_query_admission::facade::basis::RawBasisIntent;
use worth_query_admission::facade::resource_admission::WorthQueryAdmittedExecutionResourcePlan;
use worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceAdmissionDenialKind;
use worth_query_execution::court_doors_for_test::domain_computation::operation_binding::authority::topology::test_topology;
use worth_query_execution::court_doors_for_test::domain_computation::operation_binding::authority::topology::WorthQueryExecutionResourceTopology;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionBoundOperationAuthority;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntimeInstaller;
use worth_query_execution::facade::domain_computation::WorthQueryInstalledDomainExecutionAuthority;
use worth_query_execution::facade::domain_computation::WorthQueryInstalledOperationExecutionSupport;
use worth_query_installation::facade::WorthQueryInstallationGeneration;
pub fn runtime() -> worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntime {
    WorthQueryExecutionRuntimeInstaller::new()
        .install(
            WorthQueryInstallationGeneration::initial(),
            std::iter::empty(),
        )
        .unwrap()
        .into_parts()
        .0
}
pub fn authority(
    runtime: &worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntime,
    binding_identity: &str,
    contract_identity: &str,
    plan: &WorthQueryAdmittedExecutionResourcePlan,
) -> WorthQueryExecutionBoundOperationAuthority {
    let (basis_identity, semantic_basis) = admitted_test_basis();
    WorthQueryExecutionBoundOperationAuthority {
        runtime_authority: runtime.authority_identity(),
        installation_runtime_ordinal: runtime.installed_packages().runtime_ordinal(),
        binding_identity: Arc::from(binding_identity),
        operation_identity: Arc::from("installed-operation"),
        basis_identity,
        semantic_basis,
        canonical_query_digest: Arc::from("installed-query"),
        operation_resource_contract_identity: Arc::from(contract_identity),
        provider_plan_declarations: Arc::new(Default::default()),
        commit_posture:
            worth_query_execution::facade::domain_computation::WorthQueryExecutionCommitPosture::ReadOnly,
        direct_resource_topology: Default::default(),
        workflow_stage_resources: None,
        operation_evidence_contract: None,
        installed_support: WorthQueryInstalledOperationExecutionSupport::direct(
            plan.support_snapshot().clone(),
        ),
        installed_domain: WorthQueryInstalledDomainExecutionAuthority::mint(
            runtime.authority_identity(),
            "test-domain",
            WorthQueryInstallationGeneration::initial(),
            runtime.retain_current_generation(),
        ),
        graph_work_affinity: None,
        application_operation_attempt: None,
        application_operation_slot: None,
        application_schema_binding: None,
        application_snapshot: None,
        application_product_observation: None,
    }
}
pub fn direct_authority(
    runtime: &worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntime,
    plan: &WorthQueryAdmittedExecutionResourcePlan,
) -> WorthQueryExecutionBoundOperationAuthority {
    authority(
        runtime,
        plan.binding_identity(),
        plan.contract_identity(),
        plan,
    )
}
pub fn authority_for_product(
    mut authority: WorthQueryExecutionBoundOperationAuthority,
    product: &worth_query_execution::facade::domain_computation::WorthQueryProductBranchLease,
) -> WorthQueryExecutionBoundOperationAuthority {
    authority.application_product_observation = Some(product.observation().clone());
    authority
}
pub fn direct_authority_with_graph(
    runtime: &worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntime,
    plan: &WorthQueryAdmittedExecutionResourcePlan,
    graph: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
    access: worth_query_installation::facade::WorthQueryOperationGraphAccess,
) -> WorthQueryExecutionBoundOperationAuthority {
    let mut authority = direct_authority(runtime, plan);
    authority.installation_runtime_ordinal = graph.runtime_ordinal();
    authority.direct_resource_topology = worth_query_execution::court_doors_for_test::domain_computation::operation_binding::authority::topology::resource_topology(
        std::iter::empty(),
        &[graph],
        std::iter::once((graph.role(), access)),
        std::iter::empty(),
        worth_query_execution::facade::domain_computation::WorthQueryExecutionCommitPosture::ReadOnly,
    );
    authority.provider_plan_declarations = Arc::new(
        worth_query_execution::court_doors_for_test::domain_computation::provider_session::protocol::declared_closure::WorthQueryProviderPlanDeclarations::test_direct(
            graph.role(),
            Some(access),
            false,
        ),
    );
    authority
}
pub fn direct_authority_with_graph_and_decision_facts(
    runtime: &worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntime,
    plan: &WorthQueryAdmittedExecutionResourcePlan,
    graph: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
    access: worth_query_installation::facade::WorthQueryOperationGraphAccess,
    families: Vec<worth_query_installation::facade::WorthQueryDecisionFactFamily>,
) -> WorthQueryExecutionBoundOperationAuthority {
    let mut authority = direct_authority_with_graph(runtime, plan, graph, access);
    authority.provider_plan_declarations = Arc::new(
        worth_query_execution::court_doors_for_test::domain_computation::provider_session::protocol::declared_closure::WorthQueryProviderPlanDeclarations::test_direct_with_decision_facts(
            graph.role(),
            access,
            families,
        ),
    );
    authority
}
pub fn direct_authority_with_graph_effect(
    runtime: &worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntime,
    plan: &WorthQueryAdmittedExecutionResourcePlan,
    graph: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
) -> WorthQueryExecutionBoundOperationAuthority {
    let mut authority = direct_authority(runtime, plan);
    authority.installation_runtime_ordinal = graph.runtime_ordinal();
    authority.direct_resource_topology = worth_query_execution::court_doors_for_test::domain_computation::operation_binding::authority::topology::resource_topology(
        std::iter::empty(),
        &[graph],
        std::iter::empty(),
        std::iter::once(graph.role()),
        worth_query_execution::facade::domain_computation::WorthQueryExecutionCommitPosture::ReadOnly,
    );
    authority.provider_plan_declarations = Arc::new(
        worth_query_execution::court_doors_for_test::domain_computation::provider_session::protocol::declared_closure::WorthQueryProviderPlanDeclarations::test_direct(
            graph.role(),
            None,
            true,
        ),
    );
    authority
}
pub fn direct_authority_with_graph_effect_and_decision_facts(
    runtime: &worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntime,
    plan: &WorthQueryAdmittedExecutionResourcePlan,
    graph: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
    families: Vec<worth_query_installation::facade::WorthQueryDecisionFactFamily>,
) -> WorthQueryExecutionBoundOperationAuthority {
    let mut authority = direct_authority_with_graph_effect(runtime, plan, graph);
    authority.provider_plan_declarations = Arc::new(
        worth_query_execution::court_doors_for_test::domain_computation::provider_session::protocol::declared_closure::WorthQueryProviderPlanDeclarations::test_effect_with_decision_facts(
            graph.role(),
            families,
        ),
    );
    authority
}
pub fn direct_authority_with_graph_effect_decision_facts_and_invariants(
    runtime: &worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntime,
    plan: &WorthQueryAdmittedExecutionResourcePlan,
    graph: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
    families: Vec<worth_query_installation::facade::WorthQueryDecisionFactFamily>,
    invariants: Vec<
        worth_query_installation::facade::WorthQueryInstalledInvariantExecutionRequirement,
    >,
) -> WorthQueryExecutionBoundOperationAuthority {
    let mut authority =
        direct_authority_with_graph_effect_and_decision_facts(runtime, plan, graph, families);
    authority.provider_plan_declarations = Arc::new(
        worth_query_execution::court_doors_for_test::domain_computation::provider_session::protocol::declared_closure::WorthQueryProviderPlanDeclarations::test_effect_with_decision_facts_and_invariants(
            graph.role(),
            authority.provider_plan_declarations.decision_fact_families().to_vec(),
            invariants,
        ),
    );
    authority
}
pub fn workflow_authority(
    runtime: &worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntime,
    plan: &worth_query_admission::facade::resource_admission::WorthQueryAdmittedWorkflowResourcePlan,
) -> WorthQueryExecutionBoundOperationAuthority {
    let stages = plan
        .stages()
        .map(|(stage, resources)| {
            (
                Arc::<str>::from(stage),
                worth_query_execution::court_doors_for_test::domain_computation::operation_binding::authority::WorthQueryWorkflowStageResourceAuthority {
                    contract_identity: Arc::from(resources.contract_identity()),
                    topology: Default::default(),
                    predecessors: Arc::from([]),
                    artifact_contracts:
                        worth_query_execution::court_doors_for_test::domain_computation::artifact_owner::installed_binding::WorthQueryInstalledWorkflowArtifactContracts::empty(),
                },
            )
        })
        .collect();
    let support = WorthQueryInstalledOperationExecutionSupport::workflow(
        plan.operation().support_snapshot().clone(),
        plan.stages()
            .map(|(stage, resources)| (stage.to_owned(), resources.support_snapshot().clone())),
    );
    let (basis_identity, semantic_basis) = admitted_test_basis();
    WorthQueryExecutionBoundOperationAuthority {
        runtime_authority: runtime.authority_identity(),
        installation_runtime_ordinal: runtime.installed_packages().runtime_ordinal(),
        binding_identity: Arc::from(plan.operation().binding_identity()),
        operation_identity: Arc::from("installed-workflow-operation"),
        basis_identity,
        semantic_basis,
        canonical_query_digest: Arc::from("installed-query"),
        operation_resource_contract_identity: Arc::from(plan.operation().contract_identity()),
        provider_plan_declarations: Arc::new(Default::default()),
        commit_posture:
            worth_query_execution::facade::domain_computation::WorthQueryExecutionCommitPosture::ReadOnly,
        direct_resource_topology: Default::default(),
        workflow_stage_resources: Some(stages),
        operation_evidence_contract: None,
        installed_support: support,
        installed_domain: WorthQueryInstalledDomainExecutionAuthority::mint(
            runtime.authority_identity(),
            "test-domain",
            WorthQueryInstallationGeneration::initial(),
            runtime.retain_current_generation(),
        ),
        graph_work_affinity: None,
        application_operation_attempt: None,
        application_operation_slot: None,
        application_schema_binding: None,
        application_snapshot: None,
        application_product_observation: None,
    }
}
pub fn workflow_authority_with_output_artifact(
    runtime: &worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntime,
    plan: &worth_query_admission::facade::resource_admission::WorthQueryAdmittedWorkflowResourcePlan,
    stage_identity: &str,
    output: Arc<worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>,
) -> WorthQueryExecutionBoundOperationAuthority {
    let mut authority = workflow_authority(runtime, plan);
    authority
        .workflow_stage_resources
        .as_mut()
        .and_then(|stages| stages.get_mut(stage_identity))
        .expect("managed-run artifact stage must exist")
        .artifact_contracts =
        worth_query_execution::court_doors_for_test::domain_computation::artifact_owner::installed_binding::WorthQueryInstalledWorkflowArtifactContracts::with_output(
            output,
        );
    authority
}
pub fn workflow_authority_with_stage_graph(
    runtime: &worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntime,
    plan: &worth_query_admission::facade::resource_admission::WorthQueryAdmittedWorkflowResourcePlan,
    stage_identity: &str,
    graph: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
    access: worth_query_installation::facade::WorthQueryOperationGraphAccess,
) -> WorthQueryExecutionBoundOperationAuthority {
    let mut authority = workflow_authority(runtime, plan);
    authority.installation_runtime_ordinal = graph.runtime_ordinal();
    authority
        .workflow_stage_resources
        .as_mut()
        .and_then(|stages| stages.get_mut(stage_identity))
        .expect("managed-run graph stage must exist")
        .topology = worth_query_execution::court_doors_for_test::domain_computation::operation_binding::authority::topology::resource_topology(
        std::iter::empty(),
        &[graph],
        std::iter::once((graph.role(), access)),
        std::iter::empty(),
        worth_query_execution::facade::domain_computation::WorthQueryExecutionCommitPosture::ReadOnly,
    );
    authority.provider_plan_declarations = Arc::new(
        worth_query_execution::court_doors_for_test::domain_computation::provider_session::protocol::declared_closure::WorthQueryProviderPlanDeclarations::test_workflow_stage(
            stage_identity,
            graph.role(),
            access,
        ),
    );
    authority
}
pub fn workflow_authority_with_stage_graph_and_output_artifact(
    runtime: &worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntime,
    plan: &worth_query_admission::facade::resource_admission::WorthQueryAdmittedWorkflowResourcePlan,
    stage_identity: &str,
    graph: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
    access: worth_query_installation::facade::WorthQueryOperationGraphAccess,
    output: Arc<worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>,
) -> WorthQueryExecutionBoundOperationAuthority {
    let mut authority =
        workflow_authority_with_stage_graph(runtime, plan, stage_identity, graph, access);
    authority
        .workflow_stage_resources
        .as_mut()
        .and_then(|stages| stages.get_mut(stage_identity))
        .expect("managed-run graph and artifact stage must exist")
        .artifact_contracts =
        worth_query_execution::court_doors_for_test::domain_computation::artifact_owner::installed_binding::WorthQueryInstalledWorkflowArtifactContracts::with_output(
            output,
        );
    authority
}
pub fn admitted_test_basis() -> (Arc<str>, NormalizedBasisIntent) {
    let normalized = normalize_raw_basis_intent(
        RawBasisIntent::CurrentHead,
        ObservationLaneWitness::lane_name(),
    )
    .expect("current-head test basis should normalize");
    let eligibility = evaluate_basis_observation_eligibility(normalized)
        .expect("current-head test basis should be eligible");
    let capability = admit_basis_capability(eligibility);
    (
        Arc::from(capability.capability_digest()),
        capability.normalized().clone(),
    )
}

#[path = "resource_plan/mod.rs"]
pub mod resource_plan;
