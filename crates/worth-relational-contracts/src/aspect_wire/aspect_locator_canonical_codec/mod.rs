pub mod borrowed_allocation;
pub mod error;
pub mod reading;
pub mod serde_modules;
pub mod tags;
pub mod writing;

pub use reading::decode_aspect_field_locator;
#[cfg(test)]
pub use reading::{decode_aspect_value_locator, decode_boundary_source_locator};
pub use serde_modules::{
    serde_canonical_aspect_field_locator, serde_canonical_aspect_field_locator_arc_slice,
    serde_canonical_aspect_value_locator,
};
pub use writing::encode_aspect_field_locator;
#[cfg(test)]
pub use writing::{encode_aspect_value_locator, encode_boundary_source_locator};

pub use borrowed_allocation::decoded_bytes as locator_decoded_bytes;

pub use writing::write_aspect_field_locator;
