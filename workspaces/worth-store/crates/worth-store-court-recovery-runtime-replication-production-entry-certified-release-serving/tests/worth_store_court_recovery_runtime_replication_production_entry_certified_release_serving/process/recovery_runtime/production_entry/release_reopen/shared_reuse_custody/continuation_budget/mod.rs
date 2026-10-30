//! Real COW continuation backing denial, independently observed at media.

#[allow(unused_imports)]
use crate::production_entry::release_reopen::shared_reuse_custody::continuation_budget::*;

#[test]
fn continuation_backing_denies_before_cow_and_successful_twin_appends() {
    std::thread::Builder::new()
        .name("cow-continuation-backing".to_owned())
        .stack_size(16 << 20)
        .spawn(run)
        .expect("continuation worker")
        .join()
        .expect("continuation worker did not panic");
}
