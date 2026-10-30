//! Destination-first release of a reused publication: the drop invalidates the
//! directory watermark, publishes a replacement binding in the same transition,
//! and the surviving source stays readable through ordinary catalog reads
//! before and after a checkpoint and fresh C8 reopen.

#[allow(unused_imports)]
use crate::production_entry::release_reopen::shared_reuse_custody::destination_first::*;

#[test]
fn destination_first_release_keeps_the_source_routed_through_fresh_recovery_custody() {
    std::thread::Builder::new()
        .name("destination-first-c8-custody".to_owned())
        .stack_size(16 << 20)
        .spawn(run)
        .expect("recovery journey worker")
        .join()
        .expect("recovery journey worker did not panic");
}
