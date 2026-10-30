use worth_store_testkit_store::child_process::{LOCATOR_ENV, ROLE_ENV, ROOT_ENV};
use worth_store_testkit_store::physical_record_journeys::residency_writeback_fresh_reopen;

#[test]
fn physical_record_child_role() {
    let Ok(role) = std::env::var(ROLE_ENV) else {
        return;
    };
    let root = std::path::PathBuf::from(std::env::var_os(ROOT_ENV).expect("child root"));
    match role.as_str() {
        "physical_writeback_writer" => residency_writeback_fresh_reopen::writer(&root),
        "physical_writeback_observer" => residency_writeback_fresh_reopen::observer(
            &root,
            &std::env::var(LOCATOR_ENV).expect("writer digest"),
        ),
        "physical_writeback_reopener" => residency_writeback_fresh_reopen::reopener(
            &root,
            &std::env::var(LOCATOR_ENV).expect("writer locator"),
        ),
        _ => panic!("unknown physical writeback child role: {role}"),
    }
}
