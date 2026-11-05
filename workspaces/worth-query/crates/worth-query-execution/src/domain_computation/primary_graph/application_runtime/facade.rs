//! The runtime's consumer methods: each only calls the subsystem that owns the work.
mod application_attempt;
mod application_checkpoint;
mod application_discovery;
mod application_entry;
mod application_installation;
mod application_resource;
mod authorization;
mod causal_inspection;
mod conditional_operation;
mod operation_body;
mod provider;
