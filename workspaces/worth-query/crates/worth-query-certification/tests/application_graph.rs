//! Certification of the application graph: what a branch's own program means
//! for the rules it actually enforces.
#![recursion_limit = "256"]

#[path = "application_graph/adoption.rs"]
mod adoption;
#[path = "application_graph/allocation_custody.rs"]
mod allocation_custody;
#[path = "application_graph/canonical_admission.rs"]
mod canonical_admission;
#[path = "application_graph/canonical_identity.rs"]
mod canonical_identity;
#[path = "application_graph/canonical_transition.rs"]
mod canonical_transition;
#[path = "application_graph/canonical_value.rs"]
mod canonical_value;
#[path = "application_graph/correction.rs"]
mod correction;
#[path = "application_graph/document_retention_model.rs"]
mod document_retention_model;
#[path = "application_graph/expressions/conditions.rs"]
mod expression_conditions;
#[path = "application_graph/fork_decision_reads.rs"]
mod fork_decision_reads;
#[path = "application_graph/history_retirement.rs"]
mod history_retirement;
#[path = "application_graph/mutation_binding_guard.rs"]
mod mutation_binding_guard;
#[path = "application_graph/producer_predicate_checkpoint.rs"]
mod producer_predicate_checkpoint;
#[path = "application_graph/recorded_replay_ordering.rs"]
mod recorded_replay_ordering;
#[path = "application_graph/restored_primary_backend.rs"]
mod restored_primary_backend;
#[path = "application_graph/restored_replay.rs"]
mod restored_replay;
#[path = "application_graph/source_only.rs"]
mod source_only;
#[path = "support/execution_policy.rs"]
mod test_request_policy;
#[path = "application_graph/unpublished_recovery.rs"]
mod unpublished_recovery;
#[path = "application_graph/workflow.rs"]
mod workflow;
#[path = "application_graph/workflow_actor_wait.rs"]
mod workflow_actor_wait;
#[path = "application_graph/workflow_approval.rs"]
mod workflow_approval;
#[path = "application_graph/workflow_assessment.rs"]
mod workflow_assessment;
#[path = "application_graph/workflow_compilation.rs"]
mod workflow_compilation;
#[path = "application_graph/workflow_component_scale.rs"]
mod workflow_component_scale;
#[path = "application_graph/workflow_condition.rs"]
mod workflow_condition;
#[path = "application_graph/workflow_control_scale.rs"]
mod workflow_control_scale;
#[path = "application_graph/workflow_definition_readmission.rs"]
mod workflow_definition_readmission;
#[path = "application_graph/workflow_draft_archive.rs"]
mod workflow_draft_archive;
#[path = "application_graph/workflow_effect_resources.rs"]
mod workflow_effect_resources;
#[path = "application_graph/workflow_execution.rs"]
mod workflow_execution;
#[path = "application_graph/workflow_history_scale.rs"]
mod workflow_history_scale;
#[path = "application_graph/workflow_navigation.rs"]
mod workflow_navigation;
#[path = "application_graph/workflow_operation_input.rs"]
mod workflow_operation_input;
#[path = "application_graph/workflow_ordinary.rs"]
mod workflow_ordinary;
#[path = "application_graph/workflow_ordinary_component.rs"]
mod workflow_ordinary_component;
#[path = "application_graph/workflow_ordinary_control.rs"]
mod workflow_ordinary_control;
#[path = "application_graph/workflow_ordinary_currentness.rs"]
mod workflow_ordinary_currentness;
#[path = "application_graph/workflow_ordinary_multi_subject.rs"]
mod workflow_ordinary_multi_subject;
#[path = "application_graph/workflow_ordinary_navigation.rs"]
mod workflow_ordinary_navigation;
#[path = "application_graph/workflow_progress_retention.rs"]
mod workflow_progress_retention;
#[path = "application_graph/workflow_proposal.rs"]
mod workflow_proposal;
#[path = "application_graph/workflow_publication_scale.rs"]
mod workflow_publication_scale;
#[path = "application_graph/workflow_receipt_lifecycle.rs"]
mod workflow_receipt_lifecycle;
#[path = "application_graph/workflow_retirement.rs"]
mod workflow_retirement;
#[path = "application_graph/workflow_retry.rs"]
mod workflow_retry;
#[path = "application_graph/workflow_start_intent.rs"]
mod workflow_start_intent;
#[path = "application_graph/workflow_vocabulary.rs"]
mod workflow_vocabulary;

#[path = "application_graph/source_bound_replay.rs"]
mod source_bound_replay;

#[path = "application_graph/lease_quote.rs"]
mod lease_quote;
#[path = "application_graph/native_staging_allocation.rs"]
mod native_staging_allocation;

#[path = "support/process_deadline.rs"]
mod process_deadline;

#[path = "application_graph/workflow_owning.rs"]
mod workflow_owning;
