use worth_query_admission::facade::resource_admission::WorthQueryAdmittedExecutionResourcePlan;
use worth_query_installation::facade::{
    WorthQueryInstalledArtifactContractAuthority, WorthQueryInstalledGraphParticipationAuthority,
};

use super::WorthQueryExecutionBoundOperationAuthority;
use crate::domain_computation::provider_session::{
    WorthQueryProviderExecutionPlanContract, WorthQueryValidatedProviderPlan,
};

impl WorthQueryExecutionBoundOperationAuthority {
    pub(crate) fn admits_provider_plan_graph(
        &self,
        stage_identity: Option<&str>,
        graph: &WorthQueryInstalledGraphParticipationAuthority,
    ) -> bool {
        if graph.runtime_ordinal() != self.installation_runtime_ordinal {
            return false;
        }
        match (stage_identity, &self.workflow_stage_resources) {
            (None, None) => self
                .direct_resource_topology
                .contains_graph_authority(graph),
            (Some(stage), Some(stages)) => stages
                .get(stage)
                .is_some_and(|stage| stage.topology.contains_graph_authority(graph)),
            _ => false,
        }
    }

    pub(crate) fn admits_provider_plan_resources(
        &self,
        stage_identity: Option<&str>,
        resources: &WorthQueryAdmittedExecutionResourcePlan,
    ) -> bool {
        match (stage_identity, &self.workflow_stage_resources) {
            (None, None) => self.admits_direct_provider_plan_resources(resources),
            (Some(stage_identity), Some(stages)) => {
                let Some(stage) = stages.get(stage_identity) else {
                    return false;
                };
                resources.binding_identity()
                    == format!("{}:{stage_identity}", self.binding_identity)
                    && resources.contract_identity() == stage.contract_identity.as_ref()
                    && self
                        .installed_support
                        .workflow_stage(stage_identity)
                        .is_some_and(|support| support == resources.support_snapshot())
                    && stage.topology.admits(resources.support_snapshot())
            }
            _ => false,
        }
    }

    pub(crate) fn provider_plan_contract(
        &self,
        execution: WorthQueryValidatedProviderPlan<'_>,
    ) -> Option<WorthQueryProviderExecutionPlanContract> {
        if !execution.belongs_to(self) {
            return None;
        }
        let stage_identity = execution.stage_identity();
        let artifact_closure = self.provider_plan_artifact_closure(stage_identity)?;
        WorthQueryProviderExecutionPlanContract::bind(
            execution,
            &self.provider_plan_declarations,
            artifact_closure,
        )
    }

    fn admits_direct_provider_plan_resources(
        &self,
        resources: &WorthQueryAdmittedExecutionResourcePlan,
    ) -> bool {
        self.admits_operation_plan(resources)
            && self
                .installed_support
                .direct_operation()
                .is_some_and(|support| support == resources.support_snapshot())
            && self
                .direct_resource_topology
                .admits(resources.support_snapshot())
    }

    fn provider_plan_artifact_closure(&self, stage_identity: Option<&str>) -> Option<Vec<String>> {
        match stage_identity {
            None if self.workflow_stage_resources.is_none() => Some(
                self.operation_evidence_contract
                    .iter()
                    .map(|contract| installed_artifact_identity("operation-evidence", contract))
                    .collect(),
            ),
            Some(stage_identity) => {
                let contracts = self.workflow_stage_artifact_contracts(stage_identity)?;
                Some(
                    [
                        ("stage-input", contracts.input()),
                        ("stage-output", contracts.output()),
                        ("stage-evidence", contracts.evidence()),
                    ]
                    .into_iter()
                    .filter_map(|(role, contract)| {
                        contract.map(|contract| installed_artifact_identity(role, contract))
                    })
                    .collect(),
                )
            }
            None => None,
        }
    }
}

fn installed_artifact_identity(
    role: &str,
    contract: &WorthQueryInstalledArtifactContractAuthority,
) -> String {
    format!(
        "{role}|admission={}|contract={}",
        contract.admission_identity().render_support_hex(),
        contract.contract().identity().as_str(),
    )
}
