mod front_doors;
mod json_input;
mod json_lowering;
mod outcome;
mod reference_lowering;
mod scalar_lowering;

pub use front_doors::{compatibility, CompatibilityFrontDoor, JsonCompatibilityFrontDoor};
pub use json_input::JsonCompatibilityAspectInput;
pub use json_lowering::{lower_json_aspect_value, lower_json_record_aspect_state};
pub use outcome::{
    JsonCompatibilityLoweringDeferred, JsonCompatibilityLoweringDenial,
    JsonCompatibilityLoweringFailure, JsonCompatibilityLoweringOutcome,
    JsonCompatibilityLoweringStale, JsonCompatibilityRebindRequired,
};

use crate::facade::ResponsibilityArea;

pub fn responsibility() -> ResponsibilityArea {
    ResponsibilityArea::new(
        "compatibility_bridges",
        "explicit transitional lowering boundaries into canonical aspect-native meaning",
        "long-term JSON-shaped authoritative truth",
    )
}
