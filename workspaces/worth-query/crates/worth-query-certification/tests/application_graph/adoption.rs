//! What a host's rostered programs mean before anything adopts a new one.

#[path = "adoption/branch_adoption.rs"]
mod branch_adoption;
#[path = "adoption/branch_adoption_recovery.rs"]
mod branch_adoption_recovery;
#[path = "adoption/branch_divergence.rs"]
mod branch_divergence;
#[path = "adoption/migration.rs"]
mod migration;
#[path = "adoption/recovery_terminals.rs"]
mod recovery_terminals;
#[path = "adoption/roster_admission.rs"]
mod roster_admission;
#[path = "adoption/semantic_impact.rs"]
mod semantic_impact;
#[path = "adoption/signal_fork_inheritance.rs"]
mod signal_fork_inheritance;
