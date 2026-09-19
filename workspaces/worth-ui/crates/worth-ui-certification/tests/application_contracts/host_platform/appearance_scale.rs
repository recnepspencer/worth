mod appearance;
mod authoring;
mod execution;
mod geometry;

const NODE_COUNT: usize = 4_096;
const STYLED_COUNT: usize = 3_072;
const ROLE_COUNT: usize = 256;
const SLOT_COUNT: usize = 512;
const NEIGHBORHOOD_COUNT: usize = 64;
const SURFACE_COUNT: usize = 4;
const CONSUMERS_PER_ROLE: usize = 12;

#[test]
#[ignore = "AP10 closure stress: combined appearance, Motion, Portal, and Backdrop scale world"]
fn ap10_theme_switch_selects_exact_consumers_in_the_4096_node_world() {
    execution::verify();
}
