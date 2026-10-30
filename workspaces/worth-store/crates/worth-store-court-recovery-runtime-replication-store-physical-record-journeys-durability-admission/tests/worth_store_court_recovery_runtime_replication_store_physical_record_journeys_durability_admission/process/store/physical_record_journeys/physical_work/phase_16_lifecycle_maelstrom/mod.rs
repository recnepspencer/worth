#[allow(unused_imports)]
use crate::physical_work::phase_16_lifecycle_maelstrom::*;
#[test]
fn lifecycle_maelstrom_joins_real_authority_effects_and_shutdown() {
    let world =
        crate::physical_record_journeys::physical_work::phase_16_lifecycle_maelstrom::world::open();
    let trace = crate::physical_record_journeys::physical_work::phase_16_lifecycle_maelstrom::joined_execution::execute(&world, &MODEL);
    crate::physical_record_journeys::physical_work::phase_16_lifecycle_maelstrom::shutdown_trace::close_and_finish(world, trace, &MODEL);
}
