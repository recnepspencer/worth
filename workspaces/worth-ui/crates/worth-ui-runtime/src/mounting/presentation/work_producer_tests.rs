#[path = "work_producer_tests/batch_b_preplan_slope.rs"]
mod batch_b_preplan_slope;
#[path = "work_producer_tests/damage_bounds.rs"]
mod damage_bounds;
#[path = "work_producer_tests/delta_source.rs"]
mod delta_source;
#[path = "work_producer_tests/effect_expectations.rs"]
mod effect_expectations;
#[path = "work_producer_tests/membership_change.rs"]
mod membership_change;
#[path = "work_producer_tests/precise_damage.rs"]
mod precise_damage;
#[path = "work_producer_tests/producer_slope.rs"]
mod producer_slope;
#[path = "work_producer_tests/rect_node.rs"]
mod rect_node;
#[path = "work_producer_tests/replacement_damage.rs"]
mod replacement_damage;
#[path = "work_producer_tests/text_node.rs"]
mod text_node;
#[path = "work_producer_tests/total_order.rs"]
mod total_order;
#[path = "work_producer_tests/unchanged_progression.rs"]
mod unchanged_progression;
#[path = "work_producer_tests/world.rs"]
pub(super) mod world;

use super::work_producer::UiMountedPresentationState;
use world::{rect_spec, rect_spec_with_clip, MountedPresentationWorld};
