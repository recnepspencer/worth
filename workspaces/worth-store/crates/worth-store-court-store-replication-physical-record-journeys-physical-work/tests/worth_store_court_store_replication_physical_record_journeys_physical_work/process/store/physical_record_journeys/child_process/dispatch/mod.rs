use worth_store_testkit_store::child_process::{ROLE_ENV, ROOT_ENV};
use worth_store_testkit_store::physical_record_journeys::{close_phase_crash, physical_work};

#[test]
fn physical_record_child_role() {
    let Ok(role) = std::env::var(ROLE_ENV) else {
        return;
    };
    let root = std::path::PathBuf::from(std::env::var_os(ROOT_ENV).unwrap());
    match role.as_str() {
        "close_phase_writer" => close_phase_crash::writer(&root),
        "close_phase_reopener" => close_phase_crash::reopener(&root),
        "physical_work_crash_writer" => physical_work::failure::recovery::crash_writer(&root),
        "physical_work_crash_reopener" => physical_work::failure::recovery::crash_reopener(&root),
        _ => panic!("unknown physical-work child role: {role}"),
    }
}
