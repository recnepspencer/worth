//! Issued custody, real owner gates, and authentic registration interleavings.
use super::*;
use crate::internal_port::change_source::retained_observation::RelationalRetainedObservationOwnerPort as _;
use std::sync::Arc;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};

use crate::branch::basis::{AdmittedRelationalBranchBasis, RelationalBranchBasisDescriptor};
use crate::branch::basis_admission_tests::installed_history;
use crate::branch::basis_observation_inputs::BranchObservationInputs;
use crate::config::data::runtime_config::RelationalRuntimeConfig;
use crate::runtime::state::subsystems::{
    history::HistorySubsystem, services::instrumentation::RuntimeInstrumentation,
    visibility::VisibilitySubsystem,
};
use crate::schema::data::RelationalSchemaRegistry;

struct NativeFixture {
    history: HistorySubsystem,
    visibility: VisibilitySubsystem,
    schema: RelationalSchemaRegistry,
    instrumentation: RuntimeInstrumentation,
}

impl NativeFixture {
    fn new() -> Self {
        Self {
            history: installed_history(1),
            visibility: VisibilitySubsystem::new(&RelationalRuntimeConfig::default()),
            schema: RelationalSchemaRegistry::new(),
            instrumentation: RuntimeInstrumentation::new(),
        }
    }

    fn observe(
        &self,
    ) -> (
        RelationalBranchBasisDescriptor,
        AdmittedRelationalBranchBasis,
    ) {
        let inputs = BranchObservationInputs {
            history: &self.history,
            runtime_instance_id: self.history.runtime_instance_id,
            empty_schema: &self.schema,
            instrumentation: &self.instrumentation,
        };
        let branch = inputs.branch_identity(&self.history.main_branch).unwrap();
        inputs.observe(&branch, || None).unwrap()
    }

    fn retain(&self, basis: &AdmittedRelationalBranchBasis) -> RelationalRetainedObservation {
        let retention = crate::branch::basis_retention::retain_component_basis(
            &self.history,
            self.history.runtime_instance_id,
            &self.instrumentation,
            basis,
        )
        .unwrap();
        let snapshot = self.visibility.allocate_snapshot_id().unwrap();
        RelationalRetainedObservation::from_admitted(snapshot, basis.observation(), retention)
    }
}

#[test]
fn retained_observation_original_cut_and_lane_busy_retry_same_request() {
    let stop = CancellationSource::new();
    let host = SerialRequest::from_memory(SerialMemoryBudget::new(None), stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    let fixture = NativeFixture::new();
    let (_, basis) = fixture.observe();
    let retained = fixture.retain(&basis);
    let mut task = retained.into_retirement();
    crate::history::retention::assert_external_cut_and_lane_busy(&mut task, &basis, &request);
    drop(basis);
    finish(&mut task, &request);
    let retired = task.into_retired().unwrap();
    assert!(!fixture.history.has_detached_retention_obligations());
    retired.reclaim();
}

#[test]
fn retained_observation_stop_after_external_release_preserves_once_only_accounting() {
    let stop = CancellationSource::new();
    let host = SerialRequest::from_memory(SerialMemoryBudget::new(None), stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    let fixture = NativeFixture::new();
    let (_, basis) = fixture.observe();
    let terminal = fixture
        .instrumentation
        .external_retention_terminal_accounting();
    let before = terminal.snapshot();
    let root = Arc::downgrade(&basis.inner.root);
    let retained = fixture.retain(&basis);
    drop(basis);
    let mut task = retained.into_retirement();
    assert_eq!(
        task.advance(&request),
        RelationalRetainedObservationRetirementProgress::Advanced
    );
    assert_eq!(
        terminal.snapshot().explicit_releases,
        before.explicit_releases + 1
    );
    stop.cancel();
    assert!(matches!(
        task.advance(&request),
        RelationalRetainedObservationRetirementProgress::Stopped(_)
    ));
    assert!(root.upgrade().is_some());
    assert_eq!(
        terminal.snapshot().dropped_releases,
        before.dropped_releases
    );
    let task = task.into_retired().unwrap_err();
    drop(task); // Explicitly unbudgeted abandonment joins remaining ordinary cleanup.
    assert_eq!(
        terminal.snapshot().explicit_releases,
        before.explicit_releases + 1
    );
    assert_eq!(
        terminal.snapshot().dropped_releases,
        before.dropped_releases
    );
    assert!(!fixture.history.has_detached_retention_obligations());
}

#[test]
fn retained_observation_registry_busy_retries_after_original_observation_release() {
    let stop = CancellationSource::new();
    let host = SerialRequest::from_memory(SerialMemoryBudget::new(None), stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    let fixture = NativeFixture::new();
    let (descriptor, basis) = fixture.observe();
    let retained = fixture.retain(&basis);
    drop(basis);
    let mut task = retained.into_retirement();
    crate::branch::basis_registry::assert_original_registry_busy(
        &mut task,
        &fixture.history,
        &descriptor,
        &request,
    );
    task.into_retired().unwrap().reclaim();
    assert!(!fixture.history.has_detached_retention_obligations());
}

#[test]
fn retained_observation_original_prune_is_not_decremented_twice() {
    let stop = CancellationSource::new();
    let host = SerialRequest::from_memory(SerialMemoryBudget::new(None), stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    let fixture = NativeFixture::new();
    let (descriptor, basis) = fixture.observe();
    let retained = fixture.retain(&basis);
    drop(basis);
    let mut task = retained.into_retirement();
    crate::branch::basis_registry::assert_pruned_original_registration(
        &mut task,
        &fixture.history,
        &descriptor,
        &request,
    );
    task.into_retired().unwrap().reclaim();
    assert!(!fixture.history.has_detached_retention_obligations());
}

#[test]
fn retained_observation_successor_registration_and_metrics_survive_original_retirement() {
    let stop = CancellationSource::new();
    let host = SerialRequest::from_memory(SerialMemoryBudget::new(None), stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    let fixture = NativeFixture::new();
    let (descriptor, basis) = fixture.observe();
    let retained = fixture.retain(&basis);
    drop(basis);
    let mut task = retained.into_retirement();
    crate::branch::basis_registry::assert_successor_registration_preserved(
        &mut task,
        &fixture.history,
        &descriptor,
        &request,
        || fixture.observe().1,
    );
    task.into_retired().unwrap().reclaim();
    assert!(!fixture.history.has_detached_retention_obligations());
}

fn finish(task: &mut RelationalRetainedObservationRetirement, request: &ExecutionRequest<'_, '_>) {
    for _ in 0..12 {
        match task.advance(request) {
            RelationalRetainedObservationRetirementProgress::Advanced => {}
            RelationalRetainedObservationRetirementProgress::Complete => return,
            progress => panic!("unexpected retirement progress: {progress:?}"),
        }
    }
    panic!("original custody did not complete retirement");
}

#[test]
fn single_external_lease_retirement_preserves_an_independent_observation_veto() {
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request = ExecutionRequest::serial(&host);
    let fixture = NativeFixture::new();
    let (_, basis) = fixture.observe();
    let retained = fixture.retain(&basis);
    let accounting = fixture
        .instrumentation
        .external_retention_terminal_accounting();
    let before = accounting.snapshot();
    let mut task = retained.retention.into_observation_retirement();
    drop(retained.observation);
    finish(&mut task, &request);
    let retired = task.into_retired().unwrap();
    assert!(fixture.history.has_detached_retention_obligations());
    assert_eq!(
        accounting.snapshot().explicit_releases,
        before.explicit_releases + 1
    );
    assert_eq!(
        accounting.snapshot().dropped_releases,
        before.dropped_releases
    );
    drop(basis);
    assert!(!fixture.history.has_detached_retention_obligations());
    retired.reclaim();
    assert_eq!(
        accounting.snapshot().explicit_releases,
        before.explicit_releases + 1
    );
    assert_eq!(
        accounting.snapshot().dropped_releases,
        before.dropped_releases
    );
}
