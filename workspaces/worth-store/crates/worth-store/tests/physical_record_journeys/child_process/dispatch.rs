use std::path::PathBuf;

use super::{LOCATOR_ENV, ORACLE_ENV, ROLE_ENV, ROOT_ENV};

#[test]
fn c5_child_role() {
    let Ok(role) = std::env::var(ROLE_ENV) else {
        return;
    };
    let root = PathBuf::from(std::env::var_os(ROOT_ENV).unwrap());
    dispatch(&role, &root);
}

fn dispatch(role: &str, root: &std::path::Path) {
    match role {
        "writer" => super::record_round_trip::writer(root),
        "reader" => super::record_round_trip::reader(root),
        "segment_reader" => super::segment_read::run(root),
        "extent_reader" => {
            super::super::extent_child::extent_reader(root, &std::env::var(LOCATOR_ENV).unwrap())
        }
        "allocation_writer" => super::super::extent_child::allocation_writer(
            root,
            &std::env::var(LOCATOR_ENV).unwrap(),
        ),
        "allocation_reader" => super::super::extent_child::allocation_reader(
            root,
            &std::env::var(LOCATOR_ENV).unwrap(),
        ),
        "scale_allocation_reader" => super::super::extent_child::scale_allocation_reader(
            root,
            &std::env::var(LOCATOR_ENV).unwrap(),
        ),
        "batch_admission_probe" => super::admission_probe::batch(),
        "geometry_admission_probe" => super::admission_probe::geometry(root),
        "second_owner" => super::ownership_probe::run(root),
        "close_phase_writer" => super::super::close_phase_crash::writer(root),
        "close_phase_reopener" => super::super::close_phase_crash::reopener(root),
        "courtroom_writer" => super::super::courtroom_child::writer(
            root,
            PathBuf::from(std::env::var_os(LOCATOR_ENV).unwrap()),
            PathBuf::from(std::env::var_os(ORACLE_ENV).unwrap()),
        ),
        "courtroom_reopener" => super::super::courtroom_child::reopener(
            root,
            PathBuf::from(std::env::var_os(LOCATOR_ENV).unwrap()),
        ),
        "publication_reopener" => super::super::publication_reopener::run(root),
        "residency_pressure_writer" => super::super::residency_pressure_processes::pressure_writer(
            root,
            &std::env::var(LOCATOR_ENV).unwrap(),
        ),
        "residency_pressure_reader" => super::super::residency_pressure_processes::pressure_reader(
            root,
            &std::env::var(LOCATOR_ENV).unwrap(),
        ),
        "physical_writeback_writer" => super::super::residency_writeback_fresh_reopen::writer(root),
        "physical_writeback_observer" => super::super::residency_writeback_fresh_reopen::observer(
            root,
            &std::env::var(LOCATOR_ENV).unwrap(),
        ),
        "physical_writeback_reopener" => super::super::residency_writeback_fresh_reopen::reopener(
            root,
            &std::env::var(LOCATOR_ENV).unwrap(),
        ),
        "physical_work_crash_writer" => super::super::physical_work::failure::crash_writer(root),
        "physical_work_crash_reopener" => {
            super::super::physical_work::failure::crash_reopener(root)
        }
        "phase16_maelstrom_reopener" => {
            super::super::physical_work::phase_16_maelstrom_reopener(root)
        }
        "interference_baseline_reopener" => {
            super::super::maintenance_interference::serving_child(root)
        }
        "interference_canonical_child" => {
            super::super::maintenance_interference::canonical_child(root)
        }
        _ => panic!("unknown child role"),
    }
}
