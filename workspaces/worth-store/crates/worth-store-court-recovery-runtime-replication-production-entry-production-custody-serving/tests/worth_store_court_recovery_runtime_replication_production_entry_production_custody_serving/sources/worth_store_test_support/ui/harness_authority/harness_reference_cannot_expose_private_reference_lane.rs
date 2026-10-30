use worth_store_test_support::test_authority::HarnessPhysicalReference;

fn main() {
    let reference = HarnessPhysicalReference::for_courtroom_replay(1);
    let _raw = reference.as_physical_reference();
}
