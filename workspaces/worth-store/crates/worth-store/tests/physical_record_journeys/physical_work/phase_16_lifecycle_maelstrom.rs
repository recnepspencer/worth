mod fixture;
pub(super) mod fresh_process;
mod joined_execution;
mod shutdown_trace;
mod workflows;
mod world;

const MODEL: LifecycleMaelstromModel = LifecycleMaelstromModel {
    disjoint_read_effects: 2,
    retry_write_attempts: 2,
    retry_write_completions: 1,
    exact_writeback_effects: 1,
    fresh_root_generation: 1,
    fresh_records: &[],
};

struct LifecycleMaelstromModel {
    disjoint_read_effects: u64,
    retry_write_attempts: u64,
    retry_write_completions: u64,
    exact_writeback_effects: u64,
    fresh_root_generation: u64,
    fresh_records: &'static [&'static [u8]],
}

#[test]
fn lifecycle_maelstrom_joins_real_authority_effects_and_shutdown() {
    let world = world::open();
    let trace = joined_execution::execute(&world, &MODEL);
    shutdown_trace::close_and_finish(world, trace, &MODEL);
}
