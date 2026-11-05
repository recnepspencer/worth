//! The runtime implements every module's host trait here: it composes the subsystems.
mod application_checkpoint;
mod application_discovery;
mod application_entry;
mod application_resource;
mod authorization;
mod identity;
mod operation_body;
mod output_demand;
mod principal_currentness_capture;
mod product_activation;
mod resolution;
