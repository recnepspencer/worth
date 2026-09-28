use std::collections::BTreeSet;

use worth_harness::facade::{
    AdapterSupport, CaptureDepth, DiagnosticsHarnessAdapter, DiagnosticsLevel, ExecutionMode,
    ExecutionProfile, ExecutionRequest, HarnessAdapter, HarnessCapabilities, MutationBatch,
    ReplayHarnessAdapter, ReplayRecord, ReplayRequest, RunRecord, SnapshotRecord,
};

use crate::facade::harness::RelationalHarnessError;
use crate::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use crate::facade::transactions::WorkerIntentBatch;

use super::batches::{entity_fixture_batch, relation_fixture_batch};
use super::data::{RelationalFixture, RelationalHarnessAdapter};
use super::targets::{commit_error_to_harness_error, default_harness_schema_registry};

mod aspect_snapshot_binaries;
mod diagnostic_fields_summary_projection;
mod diagnostics_capture;
mod diagnostics_summary_fields;
mod execution;
mod replay_capture;
mod replay_summary_fields;
mod run_summary_fields;
mod snapshot_capture;
mod terminal_harness_summary_projection;

impl HarnessAdapter for RelationalHarnessAdapter {
    type Runtime = RelationalRuntime;
    type Fixture = RelationalFixture;
    type Mutation = WorkerIntentBatch;
    type TargetId = String;
    type Error = RelationalHarnessError;

    fn adapter_name(&self) -> &'static str {
        "worth-relational"
    }

    fn capabilities(&self) -> HarnessCapabilities {
        let mut capabilities = HarnessCapabilities::default();
        capabilities.execution_modes = BTreeSet::from([
            ExecutionMode::RuntimeDefault,
            ExecutionMode::Serial,
            ExecutionMode::StagedParallel,
            ExecutionMode::FullParallel,
        ]);
        capabilities.diagnostics_levels = BTreeSet::from([
            DiagnosticsLevel::Operational,
            DiagnosticsLevel::Development,
            DiagnosticsLevel::Forensic,
        ]);
        capabilities.capture_depths = BTreeSet::from([
            CaptureDepth::Minimal,
            CaptureDepth::Standard,
            CaptureDepth::Rich,
        ]);
        capabilities.replay_support = AdapterSupport::Supported;
        capabilities.rich_record_kinds = BTreeSet::from([
            "relational_patch".to_string(),
            "relational_replay".to_string(),
            "relational_diagnostics".to_string(),
        ]);
        capabilities
    }

    fn create_runtime(&self) -> Result<Self::Runtime, Self::Error> {
        Ok(RelationalRuntimeApi::builder()
            .schema_registry(default_harness_schema_registry())
            .build())
    }

    fn prepare_runtime(
        &self,
        runtime: &mut Self::Runtime,
        profile: &ExecutionProfile,
    ) -> Result<(), Self::Error> {
        execution::prepare_runtime(runtime, profile)
    }

    fn load_fixture(
        &self,
        runtime: &mut Self::Runtime,
        fixture: &worth_harness::facade::ScenarioFixture<Self::Fixture>,
    ) -> Result<(), Self::Error> {
        if fixture.fixture.entities.is_empty() && fixture.fixture.relations.is_empty() {
            return Ok(());
        }
        let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
        txn.push_batch(entity_fixture_batch(&fixture.fixture.entities))
            .expect("test staging stays within configured resource budgets");
        let outcome = txn.commit(runtime).map_err(commit_error_to_harness_error)?;
        let entity_ids = outcome
            .changed_records
            .iter()
            .filter_map(|record| match record {
                crate::transactions::data::RecordRef::Entity(entity_id) => Some(*entity_id),
                crate::transactions::data::RecordRef::Relation(_) => None,
            })
            .collect::<Vec<_>>();
        if !fixture.fixture.relations.is_empty() {
            let mut relation_txn =
                crate::tests::support::test_owner_begin_transaction_for_main(runtime);
            relation_txn
                .push_batch(relation_fixture_batch(
                    &fixture.fixture.relations,
                    &entity_ids,
                )?)
                .expect("test staging stays within configured resource budgets");
            relation_txn
                .commit(runtime)
                .map_err(commit_error_to_harness_error)?;
        }
        Ok(())
    }

    fn apply_mutation_batch(
        &self,
        runtime: &mut Self::Runtime,
        batch: &MutationBatch<Self::Mutation>,
    ) -> Result<(), Self::Error> {
        let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
        for operation in &batch.operations {
            let operation: WorkerIntentBatch = operation.clone();
            txn.push_batch(operation)
                .expect("test staging stays within configured resource budgets");
        }
        txn.commit(runtime).map_err(commit_error_to_harness_error)?;
        Ok(())
    }

    fn execute(
        &self,
        runtime: &mut Self::Runtime,
        fixture: &worth_harness::facade::ScenarioFixture<Self::Fixture>,
        request: &ExecutionRequest<Self::TargetId>,
        profile: &ExecutionProfile,
    ) -> Result<RunRecord<Self::TargetId>, Self::Error> {
        execution::execute_request(self, runtime, fixture, request, profile)
    }

    fn capture_snapshot(
        &self,
        runtime: &Self::Runtime,
        fixture: &worth_harness::facade::ScenarioFixture<Self::Fixture>,
        request: &ExecutionRequest<Self::TargetId>,
        profile: &ExecutionProfile,
    ) -> Result<SnapshotRecord<Self::TargetId>, Self::Error> {
        snapshot_capture::capture_snapshot(self, runtime, fixture, request, profile)
    }
}

impl DiagnosticsHarnessAdapter for RelationalHarnessAdapter {
    fn capture_diagnostics(
        &self,
        runtime: &Self::Runtime,
        fixture: &worth_harness::facade::ScenarioFixture<Self::Fixture>,
        profile: &ExecutionProfile,
    ) -> Result<worth_harness::facade::DiagnosticsRecord, Self::Error> {
        diagnostics_capture::capture_diagnostics(self, runtime, fixture, profile)
    }
}

impl ReplayHarnessAdapter for RelationalHarnessAdapter {
    fn capture_replay(
        &self,
        runtime: &Self::Runtime,
        fixture: &worth_harness::facade::ScenarioFixture<Self::Fixture>,
        replay: &ReplayRequest<Self::TargetId>,
    ) -> Result<ReplayRecord<Self::TargetId>, Self::Error> {
        replay_capture::capture_replay(self, runtime, fixture, replay)
    }
}
