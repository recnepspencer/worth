//! A completed partial drop leaves a standing checkpoint reservation. Its
//! checkpoint runs on that reservation even when every other Recovery byte
//! is held, and a cancelled capture leaves the reservation whole.

#[allow(unused_imports)] use crate::production_entry::certified_release_serving::release_capacity_budget::checkpoint_funding::*;

#[test]
fn partial_drop_checkpoints_on_its_standing_reservation_and_reopens() {
    std::thread::Builder::new()
        .name("release-checkpoint-reservation".to_owned())
        .stack_size(16 << 20)
        .spawn(run)
        .expect("checkpoint reservation worker")
        .join()
        .expect("checkpoint reservation worker did not panic");
}
