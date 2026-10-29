//! History and unrelated-instance axes through real workflow publication owners.
//! This court does not claim sparse-definition publication or authored-node qualification.

pub use std::time::Instant;
pub use worth_query_host::facade::{
    application_entry::{
        PublishedWorkflowDefinitionRef, PublishedWorkflowInstanceRef,
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
        WorkflowInstanceStartOutcome, WorkflowProposalOutcome, WorkflowProposalPreparationDenial,
        WorthQueryWorkflowProposalPreparationDenial,
    },
    declaration::application_program::ApplicationWorkflowComponentLimits,
    primary_graph::WorthQueryApplicationAttemptDenialKind,
};
pub use worth_query_installation::facade::{
    WorthQueryApplicationWorkflowResourceCeiling, WorthQueryWorkflowHistoryReconstructionBudget,
};
pub use worth_query_replay::facade::WorthQueryCertificationCostRuntimeExt;

pub use super::document_retention_model::{
    host::{publish_on_first_program, DocumentWorkflowRuntime},
    workflow::{
        bounded_retry_definition_with_attempts, propose_authoring_instance, publish_definition,
        retain_workflow_with_resources, start_instance,
    },
};

pub const SAMPLES: u64 = 20;
pub const MAXIMUM_HISTORY: u32 = 10_128;

pub struct HistoryCourt {
    pub application: DocumentWorkflowRuntime,
    pub definition: PublishedWorkflowDefinitionRef,
    pub instance: PublishedWorkflowInstanceRef,
    pub next_key: u64,
    pub history: u64,
}

impl HistoryCourt {
    pub fn new() -> Self {
        Self::with_budget(WorthQueryWorkflowHistoryReconstructionBudget::standard())
    }

    pub fn with_budget(budget: WorthQueryWorkflowHistoryReconstructionBudget) -> Self {
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
        let application = retain_workflow_with_resources(publish_on_first_program(), resources);
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

    pub fn advance(&mut self) {
        let performed = match propose_authoring_instance(
            &self.application,
            self.instance.clone(),
            self.next_key,
        )
        .unwrap_or_else(|denial| panic!("history {} preparation failed: {denial:?}", self.history))
        {
            WorkflowProposalOutcome::Published(performed) => performed,
            WorkflowProposalOutcome::Application(
                worth_query_host::facade::primary_graph::WorthQueryApplicationUncommitted::NoEffect(
                    denial,
                ),
            ) => panic!("history {} no effect: {:?}", self.history, denial.cause()),
            other => panic!(
                "history {} proposal failed with outcome {:?}",
                self.history,
                std::mem::discriminant(&other)
            ),
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

    pub fn fill(&mut self, history: u64) {
        while self.history < history {
            self.advance();
        }
    }

    pub fn unrelated_instance(&mut self) {
        let instance = started(&self.application, self.definition.clone(), self.next_key);
        self.next_key += 1;
        assert!(matches!(
            propose_authoring_instance(&self.application, instance, self.next_key).unwrap(),
            WorkflowProposalOutcome::Published(_)
        ));
        self.next_key += 1;
    }

    pub fn measure_warm(&mut self, instances: usize) -> u64 {
        let history = self.history;
        let native_scope = self
            .application
            .with_host_advancement(|host_phase| {
                self.application
                    .runtime()
                    .capture_certification_cost_scope(&host_phase, self.application.current_world())
                    .expect("the measured branch must remain admitted")
            })
            .expect("installed host call opens");
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
            .with_host_advancement(|host_phase| {
                self.application
                    .runtime()
                    .observe_certification_cost(&host_phase, &native_scope)
                    .expect("native owners must remain inspectable")
            })
            .expect("installed host call opens");
        let native_delta = native.relational().sharing_cost_delta();
        // History behind each settled publication retires, so World holds
        // the same commits however many samples publish.
        assert_eq!(
            native.world_history_after().installed_commits(),
            native.world_history_before().installed_commits()
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

    pub fn reconstruct(&mut self) {
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

pub fn started(
    application: &DocumentWorkflowRuntime,
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
