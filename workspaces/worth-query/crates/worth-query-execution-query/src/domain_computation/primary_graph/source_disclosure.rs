//! Original source disclosure meaning; fresh use must independently authorize it.
pub use worth_query_execution_lineage::facade::primary_graph::source_disclosure::{inspection, original};
pub mod inspection_capture;
mod parameter_capture;

pub use inspection::OriginalSourceInspectionDisclosure;
pub use inspection_capture::capture_inspection;
pub use original::OriginalSourceDisclosure;
pub use parameter_capture::capture_parameters;
