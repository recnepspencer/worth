//! Pre-effect native membership installation and exact predecessor write fences.
use super::*;
pub mod producer_cascade;
pub use producer_cascade::CanonicalProducerRetirement;
pub mod relation_failure;
pub use relation_failure::relation_stop;
pub mod retained_origin;
