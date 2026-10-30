//! Run the scale observer in the independently spawned owning court executable.
mod dispatch {
    #[test]
    fn physical_record_child_role() {
        use worth_store_testkit_store::child_process::{ROLE_ENV, ROOT_ENV};

        if std::env::var(ROLE_ENV).as_deref() == Ok("segment_reader") {
            let root = std::path::PathBuf::from(std::env::var_os(ROOT_ENV).unwrap());
            let serving =
                worth_store_testkit_store::physical_record_journeys::serving_from_open(&root);
            worth_store_testkit_store::physical_record_journeys::child_process::segment_reader(
                serving,
            );
            return;
        }
        worth_store_testkit_store::physical_record_journeys::child_process::dispatch_child();
    }
}
