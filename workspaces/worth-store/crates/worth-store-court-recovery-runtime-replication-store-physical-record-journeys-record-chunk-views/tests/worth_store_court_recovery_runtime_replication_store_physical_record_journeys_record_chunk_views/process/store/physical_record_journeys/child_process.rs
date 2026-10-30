//! Child roles belong to the court executable that launches them.
mod dispatch {
    use worth_store_testkit_store::child_process::{ROLE_ENV, ROOT_ENV};
    use worth_store_testkit_store::physical_record_journeys::maintenance_interference;

    #[test]
    fn physical_record_child_role() {
        let Ok(role) = std::env::var(ROLE_ENV) else {
            return;
        };
        let root = std::path::PathBuf::from(std::env::var_os(ROOT_ENV).expect("child root"));
        match role.as_str() {
            "interference_baseline_reopener" => {
                maintenance_interference::reopen::serving_child(&root)
            }
            "interference_canonical_child" => {
                maintenance_interference::canonical::canonical_child(&root)
            }
            _ => panic!("unknown maintenance-interference child role: {role}"),
        }
    }
}
