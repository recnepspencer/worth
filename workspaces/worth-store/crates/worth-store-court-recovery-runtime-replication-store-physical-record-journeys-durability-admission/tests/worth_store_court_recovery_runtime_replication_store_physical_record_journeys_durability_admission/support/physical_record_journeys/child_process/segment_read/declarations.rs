pub(crate) fn run(root: &std::path::Path) {
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_open(root);
    worth_store_testkit_store::physical_record_journeys::child_process::segment_reader(serving);
}
