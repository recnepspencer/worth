//! The parent and its child roles execute in the same owning court binary.
mod dispatch {
    #[test]
    fn physical_record_child_role() {
        worth_store_testkit_store::physical_record_journeys::child_process::dispatch_child();
    }
}
