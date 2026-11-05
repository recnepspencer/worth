//! Original source disclosure meaning; fresh use must independently authorize it.
pub(crate) mod inspection;
pub(crate) mod inspection_capture;
pub(crate) mod original;
mod parameter_capture;

pub(in crate::domain_computation::primary_graph) use inspection_capture::capture_inspection;

pub(in crate::domain_computation::primary_graph) use parameter_capture::capture_parameters;
