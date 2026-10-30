//! Child execution of the process-kill plan; all parks use the existing Store gates.
#[allow(unused_imports)]
use crate::process_kill::child::*;
use std::path::Path;
use {
    crate::durable_run::apply, crate::durable_run::open, crate::killed_child,
    crate::process_kill::await_mutation_seam, crate::process_kill::child_wal_profile,
    crate::process_kill::encode, crate::process_kill::script, crate::process_kill::Stop,
    crate::process_kill::LOCATION_ENV, crate::process_kill::MARKERS_ENV,
    crate::process_kill::ROLE_ENV, crate::workload, crate::workload::Step, std::path::PathBuf,
    std::time::Instant, worth_execution::ExecutionRequest,
    worth_store::physical_runtime::production::PhysicalCheckpointStep,
};

#[test]
fn relational_home_child() {
    let Some(role) = std::env::var_os(ROLE_ENV) else {
        return;
    };
    let stop = crate::process_kill::Stop::all()
        .find(|stop| Some(stop.label().as_str()) == role.to_str())
        .expect("a known child role");
    let location =
        std::path::PathBuf::from(std::env::var_os(LOCATION_ENV).expect("child location"));
    let markers = std::path::PathBuf::from(std::env::var_os(MARKERS_ENV).expect("child markers"));
    let run = || child(stop, &location, &markers);
    match child_wal_profile() {
        Some(profile) => profile.run(run),
        None => run(),
    }
}
