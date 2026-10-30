//! A captured checkpoint prefix and a later object's drop coexist without losing either custody.

#[allow(unused_imports)] use crate::production_entry::certified_release_serving::release_capacity_budget::checkpoint_overlap::*;

#[test]
fn paused_checkpoint_commits_its_prefix_then_later_drop_tail_reopens() {
    std::thread::Builder::new()
        .name("release-checkpoint-overlap".to_owned())
        .stack_size(16 << 20)
        .spawn(run)
        .expect("overlapping checkpoint worker")
        .join()
        .expect("overlapping checkpoint worker did not panic");
}
