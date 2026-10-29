#[allow(unused_imports)] use crate::physical_record_journeys::child_process::dispatch::*;
use {std::path::PathBuf};

use {worth_store_testkit_store::child_process::LOCATOR_ENV, worth_store_testkit_store::child_process::ORACLE_ENV, worth_store_testkit_store::child_process::ROLE_ENV, worth_store_testkit_store::child_process::ROOT_ENV};

#[test]
fn c5_child_role() {
    let Ok(role) = std::env::var(ROLE_ENV) else {
        return;
    };
    let root = std::path::PathBuf::from(std::env::var_os(ROOT_ENV).unwrap());
    dispatch(&role, &root);
}
