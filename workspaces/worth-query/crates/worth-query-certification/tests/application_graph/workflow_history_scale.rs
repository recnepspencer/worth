//! History and unrelated-instance axes through real workflow publication owners.
//! This court does not claim geometry population or authored-node qualification.

use std::time::Instant;
use worth_query_host::facade::{
    application_entry::{
        PublishedWorkflowDefinitionRef, PublishedWorkflowInstanceRef,
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
        WorkflowInstanceStartOutcome, WorkflowProposalOutcome, WorkflowProposalPreparationDenial,
        WorthQueryWorkflowProposalPreparationDenial,
    },
    declaration::application_program::ApplicationWorkflowComponentLimits,
    primary_graph::WorthQueryApplicationAttemptDenialKind,
};
use worth_query_installation::facade::{
    WorthQueryApplicationWorkflowResourceCeiling, WorthQueryWorkflowHistoryReconstructionBudget,
};
use worth_query_replay::facade::WorthQueryCertificationCostRuntimeExt;

use super::bounded_dimension_model::{
    host::{publish_on_first_program_for_history_scale, BoundedDimensionWorkflowRuntime},
    workflow::{
        bounded_retry_definition_with_attempts, propose_authoring_instance, publish_definition,
        retain_workflow_with_resources, start_instance,
    },
};

const SAMPLES: u64 = 20;
const MAXIMUM_HISTORY: u32 = 10_128;

struct HistoryCourt {
    application: BoundedDimensionWorkflowRuntime,
    definition: PublishedWorkflowDefinitionRef,
    instance: PublishedWorkflowInstanceRef,
    next_key: u64,
    history: u64,
}

impl HistoryCourt {
    fn new() -> Self {
        Self::with_budget(WorthQueryWorkflowHistoryReconstructionBudget::standard())
    }

    fn with_budget(budget: WorthQueryWorkflowHistoryReconstructionBudget) -> Self {
        let resources = WorthQueryApplicationWorkflowResourceCeiling::new(
            32,
            64,
            4,
            ApplicationWorkflowComponentLimits::new(32, 4, 128, 256, 256).unwrap(),
            64 * 1024,
            101,
            MAXIMUM_HISTORY,
            256 * 1024,
        )
        .unwrap()
        .with_history_reconstruction_budget(budget);
        let application =
            retain_workflow_with_resources(publish_on_first_program_for_history_scale(), resources);
        let definition = match publish_definition(
            &application,
            bounded_retry_definition_with_attempts(6_000),
            WorkflowDefinitionExpectedPredecessor::Absent,
            30_000,
        )
        .unwrap()
        {
            WorkflowDefinitionPublicationOutcome::Published(performed) => {
                performed.definition().clone()
            }
            other => panic!(
                "history court definition publication failed: {:?}",
                std::mem::discriminant(&other)
            ),
        };
        let instance = started(&application, definition.clone(), 30_001);
        Self {
            application,
            definition,
            instance,
            next_key: 30_002,
            history: 0,
        }
    }

    fn advance(&mut self) {
        let performed = match propose_authoring_instance(
            &self.application,
            self.instance.clone(),
            self.next_key,
        )
        .unwrap_or_else(|denial| panic!("history {} preparation failed: {denial:?}", self.history))
        {
            WorkflowProposalOutcome::Published(performed) => performed,
            WorkflowProposalOutcome::Application(worth_query_host::facade::primary_graph::WorthQueryApplicationCommitOutcome::NoEffect(denial)) => panic!("history {} no effect: {:?}", self.history, denial.cause()),
            other => panic!("history {} proposal failed with outcome {:?}", self.history, std::mem::discriminant(&other)),
        };
        assert!(!performed.replayed());
        assert_eq!(
            performed.node_path(),
            if self.history.is_multiple_of(2) {
                "proposal/first"
            } else {
                "proposal/revise"
            }
        );
        self.next_key += 1;
        self.history += 1;
    }

    fn fill(&mut self, history: u64) {
        while self.history < history {
            self.advance();
        }
    }

    fn unrelated_instance(&mut self) {
        let instance = started(&self.application, self.definition.clone(), self.next_key);
        self.next_key += 1;
        assert!(matches!(
            propose_authoring_instance(&self.application, instance, self.next_key).unwrap(),
            WorkflowProposalOutcome::Published(_)
        ));
        self.next_key += 1;
    }

    fn measure_warm(&mut self, instances: usize) -> u64 {
        let history = self.history;
        let native_scope = self
            .application
            .runtime()
            .capture_certification_cost_scope(self.application.current_world())
            .expect("the measured branch must remain admitted");
        let progress = self
            .application
            .runtime()
            .workflow_instance_progress_counters();
        let compilation = self
            .application
            .runtime()
            .workflow_compilation_reuse_counters();
        let mut micros = Vec::new();
        for _ in 0..SAMPLES {
            let start = Instant::now();
            self.advance();
            micros.push(start.elapsed().as_micros());
        }
        let after = self
            .application
            .runtime()
            .workflow_instance_progress_counters();
        let compiled = self
            .application
            .runtime()
            .workflow_compilation_reuse_counters();
        let native = self
            .application
            .runtime()
            .observe_certification_cost(&native_scope)
            .expect("native owners must remain inspectable");
        let native_delta = native.relational().sharing_cost_delta();
        assert_eq!(
            native.world_history_after().installed_commits()
                - native.world_history_before().installed_commits(),
            SAMPLES as usize
        );
        assert!(native_delta.publication_new_authoritative_bytes > 0);
        assert_eq!(native.world_retention_after().observations(), 0);
        assert_eq!(
            after.incremental_advances() - progress.incremental_advances(),
            SAMPLES as usize
        );
        assert_eq!(after.cold_misses(), progress.cold_misses());
        assert_eq!(
            after.cold_reconstruction_transition_visits(),
            progress.cold_reconstruction_transition_visits()
        );
        assert_eq!(
            after.warm_history_transition_visits(),
            progress.warm_history_transition_visits()
        );
        assert_eq!(after.evictions(), progress.evictions());
        assert_eq!(
            after.history_reconstruction_charge_bytes(),
            progress.history_reconstruction_charge_bytes()
        );
        assert_eq!(after.denials(), progress.denials());
        assert_eq!(compiled.cold_misses(), compilation.cold_misses());
        assert_eq!(compiled.cold_retains(), compilation.cold_retains());
        assert!(after.retained_charge_bytes() <= after.maximum_retained_charge_bytes());
        micros.sort_unstable();
        println!("workflow history={history} instances={instances} samples={SAMPLES} p50_us={} p95_us={} progress_retained={} plan_retained={} plan_peak={}", micros[9], micros[18], after.retained_charge_bytes(), compiled.retained_bytes(), compiled.peak_retained_bytes());
        println!("native history={history} samples={SAMPLES} copied_truth_bytes={} new_authoritative_bytes={} content_values_hashed={} touched_regions={} reused_regions={} world_commits={} world_observations={}", native_delta.copied_truth_bytes, native_delta.publication_new_authoritative_bytes, native_delta.publication_content_values_hashed, native_delta.publication_touched_region_count, native_delta.publication_reused_region_count, native.world_history_after().installed_commits(), native.world_retention_after().observations());
        assert!(native_delta.publication_content_values_hashed > 0);
        let sharing = native.relational().sharing();
        println!(
            "native live history={history} payload={} canonical={} optional_cache={}",
            sharing.unique_physical_partition_payload_bytes(),
            sharing.unique_physical_canonical_commit_bytes(),
            sharing.unique_optional_cache_bytes()
        );
        native_delta.publication_content_values_hashed
    }

    fn reconstruct(&mut self) {
        let history = self.history;
        self.application
            .runtime()
            .release_workflow_instance_progress_for_test();
        let released = self
            .application
            .runtime()
            .workflow_instance_progress_counters();
        assert_eq!(released.retained_charge_bytes(), 0);
        let start = Instant::now();
        self.advance();
        let rebuilt = self
            .application
            .runtime()
            .workflow_instance_progress_counters();
        assert_eq!(rebuilt.cold_misses(), released.cold_misses() + 1);
        assert!(
            rebuilt.history_reconstruction_charge_bytes()
                > released.history_reconstruction_charge_bytes()
        );
        assert!(
            rebuilt.peak_history_reconstruction_charge_bytes()
                <= WorthQueryWorkflowHistoryReconstructionBudget::standard().maximum_charge_bytes()
                    as usize
        );
        assert_eq!(
            rebuilt.cold_reconstruction_transition_visits()
                - released.cold_reconstruction_transition_visits(),
            history as usize
        );
        assert_eq!(
            rebuilt.incremental_advances(),
            released.incremental_advances() + 1
        );
        println!(
            "workflow reconstruction history={history} elapsed_us={}",
            start.elapsed().as_micros()
        );
    }
}

fn started(
    application: &BoundedDimensionWorkflowRuntime,
    definition: PublishedWorkflowDefinitionRef,
    key: u64,
) -> PublishedWorkflowInstanceRef {
    match start_instance(application, definition, key).unwrap() {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!(
            "history court instance start failed: {:?}",
            std::mem::discriminant(&other)
        ),
    }
}

#[test]
fn workflow_history_locality_smoke() {
    let mut court = HistoryCourt::new();
    court.fill(100);
    court.measure_warm(1);
    court.reconstruct();
}

#[test]
fn workflow_history_reconstruction_denies_work_and_memory_exhaustion_without_effects() {
    for (budget, expected_subject) in [
        (
            WorthQueryWorkflowHistoryReconstructionBudget::new(1, 128 * 1024 * 1024).unwrap(),
            "workflow history reconstruction inventory ceiling",
        ),
        (
            WorthQueryWorkflowHistoryReconstructionBudget::new(100, 32 * 1024).unwrap(),
            "workflow history reconstruction memory ceiling",
        ),
    ] {
        let mut court = HistoryCourt::with_budget(budget);
        court.fill(4);
        court
            .application
            .runtime()
            .release_workflow_instance_progress_for_test();
        let before = court
            .application
            .runtime()
            .workflow_instance_progress_counters();
        let scope = court
            .application
            .runtime()
            .capture_certification_cost_scope(court.application.current_world())
            .unwrap();
        for _ in 0..2 {
            let denial = match propose_authoring_instance(
                &court.application,
                court.instance.clone(),
                court.next_key,
            ) {
                Err(WorthQueryWorkflowProposalPreparationDenial::ProposalPreparation(
                    WorkflowProposalPreparationDenial::Attempt(denial),
                )) => denial,
                Err(other) => panic!("unexpected history reconstruction denial: {other:?}"),
                Ok(_) => panic!("history reconstruction must deny before a publication outcome"),
            };
            assert_eq!(
                denial.kind(),
                WorthQueryApplicationAttemptDenialKind::WorkflowHistoryReconstructionBudgetExceeded
            );
            assert_eq!(denial.subject(), expected_subject);
            let after = court
                .application
                .runtime()
                .workflow_instance_progress_counters();
            assert_eq!(after.retained_charge_bytes(), 0);
            assert_eq!(after.cold_retains(), before.cold_retains());
        }
        let cost = court
            .application
            .runtime()
            .observe_certification_cost(&scope)
            .unwrap();
        assert_eq!(
            cost.world_history_before().installed_commits(),
            cost.world_history_after().installed_commits()
        );
        assert_eq!(cost.world_retention_after().observations(), 0);
    }
}

#[test]
#[ignore = "bounded diagnostic of native history-copy cost, not scale qualification"]
fn workflow_history_native_cost_diagnostic() {
    let mut court = HistoryCourt::new();
    for history in [100, 200, 400] {
        court.fill(history);
        court.measure_warm(1);
    }
}

#[test]
#[ignore = "scheduled history qualification; run query-workflow-history-scale"]
fn workflow_history_and_instance_scale() {
    let mut court = HistoryCourt::new();
    for history in [100, 1_000, 10_000] {
        court.fill(history);
        court.measure_warm(1);
    }
    court.reconstruct();
    drop(court);
    for instances in [1, 10, 100] {
        let mut court = HistoryCourt::new();
        court.fill(100);
        for _ in 1..instances {
            court.unrelated_instance();
        }
        court.measure_warm(instances);
    }
}
