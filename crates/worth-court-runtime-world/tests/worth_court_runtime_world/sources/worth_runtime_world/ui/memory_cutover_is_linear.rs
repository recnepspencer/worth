use worth_runtime_world::facade::PreparedWorldRelationalCutover;

fn reuse(cutover: PreparedWorldRelationalCutover<'_, '_, '_, '_>) {
    cutover.perform_before(worth_runtime_world::facade::RuntimeWorldInstant::from_ticks(1));
    cutover.perform();
}

fn main() {}
