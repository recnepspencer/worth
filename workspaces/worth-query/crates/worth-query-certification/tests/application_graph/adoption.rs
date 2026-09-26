//! What a host's rostered programs mean before anything adopts a new one.

#[path = "adoption/branch_adoption.rs"]
mod branch_adoption;
#[path = "adoption/branch_adoption_recovery.rs"]
mod branch_adoption_recovery;
#[path = "adoption/branch_divergence.rs"]
mod branch_divergence;
#[path = "adoption/broader_scope/mod.rs"]
mod broader_scope;
#[path = "adoption/consumer_closure.rs"]
mod consumer_closure;
#[path = "adoption/custody.rs"]
mod custody;
#[path = "adoption/migration.rs"]
mod migration;
#[path = "adoption/program_codec.rs"]
mod program_codec;
#[path = "adoption/recovery_terminals.rs"]
mod recovery_terminals;
#[path = "adoption/roster_admission.rs"]
mod roster_admission;
#[path = "adoption/semantic_impact.rs"]
mod semantic_impact;
#[path = "adoption/signal_fork_inheritance.rs"]
mod signal_fork_inheritance;
#[path = "adoption/support_retirement.rs"]
mod support_retirement;
#[path = "adoption/support_retirement_races.rs"]
mod support_retirement_races;
#[path = "adoption/workflow_custody.rs"]
mod workflow_custody;
#[path = "adoption/workflow_identity.rs"]
mod workflow_identity;
#[path = "adoption/workflow_participant.rs"]
mod workflow_participant;
#[path = "adoption/workflow_participant_races.rs"]
mod workflow_participant_races;
#[path = "adoption/workflow_vocabulary.rs"]
mod workflow_vocabulary;
