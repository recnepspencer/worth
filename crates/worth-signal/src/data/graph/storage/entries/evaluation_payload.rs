//! Node-local evaluation transformations, independent of installed root ownership.
mod artifacts;
mod diagnostics;
mod lifecycle;
mod revalidation;
mod topology;
mod versions;
pub(in crate::data::graph) use topology::replace_dependency_topology;

pub(in crate::data::graph) use artifacts::apply_artifact_write;
pub(in crate::data::graph) use diagnostics::{set_causality, stamp_lineage_and_execution};
pub(in crate::data::graph) use lifecycle::{transition_clean, transition_dirty};
pub(in crate::data::graph) use revalidation::install_revalidation_resolution;
pub(in crate::data::graph) use versions::apply_aspect_version;
