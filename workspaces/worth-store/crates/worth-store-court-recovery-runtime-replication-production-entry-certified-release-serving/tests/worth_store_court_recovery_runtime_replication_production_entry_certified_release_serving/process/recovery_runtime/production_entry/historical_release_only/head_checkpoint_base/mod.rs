//! Completed B history must rejoin independently from A's selected head checkpoint.

#[allow(unused_imports)]
use crate::production_entry::historical_release_only::head_checkpoint_base::*;

#[test]
fn released_checkpoint_base_survives_completed_postcheckpoint_release() {
    std::thread::Builder::new()
        .name("completed-head-checkpoint-base".into())
        .stack_size(16 << 20)
        .spawn(run)
        .unwrap()
        .join()
        .expect("completed HeadV2 recovery worker");
}
