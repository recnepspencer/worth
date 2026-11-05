//! Inbound occurrences and installed-transport completions: their admission, progression,
//! recovery and publication, over the inbound processing state the runtime composes.
pub(crate) mod inbound_occurrence;
pub(crate) mod inbound_publication;
pub(crate) mod processing;
pub(in crate::domain_computation::primary_graph) use processing::WorthQueryInboundProcessing;
