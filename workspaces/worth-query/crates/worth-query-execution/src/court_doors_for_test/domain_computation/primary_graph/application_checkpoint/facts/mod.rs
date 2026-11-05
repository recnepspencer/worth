//! Doors under `domain_computation::primary_graph::application_checkpoint::facts`.
#[path = "encoding/mod.rs"]
pub mod encoding;
#[path = "fact_decode_denial/mod.rs"]
pub mod fact_decode_denial;
#[path = "payload/mod.rs"]
pub mod payload;
pub use crate::domain_computation::primary_graph::application_checkpoint::facts::MAXIMUM_FACT_BYTES;
pub use crate::domain_computation::primary_graph::application_checkpoint::facts::WIRE_VERSION;
pub use crate::domain_computation::primary_graph::application_checkpoint::facts::decode;
pub use crate::domain_computation::primary_graph::application_checkpoint::facts::encode;
