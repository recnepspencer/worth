use worth_foundational::facade::CanonicalF32;
use worth_query::facade::domain;

use super::WorthUiInstalledQueryTestFixture;
use crate::installed_domain::{
    measurement_recording::{
        measurement_recording_definition, WorthUiMeasurementRecording,
        WorthUiMeasurementRecordingFamily, IDENTIFY_STAGE, RECORD_STAGE,
    },
    snapshot_measurement::snapshot_measurement_definition,
};
use crate::WorthUiQueryWorkspaceExt;

/// Query-owned semantic facts observed through Worth UI's real installed
/// package and in-memory runtime. This remains certification-only support.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiInstalledOperationCertificationFacts {
    workflow_stage_receipts: usize,
    workflow_effect_receipts: usize,
    replay: domain::WorthQueryOperationReplayContract,
    conditional_node_count: usize,
    semantic_read_count: usize,
    aftermath: Option<domain::WorthQueryInstalledAftermathContract>,
    lineage: domain::WorthQueryOperationLineageContract,
    dependency_impact: domain::WorthQuerySupportRequirement,
    executor_contacts: usize,
    result_state: domain::WorthQueryOperationResultState,
}

impl WorthUiInstalledQueryTestFixture {
    /// Executes the real recording workflow and settled snapshot, then
    /// projects only the narrow facts required by downstream certification.
    pub fn installed_operation_certification_facts(
        &mut self,
    ) -> WorthUiInstalledOperationCertificationFacts {
        let recording = measurement_recording_definition();
        let snapshot = snapshot_measurement_definition();
        let workflow = self.execute_recording_workflow();
        let settled = self.settle_snapshot();
        let snapshot_semantics = snapshot.semantics();
        WorthUiInstalledOperationCertificationFacts {
            workflow_stage_receipts: workflow.0,
            workflow_effect_receipts: workflow.1,
            replay: snapshot_semantics.replay.clone(),
            conditional_node_count: snapshot_semantics.conditional_nodes.len(),
            semantic_read_count: snapshot_semantics
                .graph_reads
                .domain_roles()
                .iter()
                .map(|role| role.semantic_reads.len())
                .sum(),
            aftermath: snapshot_semantics.aftermath.clone(),
            lineage: snapshot_semantics.lineage,
            dependency_impact: recording.semantics().support.dependency_impact,
            executor_contacts: settled.counters().executor_contacts,
            result_state: settled.result_state(),
        }
    }

    fn execute_recording_workflow(&mut self) -> (usize, usize) {
        let installed = self
            .workspace
            .worth_ui()
            .expect("Worth UI domain remains installed");
        let completed = self
            .workspace
            .prepare_mutation_operating_world(self.workspace.current_world())
            .expect("fixture prepares the Query-owned mutation world")
            .family(WorthUiMeasurementRecordingFamily)
            .bind(installed.handle(), WorthUiMeasurementRecording)
            .expect("installed recording operation binds")
            .admit_workflow_resources(
                crate::installed_domain::execution_resources::operation_execution_resource_request(
                ),
                &self.workspace,
            )
            .unwrap()
            .start_workflow(&mut self.workspace)
            .unwrap()
            .advance(
                IDENTIFY_STAGE,
                domain::WorthQueryWorkflowValue::Text("certification-measurement".into()),
                &mut self.workspace,
            )
            .unwrap()
            .advance(
                RECORD_STAGE,
                domain::WorthQueryWorkflowValue::U64(u64::from(
                    CanonicalF32::from_f32(42.0).bits(),
                )),
                &mut self.workspace,
            )
            .unwrap()
            .complete()
            .unwrap();
        let stages = completed.stage_receipts().len();
        let effects = completed
            .stage_receipts()
            .iter()
            .map(|stage| stage.effect_evidence().len())
            .sum();
        (stages, effects)
    }
}

impl WorthUiInstalledOperationCertificationFacts {
    pub fn workflow_stage_receipts(&self) -> usize {
        self.workflow_stage_receipts
    }

    pub fn workflow_effect_receipts(&self) -> usize {
        self.workflow_effect_receipts
    }

    pub fn replay(&self) -> domain::WorthQueryOperationReplayContract {
        self.replay.clone()
    }

    pub fn conditional_node_count(&self) -> usize {
        self.conditional_node_count
    }

    pub fn semantic_read_count(&self) -> usize {
        self.semantic_read_count
    }

    pub fn aftermath(&self) -> Option<&domain::WorthQueryInstalledAftermathContract> {
        self.aftermath.as_ref()
    }

    pub fn lineage(&self) -> domain::WorthQueryOperationLineageContract {
        self.lineage
    }

    pub fn dependency_impact(&self) -> domain::WorthQuerySupportRequirement {
        self.dependency_impact
    }

    pub fn executor_contacts(&self) -> usize {
        self.executor_contacts
    }

    pub fn result_state(&self) -> domain::WorthQueryOperationResultState {
        self.result_state
    }
}
