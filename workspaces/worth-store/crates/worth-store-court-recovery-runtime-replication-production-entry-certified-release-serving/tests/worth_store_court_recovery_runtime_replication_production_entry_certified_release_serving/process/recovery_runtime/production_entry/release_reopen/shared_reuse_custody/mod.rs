//! Shared physical payload survives a source-first release and two fresh C8 seals.

#[allow(unused_imports)]
use crate::production_entry::release_reopen::shared_reuse_custody::*;

#[test]
fn source_first_reuse_continues_only_with_fresh_recovery_custody() {
    std::thread::Builder::new()
        .name("source-first-c8-custody".to_owned())
        .stack_size(16 << 20)
        .spawn(run)
        .expect("recovery journey worker")
        .join()
        .expect("recovery journey worker did not panic");
}

#[test]
fn fresh_child_checks_source_first_reuse() {
    crate::production_entry::release_reopen::shared_reuse_custody::fresh_process::run_child();
}
#[path = "continuation_budget/mod.rs"]
pub(crate) mod continuation_budget;
#[path = "destination_first/mod.rs"]
pub(crate) mod destination_first;
#[path = "lagging_watermark/mod.rs"]
pub(crate) mod lagging_watermark;
