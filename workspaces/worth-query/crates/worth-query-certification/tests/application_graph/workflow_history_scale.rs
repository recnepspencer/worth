//! History and unrelated-instance axes through real workflow publication owners.
//! This court does not claim geometry population or authored-node qualification.

use std::time::Instant;
use worth_query_host::facade::{
    application_entry::{
        PublishedWorkflowDefinitionRef, PublishedWorkflowInstanceRef,
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
        WorkflowInstanceStartOutcome, WorkflowProposalOutcome,
    },
    declaration::application_program::ApplicationWorkflowComponentLimits,
};
use worth_query_installation::facade::WorthQueryApplicationWorkflowResourceCeiling;
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
        .unwrap();
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
            if self.history % 2 == 0 {
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

    fn measure_warm(&mut self, instances: usize) {
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
        assert_eq!(after.denials(), progress.denials());
        assert_eq!(compiled.cold_misses(), compilation.cold_misses());
        assert_eq!(compiled.cold_retains(), compilation.cold_retains());
        assert!(after.retained_charge_bytes() <= after.maximum_retained_charge_bytes());
        micros.sort_unstable();
        println!("workflow history={history} instances={instances} samples={SAMPLES} p50_us={} p95_us={} progress_retained={} plan_retained={} plan_peak={}", micros[9], micros[18], after.retained_charge_bytes(), compiled.retained_bytes(), compiled.peak_retained_bytes());
        println!("native history={history} samples={SAMPLES} copied_truth_bytes={} new_authoritative_bytes={} touched_regions={} reused_regions={} world_commits={} world_observations={}", native_delta.copied_truth_bytes, native_delta.publication_new_authoritative_bytes, native_delta.publication_touched_region_count, native_delta.publication_reused_region_count, native.world_history_after().installed_commits(), native.world_retention_after().observations());
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
#[ignore = "known cold decision-fact budget failure; diagnostic until owner correction"]
fn workflow_history_locality_smoke() {
    let mut court = HistoryCourt::new();
    court.fill(100);
    court.measure_warm(1);
    court.reconstruct();
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
#[ignore = "scheduled history qualification is currently red on native copy cost; run query-workflow-history-scale"]
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
