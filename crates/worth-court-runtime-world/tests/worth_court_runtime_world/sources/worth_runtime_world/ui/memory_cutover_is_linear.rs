use worth_runtime_world::facade::PreparedWorldRelationalCutover;

fn reuse(cutover: PreparedWorldRelationalCutover<'_, '_, '_, '_>) {
    cutover.perform_before(std::time::Instant::now());
    cutover.perform();
}

fn main() {}
