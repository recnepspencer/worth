//! What a host's rostered programs mean before anything adopts a new one.

#[path = "adoption/checkpoint_transition.rs"]
pub mod checkpoint_transition;

#[path = "adoption/branch_adoption.rs"]
pub mod branch_adoption;
#[path = "adoption/branch_adoption_recovery.rs"]
pub mod branch_adoption_recovery;
#[path = "adoption/branch_divergence.rs"]
pub mod branch_divergence;
#[path = "adoption/broader_scope/mod.rs"]
pub mod broader_scope;
#[path = "adoption/consumer_closure.rs"]
pub mod consumer_closure;
#[path = "adoption/custody.rs"]
pub mod custody;
#[path = "adoption/live_outputs.rs"]
pub mod live_outputs;
#[path = "adoption/migration.rs"]
pub mod migration;
#[path = "adoption/program_codec.rs"]
pub mod program_codec;
#[path = "adoption/roster_admission.rs"]
pub mod roster_admission;
#[path = "adoption/rostered_restore.rs"]
pub mod rostered_restore;
#[path = "adoption/semantic_impact.rs"]
pub mod semantic_impact;
#[path = "adoption/signal_fork_inheritance.rs"]
pub mod signal_fork_inheritance;
#[path = "adoption/support_retirement.rs"]
pub mod support_retirement;
#[path = "adoption/support_retirement_races.rs"]
pub mod support_retirement_races;
#[path = "adoption/workflow_custody.rs"]
pub mod workflow_custody;
#[path = "adoption/workflow_identity.rs"]
pub mod workflow_identity;
#[path = "adoption/workflow_participant.rs"]
pub mod workflow_participant;
#[path = "adoption/workflow_participant_races.rs"]
pub mod workflow_participant_races;
#[path = "adoption/workflow_pinned_basis.rs"]
pub mod workflow_pinned_basis;
#[path = "adoption/workflow_vocabulary.rs"]
pub mod workflow_vocabulary;

#[path = "adoption/request_lifetime.rs"]
pub mod request_lifetime;

#[path = "adoption/authorization.rs"]
pub mod authorization;

#[path = "adoption/workflow_runtime_binding.rs"]
pub mod workflow_runtime_binding;
