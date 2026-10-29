pub(crate) use {std::path::PathBuf};

pub(crate) use {worth_store_testkit_store::child_process::LOCATOR_ENV, worth_store_testkit_store::child_process::ORACLE_ENV, worth_store_testkit_store::child_process::ROLE_ENV, worth_store_testkit_store::child_process::ROOT_ENV};

pub(crate) fn dispatch(role: &str, root: &std::path::Path) {
    match role {
        "writer" => crate::physical_record_journeys::child_process::record_round_trip::writer(root),
        "reader" => crate::physical_record_journeys::child_process::record_round_trip::reader(root),
        "segment_reader" => crate::physical_record_journeys::child_process::segment_read::run(root),
        "extent_reader" => {
            crate::physical_record_journeys::extent_child::extent_reader(root, &std::env::var(LOCATOR_ENV).unwrap())
        }
        "allocation_writer" => crate::physical_record_journeys::extent_child::allocation_writer(
            root,
            &std::env::var(LOCATOR_ENV).unwrap(),
        ),
        "allocation_reader" => crate::physical_record_journeys::extent_child::allocation_reader(
            root,
            &std::env::var(LOCATOR_ENV).unwrap(),
        ),
        "scale_allocation_reader" => crate::physical_record_journeys::extent_child::scale_allocation_reader(
            root,
            &std::env::var(LOCATOR_ENV).unwrap(),
        ),
        "batch_admission_probe" => crate::physical_record_journeys::child_process::admission_probe::batch(),
        "geometry_admission_probe" => crate::physical_record_journeys::child_process::admission_probe::geometry(root),
        "second_owner" => crate::child_process::ownership_probe::run(root),
        "close_phase_writer" => worth_store_testkit_store::physical_record_journeys::close_phase_crash::writer(root),
        "close_phase_reopener" => worth_store_testkit_store::physical_record_journeys::close_phase_crash::reopener(root),
        "courtroom_writer" => crate::physical_record_journeys::courtroom_child::writer(
            root,
            std::path::PathBuf::from(std::env::var_os(LOCATOR_ENV).unwrap()),
            std::path::PathBuf::from(std::env::var_os(ORACLE_ENV).unwrap()),
        ),
        "courtroom_reopener" => crate::physical_record_journeys::courtroom_child::reopener(
            root,
            std::path::PathBuf::from(std::env::var_os(LOCATOR_ENV).unwrap()),
        ),
        "publication_reopener" => crate::physical_record_journeys::publication_reopener::run(root),
        "residency_pressure_writer" => worth_store_testkit_store::physical_record_journeys::residency_pressure_processes::pressure_writer(
            root,
            &std::env::var(LOCATOR_ENV).unwrap(),
        ),
        "residency_pressure_reader" => worth_store_testkit_store::physical_record_journeys::residency_pressure_processes::pressure_reader(
            root,
            &std::env::var(LOCATOR_ENV).unwrap(),
        ),
        "physical_writeback_writer" => worth_store_testkit_store::physical_record_journeys::residency_writeback_fresh_reopen::writer(root),
        "physical_writeback_observer" => worth_store_testkit_store::physical_record_journeys::residency_writeback_fresh_reopen::observer(
            root,
            &std::env::var(LOCATOR_ENV).unwrap(),
        ),
        "physical_writeback_reopener" => worth_store_testkit_store::physical_record_journeys::residency_writeback_fresh_reopen::reopener(
            root,
            &std::env::var(LOCATOR_ENV).unwrap(),
        ),
        "physical_work_crash_writer" => worth_store_testkit_store::physical_record_journeys::physical_work::failure::recovery::crash_writer(root),
        "physical_work_crash_reopener" => {
            worth_store_testkit_store::physical_record_journeys::physical_work::failure::recovery::crash_reopener(root)
        }
        "phase16_maelstrom_reopener" => {
            crate::physical_record_journeys::physical_work::phase_16_maelstrom_reopener(root)
        }
        "interference_baseline_reopener" => {
            worth_store_testkit_store::physical_record_journeys::maintenance_interference::reopen::serving_child(root)
        }
        "interference_canonical_child" => {
            worth_store_testkit_store::physical_record_journeys::maintenance_interference::canonical::canonical_child(root)
        }
        _ => panic!("unknown child role"),
    }
}