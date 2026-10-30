//! Releasing a lagging directory watermark while a newer publication survives:
//! the replacement binding drops only the invalid watermark, keeps the newer
//! latest hint, and leaves catalog reads stale until an ordinary rebuild.

#[allow(unused_imports)]
use crate::production_entry::release_reopen::shared_reuse_custody::lagging_watermark::*;

#[test]
fn released_lagging_watermark_keeps_the_newer_latest_publication() {
    std::thread::Builder::new()
        .name("lagging-watermark-release".to_owned())
        .stack_size(16 << 20)
        .spawn(run)
        .expect("recovery journey worker")
        .join()
        .expect("recovery journey worker did not panic");
}
