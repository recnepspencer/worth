//! Record fixture child roles shared by court executables.
use super::{admission_probe, record_round_trip};
use crate::child_process::{LOCATOR_ENV, ROLE_ENV, ROOT_ENV};
use crate::physical_record_journeys::{extent_child, residency_pressure_processes};

pub fn dispatch_child() {
    let Ok(role) = std::env::var(ROLE_ENV) else {
        return;
    };
    let root = std::path::PathBuf::from(std::env::var_os(ROOT_ENV).unwrap());
    match role.as_str() {
        "writer" => record_round_trip::writer(&root),
        "reader" => record_round_trip::reader(&root),
        "extent_reader" => extent_child::extent_reader(&root, &std::env::var(LOCATOR_ENV).unwrap()),
        "allocation_writer" => {
            extent_child::allocation_writer(&root, &std::env::var(LOCATOR_ENV).unwrap())
        }
        "allocation_reader" => {
            extent_child::allocation_reader(&root, &std::env::var(LOCATOR_ENV).unwrap())
        }
        "scale_allocation_reader" => {
            extent_child::scale_allocation_reader(&root, &std::env::var(LOCATOR_ENV).unwrap())
        }
        "batch_admission_probe" => admission_probe::batch(),
        "geometry_admission_probe" => admission_probe::geometry(&root),
        "residency_pressure_writer" => residency_pressure_processes::pressure_writer(
            &root,
            &std::env::var(LOCATOR_ENV).unwrap(),
        ),
        "residency_pressure_reader" => residency_pressure_processes::pressure_reader(
            &root,
            &std::env::var(LOCATOR_ENV).unwrap(),
        ),
        _ => panic!("unknown child role"),
    }
}
