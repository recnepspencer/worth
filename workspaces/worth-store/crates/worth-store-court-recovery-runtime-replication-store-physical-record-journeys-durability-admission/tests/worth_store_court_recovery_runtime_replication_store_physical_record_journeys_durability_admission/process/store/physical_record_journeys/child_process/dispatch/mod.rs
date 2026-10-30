#[allow(unused_imports)]
use crate::physical_record_journeys::child_process::dispatch::*;

use {
    worth_store_testkit_store::child_process::ROLE_ENV,
    worth_store_testkit_store::child_process::ROOT_ENV,
};

#[test]
fn physical_record_child_role() {
    let Ok(role) = std::env::var(ROLE_ENV) else {
        return;
    };
    let root = std::path::PathBuf::from(std::env::var_os(ROOT_ENV).unwrap());
    dispatch(&role, &root);
}
