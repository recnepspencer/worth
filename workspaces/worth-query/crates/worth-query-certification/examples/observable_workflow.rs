//! Neutral owning workflow: capture, condition, distinct approval and real archival ingress.
//!
//! `cargo run -p worth-query-certification --example observable_workflow` writes
//! an archive in a fresh temporary directory through a separate process of this
//! executable. Its installed verifier authenticates the owned local pipe; this
//! is a local archive protocol, not a network-signature or crash-recovery claim.
//! Query retains the original dispatch until genuine ingress and native settlement.
#[path = "observable_workflow/archive.rs"]
mod archive;
#[path = "observable_workflow/definition.rs"]
mod definition;
#[path = "observable_workflow/discovery.rs"]
mod discovery;
#[path = "observable_workflow/model.rs"]
#[allow(dead_code, unused_imports)]
mod document_retention_model;
#[path = "observable_workflow/effect.rs"]
mod effect;
#[path = "observable_workflow/history.rs"]
mod history;
#[path = "observable_workflow/journey.rs"]
mod journey;
#[path = "observable_workflow/operations.rs"]
mod operations;
#[path = "observable_workflow/program.rs"]
mod program;
#[path = "observable_workflow/restoration.rs"]
mod restoration;

fn main() {
    match archive::handle_worker() {
        Ok(true) => (),
        Ok(false) => journey::run(),
        Err(cause) => {
            eprintln!("archive worker failed: {cause}");
            std::process::exit(1);
        }
    }
}
