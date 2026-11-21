//! Original source disclosure meaning; fresh use must independently authorize it.
pub(crate) use worth_query_execution_lineage::facade::primary_graph::source_disclosure::{inspection, original};
pub(crate) mod inspection_capture;
mod parameter_capture;

pub(in crate::domain_computation) use inspection::OriginalSourceInspectionDisclosure;
pub(in crate::domain_computation::primary_graph) use inspection_capture::capture_inspection;
pub(in crate::domain_computation) use original::OriginalSourceDisclosure;
pub(in crate::domain_computation::primary_graph) use parameter_capture::capture_parameters;
