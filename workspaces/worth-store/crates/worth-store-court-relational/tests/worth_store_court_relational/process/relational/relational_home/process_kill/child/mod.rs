//! Child execution of the process-kill plan; all parks use the existing Store gates.
#[allow(unused_imports)]
use crate::process_kill::child::*;
use {
    crate::process_kill::child_wal_profile, crate::process_kill::LOCATION_ENV,
    crate::process_kill::MARKERS_ENV, crate::process_kill::ROLE_ENV,
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
